use anyhow::Result;            // 通用错误处理
use async_trait::async_trait; // 异步 trait 支持
use reqwest::Client;          // HTTP 客户端

use super::trait_def::VersionChecker;       // 版本检查器 trait
use super::utils::extract_version_from_html; // 从 HTML 提取版本号工具

/// HTTP 页面解析检查器
/// 通过下载 HTML 页面内容并从中提取版本号
/// 适用于没有标准 API 的网站
pub struct HttpChecker;

#[async_trait]
impl VersionChecker for HttpChecker {
    fn name(&self) -> &'static str {
        "http"
    }

    /// 检查 HTTP 页面的版本信息
    /// @param upstream_url - 要检查的网页 URL
    /// @returns 从页面内容中提取的版本号
    async fn check(&self, client: &Client, upstream_url: &str, _pkgname: &str) -> Result<Option<String>> {
        if upstream_url.is_empty() {
            return Ok(None);
        }
        // 发送 GET 请求获取页面内容
        let resp = client.get(upstream_url).header("User-Agent", "my-aur-helper/0.1").send().await?;
        if !resp.status().is_success() {
            return Ok(None);
        }
        let body = resp.text().await?;
        // 尝试从 HTML 中匹配版本号模式
        if let Some(ver) = extract_version_from_html(&body) {
            return Ok(Some(ver));
        }
        Ok(None)
    }
}
