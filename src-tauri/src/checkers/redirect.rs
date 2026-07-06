use crate::errors::AppResult;
use async_trait::async_trait;   // 异步 trait 支持
use reqwest::Client;            // HTTP 客户端

use super::trait_def::VersionChecker;       // 版本检查器 trait
use super::utils::extract_version_from_url; // 从 URL 提取版本号工具

/// HTTP 重定向检查器
/// 发送 HEAD 请求，从重定向目标 URL 中提取版本号
/// 适用于下载链接中包含版本号的场景（如 GitHub release 下载）
pub struct RedirectChecker;

#[async_trait]
impl VersionChecker for RedirectChecker {
    fn name(&self) -> &'static str {
        "redirect"
    }

    /// 通过重定向 URL 检查版本
    /// @param upstream_url - 会触发重定向的下载 URL
    /// @returns 从重定向目标 URL 中提取的版本号
    async fn check(&self, client: &Client, upstream_url: &str, _pkgname: &str) -> AppResult<Option<String>> {
        if upstream_url.is_empty() {
            return Ok(None);
        }
        // 发送 HEAD 请求（比 GET 更轻量），不下载实际内容
        let resp = client.head(upstream_url).send().await?;
        // 检查响应头中的 Location（重定向目标）
        if let Some(location) = resp.headers().get("location") {
            let location_str = location.to_str().unwrap_or("");
            // 从重定向 URL 中提取版本号
            if let Some(ver) = extract_version_from_url(location_str) {
                return Ok(Some(ver));
            }
        }
        Ok(None)
    }
}
