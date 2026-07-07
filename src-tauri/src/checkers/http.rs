use crate::errors::AppResult;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use super::trait_def::VersionChecker;
use super::utils::{extract_version_from_html, extract_version_with_regex};

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
    /// @param version_extract_regex - 可选的版本提取正则表达式
    /// @returns 从页面内容中提取的版本号
    async fn check(&self, client: &Client, upstream_url: &str, pkgname: &str, version_extract_regex: Option<&str>) -> AppResult<Option<String>> {
        info!("[版本检查] 开始检查软件包: {} (检查器: {})", pkgname, self.name());
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);
        
        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(None);
        }
        
        debug!("[HTTP 请求] 请求URL: {}", upstream_url);
        debug!("[HTTP 请求] 请求方法: GET");
        debug!("[HTTP 请求] 请求头: User-Agent=my-aur-helper/0.1");
        
        let resp = client.get(upstream_url).header("User-Agent", "my-aur-helper/0.1").send().await?;
        let status = resp.status();
        debug!("[HTTP 请求] 响应状态码: {}", status);
        
        if !status.is_success() {
            debug!("[HTTP 请求] 请求失败，状态码: {}", status);
            if let Ok(body) = resp.text().await {
                debug!("[HTTP 请求] 错误响应内容: {}", body);
            }
            return Ok(None);
        }
        
        let body = resp.text().await?;
        debug!("[HTTP 请求] 响应内容长度: {} 字符", body.len());
        debug!("[HTTP 请求] 响应内容前500字符: {}", &body.chars().take(500).collect::<String>());
        
        let result = if let Some(regex) = version_extract_regex {
            debug!("[HTML 解析] 使用自定义正则提取版本号");
            match extract_version_with_regex(&body, regex) {
                Some(ver) => {
                    debug!("[HTML 解析] 使用自定义正则提取到版本号: {}", ver);
                    Ok(Some(ver))
                }
                None => {
                    debug!("[HTML 解析] 自定义正则匹配失败，尝试默认解析");
                    match extract_version_from_html(&body) {
                        Some(ver) => {
                            debug!("[HTML 解析] 使用默认规则提取到版本号: {}", ver);
                            Ok(Some(ver))
                        }
                        None => {
                            debug!("[HTML 解析] 无法从页面中提取版本号");
                            Ok(None)
                        }
                    }
                }
            }
        } else if let Some(ver) = extract_version_from_html(&body) {
            debug!("[HTML 解析] 从页面中提取到版本号: {}", ver);
            Ok(Some(ver))
        } else {
            debug!("[HTML 解析] 无法从页面中提取版本号");
            Ok(None)
        };
        
        if let Ok(Some(version)) = &result {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, version);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        result
    }
}
