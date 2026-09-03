//! cache_fill.rs — REST 回落场景的 GitHub 标签缓存补写
//!
//! 背景：batch 引擎仅在 GraphQL 快照成功时写缓存（batch.rs 遍历 cache_map 调 on_cache_ready）。
//! 当 GraphQL 失败回落到 REST 时，结果走 `checked.push(result)` 直接返回，
//! 仓库快照不会进入 cache_map，该仓库**永不写入缓存**——于是每次检查都重复请求 GitHub API。
//! 这正是 electron/electron 这类仓库缓存恒为空的原因。
//!
//! 本模块在检查完成后，为这些"漏写"的仓库补拉 tags 并写入 github_tag_cache。
use log::{info, warn};
use std::collections::HashMap;

use crate::checkers::github::graphql_batch::{fill_tags, RepoCache};
use crate::checkers::utils::extract_owner_repo;
use crate::commands::sysops::software_sync::batch_engine::PackageTask;
use crate::commands::sysops::software_sync::utils::UpstreamCheckResult;
use crate::db::Database;
use crate::errors::AppResult;
use crate::models::CheckerType;

/// 补写时最多翻取的 tag 页数（每页 100）。
///
/// 默认 60 页（6000 tag）覆盖绝大多数仓库；electron/electron 有 ~5100 个 tag，
/// 需要超过 30 页才能覆盖所有 major 线（v2-v46）。此处设 60 页以平衡完整性和请求次数，
/// 空页自动 break 避免过度请求。
const MAX_TAG_PAGES: u32 = 60;

/// 收集"检查成功但缓存仍缺失"的 GitHub 仓库
///
/// 条件：GitHub 检查器 + 本次成功产出版本 + 未被 batch 快照覆盖 + 库中尚无缓存记录
pub fn collect_missing_repos(
    db: &Database,
    tasks: &[PackageTask],
    results: &[UpstreamCheckResult],
    covered: &HashMap<(String, String), RepoCache>,
) -> Vec<(String, String)> {
    // 本次成功产出版本的包名集合
    let ok_pkgs: std::collections::HashSet<&str> = results
        .iter()
        .filter(|r| !r.upstream_version.is_empty())
        .map(|r| r.pkgname.as_str())
        .collect();

    let mut repos: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for t in tasks {
        if !matches!(
            t.checker_type,
            CheckerType::GitHubTags | CheckerType::GitHubAPI
        ) {
            continue;
        }
        if !ok_pkgs.contains(t.pkgname.as_str()) {
            continue;
        }
        let Some((owner, repo)) = extract_owner_repo(&t.upstream_url) else {
            continue;
        };
        if covered.contains_key(&(owner.clone(), repo.clone())) {
            continue; // batch 内部已写入快照，无需重复
        }
        if matches!(db.get_cache(&owner, &repo), Ok(Some(_))) {
            continue; // 库中已有记录，是否过期交由 check_and_extend_cache 增量校验处理
        }
        repos.insert((owner, repo));
    }
    repos.into_iter().collect()
}

/// 为缺失仓库补拉 tags（不持有数据库引用，可安全跨越 await）
///
/// 正则传 None：缓存是仓库级共享的（37 个 electronXX-bin 共用 electron/electron），
/// 若按某个包的正则早停，会截断其他 major 线所需的 tag。
pub async fn fetch_repo_tags(
    client: &reqwest::Client,
    repos: Vec<(String, String)>,
    token: Option<&str>,
) -> HashMap<(String, String), Vec<String>> {
    let mut map = HashMap::new();
    for (owner, repo) in repos {
        let tags = fill_tags(client, &owner, &repo, token, &[], None, MAX_TAG_PAGES).await;
        if tags.is_empty() {
            warn!(
                "[GitHub Cache] 补拉 {}/{} 未获得 tag，跳过写盘",
                owner, repo
            );
            continue;
        }
        info!(
            "[GitHub Cache] 补写 {}/{} 共 {} 个 tag",
            owner,
            repo,
            tags.len()
        );
        map.insert((owner, repo), tags);
    }
    map
}

/// 将补拉到的 tags 写入缓存
///
/// cached_version 传 None：仓库级 tags 列表无法代表某个具体包的版本，
/// 交由 check_and_extend_cache 在缓存过期时用 releases/latest 做增量校验。
pub fn write_back(
    db: &Database,
    tags_map: HashMap<(String, String), Vec<String>>,
) -> AppResult<()> {
    for ((owner, repo), tags) in tags_map {
        let data = serde_json::json!({ "tags": tags, "releases": [] }).to_string();
        if let Err(e) = db.upsert_cache(&owner, &repo, tags.len() as i32, &data, None) {
            warn!("[GitHub Cache] 补写 {}/{} 失败: {}", owner, repo, e);
        }
    }
    Ok(())
}
