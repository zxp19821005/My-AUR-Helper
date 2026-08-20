/**
 * release.rs - GitHub Release API 版本检查逻辑（latest 路径）
 *
 * 功能：通过 GitHub Release API 获取最新版本号。
 * 支持两种模式：
 * 1. check_github_release_latest: 直接获取 latest release，性能最优
 * 2. check_github_releases: 遍历所有 releases（见同目录 release_history.rs）
 */
use log::info;
use reqwest::Client;

use crate::checkers::github::binary_check::{
    check_release_assets, extract_version_from_assets, has_linux_binary,
};
use crate::checkers::github::release_history::check_github_releases;
use crate::checkers::utils::{clean_version, extract_version_with_regex};
use crate::errors::AppResult;

/// 构建带认证头的 GitHub API 请求
///
/// 代理在客户端级别配置，调用方传入的 client 已根据域名决定是否启用代理。
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
                        client,
                        owner,
                        repo,
                        token,
                        version_extract_regex,
                        true,
                        true,
                        pkgname,
                    )
                    .await;
                }
            }
        }
    }

    if let Some(tag) = data["tag_name"].as_str() {
        if check_binary_files && version_extract_regex.is_some() {
            // 启用二进制检查且设置了正则时，优先从匹配的资产文件名提取版本
            // （例如 tag 为 "latest"，真实版本号包含在 .pacman 文件名中）
            if let Some(filter) = version_extract_regex {
                if let Some(assets) = data["assets"].as_array() {
                    if let Some(version) = extract_version_from_assets(assets, filter) {
                        return Ok(Some(version));
                    }
                }
            }
            // 资产文件名中未能提取版本时，回退到 tag
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
                client,
                owner,
                repo,
                token,
                version_extract_regex,
                true,
                false,
                pkgname,
            )
            .await;
        }

        return Ok(Some(clean_version(tag)));
    }
    Ok(None)
}