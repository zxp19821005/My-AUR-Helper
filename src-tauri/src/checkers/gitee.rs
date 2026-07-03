use anyhow::Result;            // 通用错误处理
use async_trait::async_trait; // 异步 trait 支持
use reqwest::Client;          // HTTP 客户端

use super::trait_def::VersionChecker; // 版本检查器 trait
use super::utils::clean_version;     // 版本号清理工具

/// Gitee（码云）检查器
/// 通过 Gitee API v5 获取最新 release 的 tag_name
pub struct GiteeChecker;

#[async_trait]
impl VersionChecker for GiteeChecker {
    fn name(&self) -> &'static str {
        "gitee"
    }

    /// 检查 Gitee 最新 Release 版本
    /// @param upstream_url - Gitee 仓库 URL，例如 https://gitee.com/owner/repo
    /// @returns 清理后的版本号字符串
    async fn check(&self, client: &Client, upstream_url: &str, _pkgname: &str) -> Result<Option<String>> {
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
        // 调用 Gitee API v5 获取最新 release
        let api_url = format!("https://gitee.com/api/v5/repos/{}/{}/releases/latest", owner, repo);
        let resp = client
            .get(&api_url)
            .header("User-Agent", "my-aur-helper/0.1")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Ok(None); // API 请求失败（如仓库不存在或无 release）
        }
        let data: serde_json::Value = resp.json().await?;
        if let Some(tag) = data["tag_name"].as_str() {
            Ok(Some(clean_version(tag)))
        } else {
            Ok(None)
        }
    }
}
