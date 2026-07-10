/**
 * release.rs - GitHub Release API 版本检查逻辑
 *
 * 功能：通过 GitHub Release API 获取最新版本号。
 * 支持两种模式：
 * 1. check_github_release_latest: 直接获取 latest release，性能最优
 * 2. check_github_releases: 遍历所有 releases，用于测试版本检查或资产过滤
 */
use log::{debug, info, warn};
use reqwest::Client;

use crate::checkers::github::binary_check::{check_release_assets, has_linux_binary};
use crate::checkers::utils::{clean_version, extract_version_with_regex};
use crate::errors::AppResult;
use crate::versions;

/// 构建带认证头的 GitHub API 请求
pub(crate) fn build_github_request<'a>(
    client: &'a Client,
    url: &str,
    token: Option<&'a str>,
) -> reqwest::RequestBuilder {
    let mut req = client
        .get(url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }
    req
}

/// 获取 GitHub 仓库的 latest release 并提取版本号
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `token`: GitHub API Token（可选）
/// - `version_extract_regex`: 版本提取正则表达式（可选）
/// - `check_binary_files`: 是否检查二进制文件
/// - `pkgname`: 软件包名称（用于日志）
///
/// # 返回
/// - `Ok(Some(version))`: 找到的最新版本
/// - `Ok(None)`: 未找到 latest release
/// - `Err(e)`: 请求失败
pub async fn check_github_release_latest(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    version_extract_regex: Option<&str>,
    check_binary_files: bool,
    pkgname: &str,
) -> AppResult<Option<String>> {
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    let resp = build_github_request(client, &api_url, token).send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }

    let data: serde_json::Value = resp.json().await?;

    if check_binary_files {
        check_release_assets(&data, pkgname, version_extract_regex);

        if let Some(filter) = version_extract_regex {
            if let Some(assets) = data["assets"].as_array() {
                if !has_linux_binary(assets, Some(filter)) {
                    info!(
                        "[二进制检查] {}: 最新版本无匹配的资产文件，尝试查找历史版本",
                        pkgname
                    );
                    return check_github_releases(
                        client, owner, repo, token, version_extract_regex,
                        true, true, pkgname,
                    ).await;
                }
            }
        }
    }

    if let Some(tag) = data["tag_name"].as_str() {
        if check_binary_files && version_extract_regex.is_some() {
            return Ok(Some(clean_version(tag)));
        }

        if let Some(regex) = version_extract_regex {
            if let Some(version) = extract_version_with_regex(tag, regex).or_else(|| {
                let name = data["name"].as_str().unwrap_or("");
                extract_version_with_regex(name, regex)
            }) {
                return Ok(Some(version));
            }

            info!(
                "[二进制检查] {}: latest release tag '{}' 不匹配正则 '{}'，尝试查找 prerelease",
                pkgname, tag, regex
            );
            return check_github_releases(
                client, owner, repo, token, version_extract_regex,
                true, false, pkgname,
            ).await;
        }

        return Ok(Some(clean_version(tag)));
    }
    Ok(None)
}

/// 遍历 releases，提取并比较版本号（支持分页）
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `token`: GitHub API Token（可选）
/// - `version_extract_regex`: 版本提取正则表达式（可选）
/// - `check_test_versions`: 是否包含测试版本（prerelease）
/// - `check_binary_files`: 是否检查二进制文件
/// - `pkgname`: 软件包名称（用于日志）
///
/// # 返回
/// - `Ok(Some(version))`: 找到的最新版本
/// - `Ok(None)`: 未找到任何有效 release
/// - `Err(e)`: 请求失败
pub async fn check_github_releases(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    version_extract_regex: Option<&str>,
    check_test_versions: bool,
    check_binary_files: bool,
    pkgname: &str,
) -> AppResult<Option<String>> {
    let mut best_version: Option<String> = None;
    let mut page = 1;
    let per_page = 100;
    let max_pages = 5;

    let tag_filter = if let Some(regex) = version_extract_regex {
        regex::Regex::new(regex).ok()
    } else {
        None
    };

    loop {
        if page > max_pages {
            debug!(
                "[二进制检查] {}: 已达到最大页数限制 ({} 页，{} 个 releases)，停止搜索",
                pkgname, max_pages, max_pages * per_page
            );
            break;
        }

        let api_url = format!(
            "https://api.github.com/repos/{}/{}/releases?per_page={}&page={}",
            owner, repo, per_page, page
        );

        let resp = build_github_request(client, &api_url, token).send().await?;

        if resp.status().as_u16() == 403 {
            warn!("[二进制检查] {}: 触发 GitHub API 限流，停止搜索", pkgname);
            break;
        }

        if !resp.status().is_success() {
            return Ok(None);
        }

        let releases: Vec<serde_json::Value> = resp.json().await?;

        if releases.is_empty() {
            debug!(
                "[二进制检查] {}: 第 {} 页无更多 releases，停止搜索",
                pkgname, page
            );
            break;
        }

        debug!(
            "[二进制检查] {}: 正在检查第 {} 页 ({} 个 releases)",
            pkgname, page, releases.len()
        );

        for release in &releases {
            if let Some(tag) = release["tag_name"].as_str() {
                if !check_test_versions && release["prerelease"].as_bool().unwrap_or(false) {
                    debug!("[二进制检查] {}: Release {} 是 prerelease，跳过", pkgname, tag);
                    continue;
                }

                if let Some(ref re) = tag_filter {
                    let release_name = release["name"].as_str().unwrap_or(tag);
                    if !re.is_match(tag) && !re.is_match(release_name) {
                        debug!(
                            "[二进制检查] {}: Release {} ({}) 不匹配正则 {}，跳过",
                            pkgname, tag, release_name,
                            version_extract_regex.unwrap_or("")
                        );
                        continue;
                    }
                }

                if check_binary_files {
                    if let Some(assets) = release["assets"].as_array() {
                        if !has_linux_binary(assets, version_extract_regex) {
                            debug!("[二进制检查] {}: Release {} 无匹配的资产文件，跳过", pkgname, tag);
                            continue;
                        }
                    }
                }

                let version = clean_version(tag);
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

        page += 1;
    }

    Ok(best_version)
}