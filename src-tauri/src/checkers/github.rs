use anyhow::Result;              // 通用错误处理
use async_trait::async_trait;   // 异步 trait 支持
use reqwest::Client;            // HTTP 客户端

use super::trait_def::VersionChecker; // 版本检查器 trait
use super::utils::{clean_version, extract_owner_repo}; // 工具函数：清理版本号、提取 owner/repo

/// GitHub Release 检查器
/// 通过 GitHub API 获取最新 release 的 tag_name
pub struct GitHubReleaseChecker;

#[async_trait]
impl VersionChecker for GitHubReleaseChecker {
    fn name(&self) -> &'static str {
        "github_release"
    }

    /// 检查 GitHub 最新 Release 版本
    /// @param upstream_url - GitHub 仓库 URL
    /// @returns 清理后的版本号字符串
    async fn check(&self, client: &Client, upstream_url: &str, _pkgname: &str) -> Result<Option<String>> {
        if upstream_url.is_empty() {
            return Ok(None);
        }
        let (owner, repo) = match extract_owner_repo(upstream_url) {
            Some(pair) => pair,
            None => return Ok(None),
        };
        check_github_release(client, &owner, &repo, None).await
    }
}

/// GitHub Tag 检查器
/// 通过 GitHub API 获取最新的 tag 名称
pub struct GitHubTagChecker;

#[async_trait]
impl VersionChecker for GitHubTagChecker {
    fn name(&self) -> &'static str {
        "github_tag"
    }

    /// 检查 GitHub 最新 Tag 版本
    /// @param upstream_url - GitHub 仓库 URL
    /// @returns 清理后的版本号字符串
    async fn check(&self, client: &Client, upstream_url: &str, _pkgname: &str) -> Result<Option<String>> {
        if upstream_url.is_empty() {
            return Ok(None);
        }
        let (owner, repo) = match extract_owner_repo(upstream_url) {
            Some(pair) => pair,
            None => return Ok(None),
        };
        let token = std::env::var("GITHUB_TOKEN").ok(); // 尝试读取环境变量中的 GitHub Token（提高 API 限额）
        check_github_tag(client, &owner, &repo, token.as_deref()).await
    }
}

/// 检查 GitHub Release 版本
/// 调用 GitHub API 获取最新 release 信息
/// @param client - HTTP 客户端
/// @param owner - 仓库所有者
/// @param repo - 仓库名
/// @param token - 可选的 GitHub Token（用于认证）
/// @returns 最新 release 的 tag_name（清理后）
pub async fn check_github_release(client: &Client, owner: &str, repo: &str, token: Option<&str>) -> Result<Option<String>> {
    let api_url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);
    let mut req = client
        .get(&api_url)
        .header("User-Agent", "my-aur-helper/0.1")       // GitHub API 需要 User-Agent
        .header("Accept", "application/vnd.github.v3+json"); // 指定 API 版本
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t)); // 添加认证头
    }
    let resp = req.send().await?;
    if resp.status().is_success() {
        let data: serde_json::Value = resp.json().await?;
        if let Some(tag) = data["tag_name"].as_str() {
            return Ok(Some(clean_version(tag))); // 提取并清理版本号
        }
    }
    Ok(None)
}

/// 检查 GitHub Tag 版本
/// 调用 GitHub API 获取仓库的 tag 列表（仅取第一个）
/// @param client - HTTP 客户端
/// @param owner - 仓库所有者
/// @param repo - 仓库名
/// @param token - 可选的 GitHub Token（用于认证）
/// @returns 最新 tag 的名称（清理后）
pub async fn check_github_tag(client: &Client, owner: &str, repo: &str, token: Option<&str>) -> Result<Option<String>> {
    let tags_url = format!("https://api.github.com/repos/{}/{}/tags?per_page=1", owner, repo);
    let mut req = client
        .get(&tags_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }
    let resp = req.send().await?;
    if resp.status().is_success() {
        if let Ok(tags) = resp.json::<Vec<serde_json::Value>>().await {
            if let Some(tag) = tags.first() {
                if let Some(name) = tag["name"].as_str() {
                    return Ok(Some(clean_version(name)));
                }
            }
        }
    }
    Ok(None)
}
