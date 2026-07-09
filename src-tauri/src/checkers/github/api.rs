use log::{debug, info, warn};
use reqwest::Client;

use crate::checkers::utils::{clean_version, extract_version_with_regex};
use crate::errors::AppResult;
use crate::versions;

/// 检查 release 是否有 Linux 二进制文件
/// 如果提供了 asset_filter，则用其匹配 asset 名称
fn has_linux_binary(assets: &[serde_json::Value], asset_filter: Option<&str>) -> bool {
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

            best_version = match best_version.take() {
                Some(current) if versions::compare_versions(&current, &version) == versions::VersionComparison::LessThan => Some(version),
                Some(current) => Some(current),
                None => Some(version),
            };
        }
    }

    Ok(best_version)
}