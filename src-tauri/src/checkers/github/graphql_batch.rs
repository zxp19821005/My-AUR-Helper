/**
 * graphql_batch.rs - GitHub 上游版本批量检查（GraphQL）
 *
 * 通过 GitHub GraphQL API 在单次请求中批量查询多个仓库的
 * tags / releases / license / languages，将 N 次 REST 请求压缩为 ~1 次/分块，
 * 显著缓解上游限流并降低请求总量。
 *
 * 设计要点：
 * - 按 owner/repo 去重：多个软件包指向同一仓库时仅查询一次（结果按各包选项分别挑选）
 * - 分块请求：受 GraphQL 复杂度 / 响应体积约束，每批最多 MAX_REPOS_PER_QUERY 个仓库
 * - 无 Token 时返回空结果，交由调用方回退到逐包 REST（无 Token 限流极严，GraphQL 基本不可用）
 * - git 包（package_type == Git）不走批量：git describe 需 commit 计数，无廉价 GraphQL 等价，由调用方逐包处理
 * - 仓库缺失 / 分块失败时该仓库不产出结果，调用方自动回落到逐包 REST
 */
use std::collections::HashMap;

use log::warn;
use reqwest::Client;
use serde_json::Value;
use tokio::task::JoinSet;

use crate::checkers::github::graphql_batch_parse::parse_snapshot;
use crate::checkers::github::graphql_batch_helpers::select_version;
use crate::models::PackageType;

/// 每批 GraphQL 查询的仓库数量上限
///
/// 受 GitHub GraphQL 复杂度评分与单次响应体积约束：每个仓库需拉取
/// tags(100) + releases(20, 含 assets 12) + languages(5) + license，
/// 取保守值避免触发限制或产生过大 JSON。
const MAX_REPOS_PER_QUERY: usize = 10;

/// 单个软件包的 GitHub 批量检查输入（由 PackageTask 映射，携带解析出的 owner/repo）
pub struct GithubBatchItem {
    /// 软件包名称
    pub pkgname: String,
    /// 软件包数据库 ID
    pub software_id: i64,
    /// 仓库所有者
    pub owner: String,
    /// 仓库名称
    pub repo: String,
    /// 软件包类型（git 包已被调用方排除，仅供透传/日志）
    pub package_type: PackageType,
    /// 是否检查测试版本（prerelease）
    pub check_test_versions: bool,
    /// 是否检查二进制文件（决定资产过滤逻辑）
    pub check_binary_files: bool,
    /// 版本提取正则表达式（可选）
    pub version_extract_regex: Option<String>,
}

/// 单个软件包的 GitHub 批量检查结果
pub struct GithubBatchOutcome {
    /// 软件包名称
    pub pkgname: String,
    /// 软件包数据库 ID
    pub software_id: i64,
    /// 上游版本号（无版本时为 None）
    pub version: Option<String>,
    /// License SPDX ID
    pub license_spdx_id: Option<String>,
    /// 编程语言名称列表
    pub language_names: Vec<String>,
}

