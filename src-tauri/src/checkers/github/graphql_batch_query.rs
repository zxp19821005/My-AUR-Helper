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
              refs(first: 500, refPrefix: \"refs/tags/\", orderBy: {{ field: CREATED_AT, direction: DESC }}) {{ nodes {{ name pushedAt }} }} \
              releases(first: 50, orderBy: {{ field: CREATED_AT, direction: DESC }}) {{ \
                nodes {{ tagName name isPrerelease isDraft createdAt \
                  releaseAssets(first: 12) {{ nodes {{ name }} }} }} }} \
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
