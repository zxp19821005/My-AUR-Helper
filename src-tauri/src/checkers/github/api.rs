/**
 * api.rs - GitHub Release API 版本检查逻辑
 *
 * 功能：通过 GitHub Release API 获取最新版本号。
 * 支持两种模式：
 * 1. check_github_release_latest: 直接获取 latest release，性能最优
 * 2. check_github_releases: 遍历最近 10 个 releases，用于测试版本检查或资产过滤
 *
 * 二进制文件检查：
 * - 当 check_binary_files 启用时，会检查 release 的 assets 是否包含 Linux 二进制文件
 * - 如果提供了 version_extract_regex，会将其用作资产文件名过滤器
 * - 如果最新版本没有匹配的资产文件，自动回退查找历史版本
 *
 * 资产过滤逻辑：
 * - 默认：排除包含 darwin/macos/windows 的资产，其余视为 Linux 文件
 * - 自定义：使用 version_extract_regex 正则表达式匹配资产文件名
 *   例如：填写 "_amd64.deb" 可以只匹配包含该字符串的资产文件
 */
use log::{debug, info, warn};
use reqwest::Client;

use crate::checkers::utils::{clean_version, extract_version_with_regex};
use crate::errors::AppResult;
use crate::versions;

/// 检查 release 的 assets 是否包含 Linux 二进制文件
///
/// # 参数
/// - `assets`: release 的资产文件列表
/// - `asset_filter`: 资产文件名过滤器（可选），使用正则表达式匹配
///
/// # 返回
/// - `true`: 存在匹配的 Linux 二进制文件
/// - `false`: 不存在匹配的 Linux 二进制文件
fn has_linux_binary(assets: &[serde_json::Value], asset_filter: Option<&str>) -> bool {
    // 判断文件名是否明显是非 Linux 平台
    let not_linux = |name: &str| {
        let lower = name.to_lowercase();
        lower.contains("darwin") || lower.contains("macos") || lower.contains("windows")
    };
    
    if let Some(filter) = asset_filter {
        // 使用自定义正则过滤 assets
        if let Ok(re) = regex::Regex::new(filter) {
            return assets.iter().any(|a| {
                if let Some(name) = a["name"].as_str() {
                    !not_linux(name) && re.is_match(name)
                } else {
                    false
                }
            });
        }
    }
    
    // 默认：只要不是 darwin/macos/windows 就算 Linux
    assets.iter().any(|a| {
        a["name"].as_str().is_some_and(|n| !not_linux(n))
    })
}

/// 检查并打印 release 资产的详细信息
///
/// # 参数
/// - `data`: release 的 JSON 数据
/// - `pkgname`: 软件包名称（用于日志）
/// - `asset_filter`: 资产文件名过滤器（可选）
fn check_release_assets(data: &serde_json::Value, pkgname: &str, asset_filter: Option<&str>) {
    let assets = data["assets"].as_array();
    if let Some(list) = assets {
        if list.is_empty() {
            warn!("[二进制检查] {}: Release 无任何附件", pkgname);
        } else if !has_linux_binary(list, asset_filter) {
            let names: Vec<&str> = list.iter().filter_map(|a| a["name"].as_str()).collect();
            warn!("[二进制检查] {}: Release 附件中未找到 Linux 二进制文件: {:?}", pkgname, names);
        } else {
            let linux_assets: Vec<&str> = list.iter().filter_map(|a| {
                let name = a["name"].as_str()?;
                let lower = name.to_lowercase();
                if !lower.contains("darwin") && !lower.contains("macos") && !lower.contains("windows") {
                    Some(name)
                } else {
                    None
                }
            }).collect();
            info!("[二进制检查] {}: 找到 Linux 二进制文件: {:?}", pkgname, linux_assets);
        }
    }
}

/// 获取 GitHub 仓库的 latest release 并提取版本号
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `token`: GitHub API Token（可选）
/// - `version_extract_regex`: 版本提取正则表达式（可选）
///   - 当 check_binary_files 启用时，此参数用作资产文件名过滤器
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
    let api_url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);

    let mut req = client
        .get(&api_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }

    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }

    let data: serde_json::Value = resp.json().await?;

    if check_binary_files {
        // 当启用二进制检查时，version_extract_regex 用作 asset 过滤器
        check_release_assets(&data, pkgname, version_extract_regex);
        
        // 如果有 asset 过滤器，检查最新版本是否匹配
        if let Some(filter) = version_extract_regex {
            if let Some(assets) = data["assets"].as_array() {
                let has_match = has_linux_binary(assets, Some(filter));
                if !has_match {
                    info!("[二进制检查] {}: 最新版本无匹配的资产文件，尝试查找历史版本", pkgname);
                    // 回退到遍历历史版本
                    return check_github_releases(client, owner, repo, token, version_extract_regex, true, pkgname).await;
                }
            }
        }
    }

    if let Some(tag) = data["tag_name"].as_str() {
        // 当启用二进制检查且有 asset 过滤器时，version_extract_regex 已用于过滤 assets
        // 此时从 tag 提取版本应该使用 clean_version，而不是用 regex 匹配 tag
        let version = if check_binary_files && version_extract_regex.is_some() {
            clean_version(tag)
        } else if let Some(regex) = version_extract_regex {
            extract_version_with_regex(tag, regex).unwrap_or_else(|| clean_version(tag))
        } else {
            clean_version(tag)
        };
        return Ok(Some(version));
    }
    Ok(None)
}

/// 遍历最近 10 个 releases，提取并比较版本号
///
/// 用于以下场景：
/// - 需要检查测试版本（prerelease）
/// - 需要检查二进制文件，且最新版本没有匹配的资产文件
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `token`: GitHub API Token（可选）
/// - `version_extract_regex`: 版本提取正则表达式（可选）
///   - 当 check_binary_files 启用时，此参数用作资产文件名过滤器
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
    check_binary_files: bool,
    pkgname: &str,
) -> AppResult<Option<String>> {
    let api_url = format!("https://api.github.com/repos/{}/{}/releases?per_page=10", owner, repo);

    let mut req = client
        .get(&api_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }

    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }

    let releases: Vec<serde_json::Value> = resp.json().await?;
    let mut best_version: Option<String> = None;

    for release in &releases {
        if let Some(tag) = release["tag_name"].as_str() {
            // 跳过 prerelease（除非调用方明确需要）
            if release["prerelease"].as_bool().unwrap_or(false) {
                continue;
            }

            if check_binary_files {
                let assets = release["assets"].as_array();
                if let Some(list) = assets {
                    // 当启用二进制检查时，version_extract_regex 用作 asset 过滤器
                    if !has_linux_binary(list, version_extract_regex) {
                        debug!("[二进制检查] {}: Release {} 无匹配的资产文件，跳过", pkgname, tag);
                        continue;
                    }
                }
            }

            let version = if let Some(regex) = version_extract_regex {
                extract_version_with_regex(tag, regex).unwrap_or_else(|| clean_version(tag))
            } else {
                clean_version(tag)
            };

            // 使用 vercmp 算法比较版本
            best_version = match best_version.take() {
                Some(current) if versions::compare_versions(&current, &version) == versions::VersionComparison::LessThan => Some(version),
                Some(current) => Some(current),
                None => Some(version),
            };
        }
    }

    Ok(best_version)
}