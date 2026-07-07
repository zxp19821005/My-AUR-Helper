use crate::errors::AppResult;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use super::trait_def::VersionChecker;
use super::utils::{extract_version_from_url, extract_version_with_regex};

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
    /// @param version_extract_regex - 可选的版本提取正则表达式
    /// @returns 从重定向目标 URL 中提取的版本号
    async fn check(&self, client: &Client, upstream_url: &str, pkgname: &str, version_extract_regex: Option<&str>) -> AppResult<Option<String>> {
        info!("[版本检查] 开始检查软件包: {} (检查器: {})", pkgname, self.name());
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);
        
        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(None);
        }
        
        debug!("[HTTP 请求] 请求URL: {}", upstream_url);
        debug!("[HTTP 请求] 请求方法: HEAD");
        
        let resp = client.head(upstream_url).send().await?;
        let status = resp.status();
        debug!("[HTTP 请求] 响应状态码: {}", status);
        
        let result = if let Some(location) = resp.headers().get("location") {
            let location_str = location.to_str().unwrap_or("");
            debug!("[HTTP 重定向] Location 响应头: {}", location_str);
            
            if let Some(regex) = version_extract_regex {
                debug!("[URL 解析] 使用自定义正则提取版本号");
                match extract_version_with_regex(location_str, regex) {
                    Some(ver) => {
                        debug!("[URL 解析] 使用自定义正则提取到版本号: {}", ver);
                        Ok(Some(ver))
                    }
                    None => {
                        debug!("[URL 解析] 自定义正则匹配失败，尝试默认解析");
                        match extract_version_from_url(location_str) {
                            Some(ver) => {
                                debug!("[URL 解析] 从重定向 URL 中提取到版本号: {}", ver);
                                Ok(Some(ver))
                            }
                            None => {
                                debug!("[URL 解析] 无法从重定向 URL 中提取版本号");
                                Ok(None)
                            }
                        }
                    }
                }
            } else if let Some(ver) = extract_version_from_url(location_str) {
                debug!("[URL 解析] 从重定向 URL 中提取到版本号: {}", ver);
                Ok(Some(ver))
            } else {
                debug!("[URL 解析] 无法从重定向 URL 中提取版本号");
                Ok(None)
            }
        } else {
            debug!("[HTTP 重定向] 响应中未找到 Location 头");
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
