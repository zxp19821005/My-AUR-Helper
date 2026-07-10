/**
 * tags.rs - GitHub Tags 版本检查逻辑
 *
 * 功能：通过 GitHub Tags API 分页获取仓库的所有 tags，并提取最新版本号。
 * 支持版本提取正则表达式，可以过滤特定格式的 tag。
 * 支持跳过测试版本（prerelease），只返回稳定版本。
 *
 * 工作流程：
 * 1. 分页请求 GitHub Tags API（每页 100 条）
 * 2. 收集所有 tags 到列表
 * 3. 对每个 tag 应用版本提取正则或默认清理逻辑
 * 4. 使用 vercmp 算法比较版本，保留最新版本
 * 5. 返回找到的最新版本
 */
use reqwest::Client;

use crate::checkers::utils::{clean_version, extract_version_with_regex};
use crate::errors::AppResult;
use crate::versions;

/// 通过 GitHub Tags API 获取最新版本
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `token`: GitHub API Token（可选）
/// - `version_extract_regex`: 版本提取正则表达式（可选）
/// - `check_test_versions`: 是否包含测试版本（prerelease）
///
/// # 返回
/// - `Ok(Some(version))`: 找到的最新版本
/// - `Ok(None)`: 未找到任何有效版本
/// - `Err(e)`: 请求失败
pub async fn check_github_tags(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    version_extract_regex: Option<&str>,
    check_test_versions: bool,
) -> AppResult<Option<String>> {
    let mut page = 1;
    let mut all_tags = Vec::new();

    // 分页获取所有 tags
    loop {
        let tags_url = format!(
            "https://api.github.com/repos/{}/{}/tags?per_page=100&page={}",
            owner, repo, page
        );

        let mut req = client
            .get(&tags_url)
            .header("User-Agent", "my-aur-helper/0.1")
            .header("Accept", "application/vnd.github.v3+json");
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            break;
        }

        let tags: Vec<serde_json::Value> = resp.json().await?;
        if tags.is_empty() {
            break;
        }

        for tag in &tags {
            if let Some(name) = tag["name"].as_str() {
                all_tags.push(name.to_string());
            }
        }

        if tags.len() < 100 {
            break;
        }
        page += 1;
    }

    if all_tags.is_empty() {
        return Ok(None);
    }

    // 遍历所有 tags，提取并比较版本
    let mut best_version: Option<String> = None;

    for tag in &all_tags {
        let version = if let Some(regex) = version_extract_regex {
            extract_version_with_regex(tag, regex)
        } else {
            Some(clean_version(tag))
        };

        if let Some(version) = version {
            // 如果不检查测试版本，跳过 prerelease
            if !check_test_versions && versions::is_prerelease(&version) {
                continue;
            }

            // 使用 vercmp 算法比较版本
            best_version = match best_version.take() {
                Some(current)
                    if versions::compare_versions(&current, &version)
                        == versions::VersionComparison::LessThan =>
                {
                    Some(version)
                }
                Some(current) => Some(current),
                None => Some(version),
            };
        }
    }

    Ok(best_version)
}
