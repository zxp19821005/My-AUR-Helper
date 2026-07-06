use crate::errors::AppResult;
use async_trait::async_trait; // 异步 trait 支持
use reqwest::Client;          // HTTP 客户端

use super::trait_def::VersionChecker; // 版本检查器 trait
use super::utils::clean_version;     // 版本号清理工具

/// GitLab 检查器
/// 通过 GitLab API v4 获取最新 release 的 tag_name
pub struct GitLabChecker;

#[async_trait]
impl VersionChecker for GitLabChecker {
    fn name(&self) -> &'static str {
        "gitlab"
    }

    /// 检查 GitLab 最新 Release 版本
    /// @param upstream_url - GitLab 仓库 URL，例如 https://gitlab.com/owner/repo
    /// @returns 清理后的版本号字符串
    async fn check(&self, client: &Client, upstream_url: &str, _pkgname: &str) -> AppResult<Option<String>> {
        if upstream_url.is_empty() {
            return Ok(None);
        }
        // 从 URL 中提取 owner 和 repo
        let parts: Vec<&str> = upstream_url.trim_end_matches('/').trim_end_matches(".git").split('/').collect();
        if parts.len() < 2 {
            return Ok(None);
        }
        let owner = parts[parts.len() - 2];
        let repo = parts[parts.len() - 1];
        // GitLab API 需要 URL 编码的项目路径
        let project_path = format!("{}%2F{}", owner, repo);
        // 调用 GitLab API v4 获取最新 release
        let api_url = format!("https://gitlab.com/api/v4/projects/{}/releases/permalink/latest", project_path);
        let resp = client
            .get(&api_url)
            .header("User-Agent", "my-aur-helper/0.1")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Ok(None);
        }
        let data: serde_json::Value = resp.json().await?;
        if let Some(tag) = data["tag_name"].as_str() {
            Ok(Some(clean_version(tag)))
        } else {
            Ok(None)
        }
    }
}
