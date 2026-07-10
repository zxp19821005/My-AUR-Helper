/**
 * tags.rs - GitHub Tags 版本检查逻辑
 *
 * 功能：通过 GitHub Tags API 分页获取仓库的所有 tags，并提取最新版本号。
 * 支持版本提取正则表达式，可以过滤特定格式的 tag。
 * 支持跳过测试版本（prerelease），只返回稳定版本。
 * 自动跳过不包含数字的 tag（如 "continuous"、"latest"、"stable"）。
 *
 * 工作流程：
 * 1. 分页请求 GitHub Tags API（每页 100 条，最多 2 页/200 个 tags）
 * 2. 收集所有 tags 到列表
 * 3. 对每个 tag 检查是否包含数字，跳过非版本 tag
 * 4. 应用版本提取正则或默认清理逻辑
 * 5. 使用 vercmp 算法比较版本，保留最新版本
 * 6. 返回找到的最新版本
 */
use reqwest::Client;

use crate::checkers::utils::{clean_version, extract_version_with_regex};
use crate::errors::AppResult;
use crate::versions;

/// 检查字符串是否看起来像版本号（至少包含一个数字）
///
/// # 参数
/// - `s`: 要检查的字符串
///
/// # 返回
/// - `true`: 如果字符串包含数字，可能是版本号
/// - `false`: 如果字符串不包含任何数字（如 "continuous"、"latest"、"stable"）
fn looks_like_version(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_digit())
}

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
    let mut all_tags = Vec::new();

    // 只获取前 2 页 tags（最多 200 个），大多数情况第 1 页就足够
    for page in 1..=2 {
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
        let tag_count = tags.len();

        for tag in &tags {
            if let Some(name) = tag["name"].as_str() {
                all_tags.push(name.to_string());
            }
        }

        if tag_count < 100 {
            break;
        }
    }

    if all_tags.is_empty() {
        return Ok(None);
    }

    // 遍历 tags，提取并比较版本
    let mut best_version: Option<String> = None;

    for tag in &all_tags {
        let version = if let Some(regex) = version_extract_regex {
            extract_version_with_regex(tag, regex)
        } else {
            // 跳过不包含数字的 tag（如 "continuous"、"latest"、"stable"）
            if !looks_like_version(tag) {
                continue;
            }
            Some(clean_version(tag))
        };

        if let Some(version) = version {
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
