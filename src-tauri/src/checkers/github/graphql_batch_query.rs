/**
 * graphql_batch_query.rs - GitHub GraphQL 批量查询底层实现
 *
 * 功能：
 * - build_query: 构建针对一批仓库的 GraphQL 查询（alias 批量查询）
 * - query_chunk: 执行一次 GraphQL 分块查询，返回 alias -> 仓库对象
 *
 * 设计要点：
 * - 本模块仅负责 GraphQL 查询的构建与执行，不含版本挑选逻辑
 * - 从 graphql_batch.rs 拆分以符合 300 行约束
 */
use std::collections::HashMap;

use log::warn;
use reqwest::Client;
use serde_json::Value;

/// 构建针对一批仓库的 GraphQL 查询（alias 批量查询）
///
/// # schema 约束（2026-10-08 实测校正，勿再回退）
/// - `refs` 的 `orderBy.field`（类型 `RefOrderField`）只接受
///   `ALPHABETICAL` / `TAG_COMMIT_DATE`；**没有 `CREATED_AT`**
///   （`CREATED_AT` 仅用于 `releases` / `languages` 等其他 connection）
/// - `Ref` 类型**没有 `pushedAt` 字段**（tag 的提交时间需走
///   `refs → nodes → target { ... on Commit { committedDate } }`，
///   但那会让每条 ref 触发一次 commit 解析，代价高且非必需）
/// - `refs` 的 `first` 上限为 **100**（不是 releases 的 500）。
///   超限时 GitHub 返回 `Requesting N records ... exceeds the first limit
///   of 100 records`，并把**整个 repository 对象置为 null**，
///   导致该批次所有仓库的快照全部丢失、静默回落 REST。
///
/// 因此 tag 快照窗口固定为最近 100 个 tag（按 tag 提交时间倒序）；
/// 超出窗口的历史 major 线由 `fill_tags` 走 REST `/tags` 翻页补齐。
pub fn build_query(repos: &[(String, String)]) -> String {
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
              refs(first: {REFS_PAGE_LIMIT}, refPrefix: \"refs/tags/\", orderBy: {{ field: TAG_COMMIT_DATE, direction: DESC }}) {{ nodes {{ name }} }} \
              releases(first: 50, orderBy: {{ field: CREATED_AT, direction: DESC }}) {{ \
                nodes {{ tagName name isPrerelease isDraft createdAt \
                  releaseAssets(first: 12) {{ nodes {{ name }} }} }} }} \
            }}"
        ));
    }
    format!("query {{ {blocks} }}")
}

/// `refs` connection 单页上限（GitHub schema 硬限制为 100）
pub const REFS_PAGE_LIMIT: u32 = 100;

/// 执行一次 GraphQL 分块查询，返回 alias -> 仓库对象（跳过 null 项）
pub async fn query_chunk(
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
    // 顶层 errors（schema 不兼容 / 限流 / 字段错误等）：记录但继续尝试部分结果
    if let Some(errs) = data.get("errors").and_then(|d| d.as_array()) {
        warn!(
            "[GitHub GraphQL] GraphQL errors ({} 项)，可能 schema 不兼容或限流；将尝试部分结果",
            errs.len()
        );
    }
    // 单个仓库不存在时 GitHub 对该 alias 返回 null；errors 仅记录、不阻断整批
    let data_obj = match data.get("data").and_then(|d| d.as_object()) {
        Some(o) => o,
        None => {
            // 整个 data 缺失：queries 全部失败，本次查询无产出
            return None;
        }
    };
    let mut map = HashMap::new();
    for (k, v) in data_obj.iter() {
        if !v.is_null() {
            map.insert(k.clone(), v.clone());
        }
    }
    Some(map)
}

#[cfg(test)]
mod tests {
    use super::{build_query, REFS_PAGE_LIMIT};

    /// 回归测试：`refs` 的 `orderBy.field` 必须是 `TAG_COMMIT_DATE`。
    ///
    /// 2026-10-08 实测：传 `CREATED_AT` 会导致
    /// `Argument 'field' on InputObject 'RefOrder' has an invalid value (CREATED_AT)`，
    /// 该 alias 整块数据作废 → 静默回落 REST。
    #[test]
    fn refs_order_field_must_be_tag_commit_date() {
        let q = build_query(&[("owner".into(), "repo".into())]);
        assert!(
            q.contains("orderBy: { field: TAG_COMMIT_DATE, direction: DESC }"),
            "refs 排序字段必须是 TAG_COMMIT_DATE，实际查询: {q}"
        );
        // CREATED_AT 只能出现在 releases / languages 上，不能出现在 refs 上
        let refs_seg = q
            .split("refs(")
            .nth(1)
            .and_then(|s| s.split(')').next())
            .expect("查询应含 refs 段");
        assert!(
            !refs_seg.contains("CREATED_AT"),
            "refs 段不应出现 CREATED_AT（RefOrder 无此枚举），实际: {refs_seg}"
        );
    }

    /// 回归测试：`Ref` 类型没有 `pushedAt` 字段。
    ///
    /// 2026-10-08 实测：请求 `pushedAt` 返回
    /// `Field 'pushedAt' doesn't exist on type 'Ref'`。
    #[test]
    fn refs_must_not_request_pushed_at() {
        let q = build_query(&[("owner".into(), "repo".into())]);
        assert!(
            !q.contains("pushedAt"),
            "Ref 无 pushedAt 字段，查询中不应出现: {q}"
        );
    }

    /// 回归测试：`refs(first: N)` 不得超过 100。
    ///
    /// 2026-10-08 实测：`first: 500` 返回
    /// `Requesting 500 records on the 'refs' connection exceeds the 'first' limit of 100 records`，
    /// 且 GitHub 会把**整个 repository 对象置为 null** —— 一次 schema 违规导致整批快照全丢。
    #[test]
    fn refs_first_must_not_exceed_100() {
        // REFS_PAGE_LIMIT 是常量，运行时 assert! 会被 clippy 判为常量断言；
        // 放进 const 块后约束提升为编译期检查，违规直接编译失败。
        // const 上下文不支持格式化参数，故消息使用静态字符串。
        const {
            assert!(
                REFS_PAGE_LIMIT <= 100,
                "refs first 上限为 100，请调低 REFS_PAGE_LIMIT"
            );
        }
        let q = build_query(&[("owner".into(), "repo".into())]);
        assert!(
            q.contains(&format!("refs(first: {REFS_PAGE_LIMIT},")),
            "查询应使用受限的 refs first 限制，实际: {q}"
        );
    }

    /// 常规断言：alias 编号递增、owner/repo 正确转义
    #[test]
    fn build_query_generates_numbered_aliases() {
        let q = build_query(&[("o1".into(), "r1".into()), ("o2".into(), "r2".into())]);
        assert!(q.starts_with("query { r0: repository(owner: \"o1\", name: \"r1\""));
        assert!(q.contains("r1: repository(owner: \"o2\", name: \"r2\""));
    }
}
