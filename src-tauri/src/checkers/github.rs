use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use super::trait_def::VersionChecker;
use super::utils::{clean_version, extract_owner_repo, extract_version_with_regex};
use crate::errors::AppResult;

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
    /// @param version_extract_regex - 可选的版本提取正则表达式
    /// @returns 清理后的版本号字符串
    async fn check(&self, client: &Client, upstream_url: &str, pkgname: &str, version_extract_regex: Option<&str>) -> AppResult<Option<String>> {
        info!("[版本检查] 开始检查软件包: {} (检查器: {})", pkgname, self.name());
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);
        
        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(None);
        }
        let (owner, repo) = match extract_owner_repo(upstream_url) {
            Some(pair) => pair,
            None => {
                debug!("[版本检查] 无法解析 GitHub URL: {}", upstream_url);
                return Ok(None);
            }
        };
        debug!("[版本检查] 解析到仓库: owner={}, repo={}", owner, repo);
        
        let result = check_github_release(client, &owner, &repo, None, version_extract_regex).await;
        if let Ok(Some(version)) = &result {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, version);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        result
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
    /// @param version_extract_regex - 可选的版本提取正则表达式
    /// @returns 清理后的版本号字符串
    async fn check(&self, client: &Client, upstream_url: &str, pkgname: &str, version_extract_regex: Option<&str>) -> AppResult<Option<String>> {
        info!("[版本检查] 开始检查软件包: {} (检查器: {})", pkgname, self.name());
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);
        
        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(None);
        }
        let (owner, repo) = match extract_owner_repo(upstream_url) {
            Some(pair) => pair,
            None => {
                debug!("[版本检查] 无法解析 GitHub URL: {}", upstream_url);
                return Ok(None);
            }
        };
        debug!("[版本检查] 解析到仓库: owner={}, repo={}", owner, repo);
        
        let token = std::env::var("GITHUB_TOKEN").ok();
        debug!("[版本检查] GitHub Token: {}", if token.is_some() { "已配置" } else { "未配置" });
        
        let result = check_github_tag(client, &owner, &repo, token.as_deref(), version_extract_regex).await;
        if let Ok(Some(version)) = &result {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, version);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        result
    }
}

/// 检查 GitHub Release 版本
/// 调用 GitHub API 获取最新 release 信息
/// @param client - HTTP 客户端
/// @param owner - 仓库所有者
/// @param repo - 仓库名
/// @param token - 可选的 GitHub Token（用于认证）
/// @param version_extract_regex - 可选的版本提取正则表达式
/// @returns 最新 release 的 tag_name（清理后）
pub async fn check_github_release(client: &Client, owner: &str, repo: &str, token: Option<&str>, version_extract_regex: Option<&str>) -> AppResult<Option<String>> {
    let api_url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);
    
    debug!("[GitHub API] 请求URL: {}", api_url);
    debug!("[GitHub API] 请求方法: GET");
    debug!("[GitHub API] 请求头: User-Agent=my-aur-helper/0.1, Accept=application/vnd.github.v3+json");
    debug!("[GitHub API] 是否使用认证: {}", token.is_some());
    debug!("[GitHub API] 版本提取正则: {:?}", version_extract_regex);
    
    let mut req = client
        .get(&api_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
        debug!("[GitHub API] 认证方式: Bearer Token");
    }
    
    let resp = req.send().await?;
    let status = resp.status();
    debug!("[GitHub API] 响应状态码: {}", status);
    
    if status.is_success() {
        let data: serde_json::Value = resp.json().await?;
        let response_text = serde_json::to_string_pretty(&data).unwrap_or_default();
        debug!("[GitHub API] 原始响应数据: {}", response_text);
        
        if let Some(tag) = data["tag_name"].as_str() {
            let version = if let Some(regex) = version_extract_regex {
                debug!("[GitHub API] 使用自定义正则提取版本号");
                match extract_version_with_regex(tag, regex) {
                    Some(v) => v,
                    None => {
                        debug!("[GitHub API] 自定义正则匹配失败，使用默认清理");
                        clean_version(tag)
                    }
                }
            } else {
                clean_version(tag)
            };
            debug!("[GitHub API] 提取到版本号: 原始={}, 处理后={}", tag, version);
            return Ok(Some(version));
        } else {
            debug!("[GitHub API] 响应中未找到 tag_name 字段");
        }
    } else {
        debug!("[GitHub API] 请求失败，状态码: {}", status);
        if let Ok(body) = resp.text().await {
            debug!("[GitHub API] 错误响应内容: {}", body);
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
/// @param version_extract_regex - 可选的版本提取正则表达式
/// @returns 最新 tag 的名称（清理后）
pub async fn check_github_tag(client: &Client, owner: &str, repo: &str, token: Option<&str>, version_extract_regex: Option<&str>) -> AppResult<Option<String>> {
    let tags_url = format!("https://api.github.com/repos/{}/{}/tags?per_page=1", owner, repo);
    
    debug!("[GitHub API] 请求URL: {}", tags_url);
    debug!("[GitHub API] 请求方法: GET");
    debug!("[GitHub API] 请求参数: per_page=1");
    debug!("[GitHub API] 请求头: User-Agent=my-aur-helper/0.1, Accept=application/vnd.github.v3+json");
    debug!("[GitHub API] 是否使用认证: {}", token.is_some());
    debug!("[GitHub API] 版本提取正则: {:?}", version_extract_regex);
    
    let mut req = client
        .get(&tags_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
        debug!("[GitHub API] 认证方式: Bearer Token");
    }
    
    let resp = req.send().await?;
    let status = resp.status();
    debug!("[GitHub API] 响应状态码: {}", status);
    
    if status.is_success() {
        if let Ok(tags) = resp.json::<Vec<serde_json::Value>>().await {
            debug!("[GitHub API] 原始响应数据: {}", serde_json::to_string_pretty(&tags).unwrap_or_default());
            
            if let Some(tag) = tags.first() {
                if let Some(name) = tag["name"].as_str() {
                    let version = if let Some(regex) = version_extract_regex {
                        debug!("[GitHub API] 使用自定义正则提取版本号");
                        match extract_version_with_regex(name, regex) {
                            Some(v) => v,
                            None => {
                                debug!("[GitHub API] 自定义正则匹配失败，使用默认清理");
                                clean_version(name)
                            }
                        }
                    } else {
                        clean_version(name)
                    };
                    debug!("[GitHub API] 提取到版本号: 原始={}, 处理后={}", name, version);
                    return Ok(Some(version));
                } else {
                    debug!("[GitHub API] 响应中未找到 name 字段");
                }
            } else {
                debug!("[GitHub API] 响应为空列表，未找到任何 tag");
            }
        } else {
            debug!("[GitHub API] 无法解析响应为 JSON 数组");
        }
    } else {
        debug!("[GitHub API] 请求失败，状态码: {}", status);
        if let Ok(body) = resp.text().await {
            debug!("[GitHub API] 错误响应内容: {}", body);
        }
    }
    Ok(None)
}
