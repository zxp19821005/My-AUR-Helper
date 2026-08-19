/**
 * graphql_batch_parse.rs - GitHub GraphQL 批量查询的快照解析
 *
 * 将单次分批查询返回的仓库 JSON 对象解析为内存快照（RepoSnapshot），
 * 供 graphql_batch.rs 的版本挑选逻辑消费。
 *
 * 设计要点：
 * - 资产仅保留 name 字段，以复用 binary_check 的既有 Linux 二进制判定
 * - 与 REST 路径返回的数据结构语义保持一致，确保挑选结果等价
 */
use serde_json::Value;

/// 仓库快照：一次 GraphQL 查询解析出的单个仓库数据
pub struct RepoSnapshot {
    /// tag 名称列表（refs/tags）
    pub tags: Vec<String>,
    /// release 精简数据列表（按创建时间倒序）
    pub releases: Vec<ReleaseData>,
    /// License SPDX ID（如 "MIT"）
    pub license: Option<String>,
    /// 编程语言名称列表（按字节数降序）
    pub languages: Vec<String>,
}

/// Release 精简数据
pub struct ReleaseData {
    /// tag 名称
    pub tag_name: String,
    /// release 标题
    pub name: String,
    /// 是否为预发布版本
    pub is_prerelease: bool,
    /// 是否为草稿
    pub is_draft: bool,
    /// 创建时间（ISO8601 字符串，可字典序比较）
    pub created_at: String,
    /// 资产文件（仅保留 name 字段，复用 binary_check 判定）
    pub assets: Vec<Value>,
}

/// 解析单个仓库对象为快照
///
/// # 参数
/// - `repo`: GraphQL 返回的 repository 对象
///
/// # 返回
/// - `RepoSnapshot`: 包含 tags / releases / license / languages 的快照
pub fn parse_snapshot(repo: &Value) -> RepoSnapshot {
    let tags = repo
        .get("refs")
        .and_then(|r| r.get("nodes"))
        .and_then(|n| n.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|n| n.get("name").and_then(|s| s.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let releases = repo
        .get("releases")
        .and_then(|r| r.get("nodes"))
        .and_then(|n| n.as_array())
        .map(|arr| {
            arr.iter()
                .map(|r| ReleaseData {
                    tag_name: r
                        .get("tagName")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    name: r
                        .get("name")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    is_prerelease: r
                        .get("isPrerelease")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(false),
                    is_draft: r.get("isDraft").and_then(|b| b.as_bool()).unwrap_or(false),
                    created_at: r
                        .get("createdAt")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    assets: r
                        .get("assets")
                        .and_then(|a| a.get("nodes"))
                        .and_then(|n| n.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| {
                                    x.get("name")
                                        .and_then(|s| s.as_str())
                                        .map(|n| serde_json::json!({ "name": n }))
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();

    let license = repo
        .get("licenseInfo")
        .and_then(|l| l.get("spdxId"))
        .and_then(|s| s.as_str())
        .filter(|s| !s.is_empty() && *s != "NOASSERTION")
        .map(str::to_string);

    let languages = repo
        .get("languages")
        .and_then(|l| l.get("nodes"))
        .and_then(|n| n.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|n| n.get("name").and_then(|s| s.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    RepoSnapshot {
        tags,
        releases,
        license,
        languages,
    }
}
