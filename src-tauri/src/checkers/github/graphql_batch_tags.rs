/**
 * graphql_batch_tags.rs - GitHub GraphQL 批量检查辅助函数
 *
 * 从 graphql_batch.rs 拆分，负责：
 * - fill_tags: REST tags 回填（从 GraphQL 快照之后继续翻页，用于正则钉死 major 的包）
 * - collect_fallback_tasks: 识别需要 tags 回填的包（version 为空且有正则）
 *
 * 设计要点：
 * - fill_tags 复用 GraphQL 已扫描的 tags 起始位置，避免重复请求
 * - REST sort=pushed_at&direction=desc 与 GraphQL CREATED_AT DESC 顺序一致，可直接拼接
 */
use std::collections::HashMap;

use reqwest::Client;

// GithubBatchItem / GithubBatchOutcome / RepoCache 定义在同级 graphql_batch 模块中，
// 需通过完整子模块路径引入（github/mod.rs 不 re-export 这三个类型）
use super::graphql_batch::{GithubBatchItem, GithubBatchOutcome};

/// 对指定 (owner,repo) 的 tags 列表进行回填（REST 翻页，复用已扫过的 GraphQL tags）
///
/// GraphQL 快照的 tags 窗口（500 条，按 CREATED_AT DESC）能覆盖当前活跃 major 线，
/// 但对钉死历史 major 线的正则（如 `v10\.\d+\.\d+`）仍可能不足。
/// 本函数从 GraphQL 最大 pushed_at 之后的 tag 继续翻页，直到：
/// - 命中目标正则 → 停止（节省后续请求）
/// - 扫描完所有 tags → 停止
/// - 达到 max_pages 上限 → 停止（避免超时）
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner` / `repo`: 仓库所有者和名称
/// - `token`: GitHub Token（可选）
/// - `existing_tags`: GraphQL 已扫过的 tags（作为起点，非替换）
/// - `regex`: 版本正则（可选，命中即早停）
/// - `max_pages`: 最多翻页页数（默认 30，对应 3000 tag）
///
/// # 返回
/// 包含 existing_tags 拼接后结果的完整 tags 列表（按 pushed_at DESC）
pub async fn fill_tags(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    existing_tags: &[String],
    regex: Option<&str>,
    max_pages: u32,
) -> Vec<String> {
    use log::debug;
    if existing_tags.len() >= max_pages as usize * 100 {
        // GraphQL 已拿到 3000+ tag，不再翻页
        return existing_tags.to_vec();
    }

    let per_page = 100;
    let max_pages = max_pages as usize;
    let mut all_tags = existing_tags.to_vec();

    // 确定起始页号：GraphQL 约覆盖了 existing_tags.len() / per_page 页
    // div_ceil 为向上取整，与手写 (len + per_page - 1) / per_page 等价
    let start_page = existing_tags.len().div_ceil(per_page) + 1;

    let regex_compile = regex.and_then(|r| match regex::Regex::new(r) {
        Ok(re) => Some(re),
        Err(e) => {
            debug!(
                "[tags 回填] {}:{}/tags: 正则编译失败 {}, 跳过回填",
                owner, repo, e
            );
            None
        }
    });

    for page in start_page..=start_page + max_pages - 1 {
        let tags_url = format!(
            "https://api.github.com/repos/{}/{}/tags?per_page={}&page={}&sort=pushed_at&direction=desc",
            owner, repo, per_page, page
        );

        let mut req = client
            .get(&tags_url)
            .header("User-Agent", "my-aur-helper/0.1")
            .header("Accept", "application/vnd.github.v3+json");
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                debug!(
                    "[tags 回填] {}:{}/tags page={}: send error: {}",
                    owner, repo, page, e
                );
                break;
            }
        };
        if !resp.status().is_success() {
            debug!(
                "[tags 回填] {}:{}/tags page={}: HTTP {}",
                owner,
                repo,
                page,
                resp.status()
            );
            break;
        }

        let tags: Vec<serde_json::Value> = match resp.json().await {
            Ok(t) => t,
            Err(e) => {
                debug!(
                    "[tags 回填] {}:{}/tags page={}: json decode error: {}",
                    owner, repo, page, e
                );
                break;
            }
        };

        if tags.is_empty() {
            break;
        }

        let mut matched_any = false;
        for tag in &tags {
            if let Some(name) = tag["name"].as_str() {
                all_tags.push(name.to_string());
                // 正则早停：命中即止
                if let Some(re) = &regex_compile {
                    if re.is_match(name) {
                        matched_any = true;
                    }
                }
            }
        }
        if matched_any {
            debug!(
                "[tags 回填] {}:{}/tags page={}: regex matched, stopping early (total tags: {})",
                owner,
                repo,
                page,
                all_tags.len()
            );
            break;
        }
        if tags.len() < per_page {
            break;
        }
    }

    all_tags
}

/// 为所有有正则的包检查是否需要 tags 回填
///
/// 逻辑：
/// - 若包的 GraphQL 结果 version 非空 → 命中，无需回填
/// - 若包的 version 为空 且 有正则 → 需要回填（GraphQL 500 tag 窗口不够）
/// - 若无正则 → 不需要回填（稳定版取 latest release，GraphQL 已覆盖）
///
/// # 参数
/// - `items`: 所有 GitHub 批量检查输入
/// - `outcomes`: 批量检查结果
///
/// # 返回
/// `Vec<(owner, repo, regex)>` — 需要回填的包列表
pub fn collect_fallback_tasks(
    items: &[GithubBatchItem],
    outcomes: &[GithubBatchOutcome],
) -> Vec<(String, String, String)> {
    // 构建 pkgname -> outcome 映射
    let outcome_map: HashMap<&str, &GithubBatchOutcome> =
        outcomes.iter().map(|o| (o.pkgname.as_str(), o)).collect();

    let mut fallbacks: Vec<(String, String, String)> = Vec::new();
    for item in items {
        if item.version_extract_regex.is_none() {
            continue;
        }
        // version 为空 → GraphQL 未找到匹配版本，需要 REST tags 回填
        if outcome_map
            .get(item.pkgname.as_str())
            .map(|o| o.version.is_none())
            .unwrap_or(false)
        {
            fallbacks.push((
                item.owner.clone(),
                item.repo.clone(),
                item.version_extract_regex.clone().unwrap(),
            ));
        }
    }
    fallbacks
}