/// 通过 GitHub GraphQL 批量检查上游版本
///
/// 工作流程：
/// 1. 无 Token 时直接返回空（交由调用方回退 REST）
/// 2. 按 owner/repo 去重并按 MAX_REPOS_PER_QUERY 分块
/// 3. 每块发一次 GraphQL 请求，解析出各仓库快照
/// 4. 对每个包按自身选项（测试版本 / 二进制 / 正则）挑选版本
///
/// # 参数
/// - `client`: 共享 HTTP 客户端（含代理配置）
/// - `items`: 待检查的 GitHub 软件包列表
/// - `token`: GitHub Token（可选，必需）
///
/// # 返回
/// - 已成功产出结果的软件包列表（未命中仓库不会出现在结果中）
pub async fn batch_check_github(
    client: &Client,
    items: Vec<GithubBatchItem>,
    token: Option<&str>,
) -> Vec<GithubBatchOutcome> {
    let mut outcomes = Vec::new();
    let token = match token {
        Some(t) if !t.is_empty() => t,
        _ => return outcomes, // 无 Token：批量不可用，全部回退 REST
    };

    // 构建 (owner,repo) -> 包列表 索引：将「去重」与「按仓库匹配包」合并为一次 O(n)
    // 哈希构建——去重由 HashMap 键完成（替代原 Vec::contains 的 O(n²)），后续按仓库取
    // 包为 O(1)（替代原每层 repo 线性扫描全部 items 的 O(n²)）
    let mut repo_packages: HashMap<(String, String), Vec<&GithubBatchItem>> = HashMap::new();
    for it in &items {
        repo_packages
            .entry((it.owner.clone(), it.repo.clone()))
            .or_default()
            .push(it);
    }
    let repos: Vec<(String, String)> = repo_packages.keys().cloned().collect();

    // 按分块并行发送 GraphQL 请求（JoinSet），将串行分块的总耗时压平为最慢一块；
    // 单块失败 / 仓库缺失交由调用方按包回落逐包 REST，不阻断其他块
    let mut set = JoinSet::new();
    for chunk in repos.chunks(MAX_REPOS_PER_QUERY) {
        let chunk: Vec<(String, String)> = chunk.to_vec();
        let client = client.clone(); // Client 内部为 Arc，clone 开销极小
        let token = token.to_string();
        set.spawn(async move {
            let res = query_chunk(&client, &chunk, &token).await;
            (chunk, res)
        });
    }

    // 收集各分块结果（alias r{idx} -> 仓库对象），按仓库索引产出各包结果
    while let Some(join_res) = set.join_next().await {
        let (chunk, snapshot_map) = match join_res {
            Ok(v) => v,
            Err(e) => {
                warn!("[GitHub GraphQL] 分块任务失败: {}", e);
                continue;
            }
        };
        let snapshot_map = match snapshot_map {
            Some(m) => m,
            None => continue, // 整批失败：这些仓库回落 REST
        };
        for (idx, (owner, repo)) in chunk.iter().enumerate() {
            let alias = format!("r{}", idx);
            let snap = match snapshot_map.get(&alias).map(parse_snapshot) {
                Some(s) => s,
                None => continue, // 仓库缺失（404）：回落 REST
            };
            if let Some(pkgs) = repo_packages.get(&(owner.clone(), repo.clone())) {
                for it in pkgs {
                    let version = select_version(&snap, it);
                    outcomes.push(GithubBatchOutcome {
                        pkgname: it.pkgname.clone(),
                        software_id: it.software_id,
                        version,
                        license_spdx_id: snap.license.clone(),
                        language_names: snap.languages.clone(),
                    });
                }
            }
        }
    }
    outcomes
}

/// 构建针对一批仓库的 GraphQL 查询（alias 批量查询）
fn build_query(repos: &[(String, String)]) -> String {
    let mut blocks = String::new();
    for (i, (owner, repo)) in repos.iter().enumerate() {
        let alias = format!("r{}", i);
        // 用 serde_json 序列化 owner/repo，保证生成合法 JSON 字符串字面量（自动转义引号/特殊字符）
        let owner_json = serde_json::to_string(owner).unwrap();
        let repo_json = serde_json::to_string(repo).unwrap();
        blocks.push_str(&format!(
            "{alias}: repository(owner: {owner_json}, name: {repo_json}) {{ \
              licenseInfo {{ spdxId }} \
              languages(first: 5, orderBy: {{ field: SIZE, direction: DESC }}) {{ nodes {{ name }} }} \
              refs(first: 100, refPrefix: \"refs/tags/\") {{ nodes {{ name }} }} \
              releases(first: 20, orderBy: {{ field: CREATED_AT, direction: DESC }}) {{ \
                nodes {{ tagName name isPrerelease isDraft createdAt \
                  assets(first: 12) {{ nodes {{ name }} }} }} }} \
            }}"
        ));
    }
    format!("query {{ {blocks} }}")
}

/// 执行一次 GraphQL 分块查询，返回 alias -> 仓库对象（跳过 null 项）
async fn query_chunk(
    client: &Client,
    repos: &[(String, String)],
    token: &str,
) -> Option<HashMap<String, Value>> {
    let body = serde_json::json!({ "query": build_query(repos) });
    let mut req = client
        .post("https://api.github.com/graphql")
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github+json")
        .json(&body);
    req = req.header("Authorization", format!("Bearer {}", token));

    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            warn!("[GitHub GraphQL] 请求失败: {}", e);
            return None;
        }
    };
    if !resp.status().is_success() {
        warn!("[GitHub GraphQL] HTTP {}", resp.status());
        return None;
    }
    let data: Value = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            warn!("[GitHub GraphQL] 响应解析失败: {}", e);
            return None;
        }
    };
    // 单个仓库不存在时 GitHub 对该 alias 返回 null；errors 仅记录、不阻断整批
    let data_obj = data.get("data").and_then(|d| d.as_object())?;
    let mut map = HashMap::new();
    for (k, v) in data_obj.iter() {
        if !v.is_null() {
            map.insert(k.clone(), v.clone());
        }
    }
    Some(map)
}
