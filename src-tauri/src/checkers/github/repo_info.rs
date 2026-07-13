/**
 * repo_info.rs - GitHub 仓库元信息获取
 *
 * 功能：
 * - 获取 GitHub 仓库的 License 信息
 * - 获取 GitHub 仓库的编程语言列表
 */
use log::{debug, info};
use reqwest::Client;

use crate::checkers::github::release::build_github_request;
use crate::errors::AppResult;

/// 获取 GitHub 仓库的 License 信息
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `token`: GitHub API Token（可选）
///
/// # 返回
/// - `Ok(Some(json_array))`: License 列表的 JSON 数组字符串（如 `["MIT", "Apache-2.0"]`）
/// - `Ok(None)`: 未找到 License
/// - `Err(e)`: 请求失败
pub async fn fetch_github_repo_license(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
) -> AppResult<Option<String>> {
    let api_url = format!("https://api.github.com/repos/{}/{}/license", owner, repo);

    let resp = build_github_request(client, &api_url, token).send().await?;
    if !resp.status().is_success() {
        debug!("[GitHub License] 获取 license 失败: {} {}", owner, repo);
        return Ok(None);
    }

    let data: serde_json::Value = resp.json().await?;

    // GitHub API 返回单个 license 对象，我们将其包装为数组
    if let Some(spdx_id) = data["license"]["spdx_id"].as_str() {
        if spdx_id != "NOASSERTION" && !spdx_id.is_empty() {
            let licenses = vec![spdx_id.to_string()];
            let json_array = serde_json::to_string(&licenses).unwrap_or_default();
            info!("[GitHub License] {} {}: license={}", owner, repo, spdx_id);
            return Ok(Some(json_array));
        }
    }

    debug!("[GitHub License] {} {}: 未找到 license", owner, repo);
    Ok(None)
}

/// 获取 GitHub 仓库的编程语言列表
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `token`: GitHub API Token（可选）
///
/// # 返回
/// - `Ok(Vec<String>)`: 编程语言名称列表（按字节数降序排列）
/// - `Ok(vec![])`: 未找到语言信息
/// - `Err(e)`: 请求失败
pub async fn fetch_github_repo_languages(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
) -> AppResult<Vec<String>> {
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/languages",
        owner, repo
    );

    let resp = build_github_request(client, &api_url, token).send().await?;
    if !resp.status().is_success() {
        debug!("[GitHub Languages] 获取 languages 失败: {} {}", owner, repo);
        return Ok(vec![]);
    }

    let data: serde_json::Value = resp.json().await?;

    if let Some(languages) = data.as_object() {
        let mut lang_list: Vec<(String, u64)> = languages
            .iter()
            .filter_map(|(name, bytes)| bytes.as_u64().map(|b| (name.clone(), b)))
            .collect();

        lang_list.sort_by(|a, b| b.1.cmp(&a.1));

        let lang_names: Vec<String> = lang_list.into_iter().map(|(name, _)| name).collect();
        info!("[GitHub Languages] {} {}: languages={:?}", owner, repo, lang_names);
        Ok(lang_names)
    } else {
        debug!("[GitHub Languages] {} {}: 未找到 language 信息", owner, repo);
        Ok(vec![])
    }
}