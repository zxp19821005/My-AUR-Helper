/**
 * redirect.rs - HTTP 重定向版本检查器
 *
 * 功能：通过跟踪 HTTP 重定向获取最终 URL，并从中提取版本号。
 *
 * 工作流程：
 * 1. 发送 HEAD 请求到上游 URL
 * 2. 如果响应包含 Location 头，跟踪重定向
 * 3. 处理相对路径重定向（基于当前 URL 构建完整 URL）
 * 4. 从重定向 URL 中提取版本号
 * 5. 支持多次重定向（最多 5 次）
 * 6. 支持版本提取正则表达式
 *
 * 支持的版本格式：
 * - /v1.2.3 或 /1.2.3
 * - -v1.2.3 或 -1.2.3
 * - 其他常见的版本 URL 格式
 */
use crate::errors::AppResult;
use async_trait::async_trait;
use log::{debug, info, warn};
use reqwest::Client;

use super::trait_def::{CheckOptions, CheckResult, VersionChecker};
use super::utils::{extract_version_from_url, extract_version_with_regex};

/// 最大重定向次数，防止无限循环
const MAX_REDIRECTS: usize = 5;

pub struct RedirectChecker;

#[async_trait]
impl VersionChecker for RedirectChecker {
    fn name(&self) -> &'static str {
        "redirect"
    }

    async fn check(
        &self,
        client: &Client,
        upstream_url: &str,
        pkgname: &str,
        version_extract_regex: Option<&str>,
        _options: &CheckOptions,
    ) -> AppResult<CheckResult> {
        info!(
            "[版本检查] 开始检查软件包: {} (检查器: {})",
            pkgname,
            self.name()
        );
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);

        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(CheckResult::default());
        }

        // 手动跟踪重定向，获取最终的 URL
        let mut current_url = upstream_url.to_string();
        let mut version = None;

        for i in 0..MAX_REDIRECTS {
            debug!("[HTTP 重定向] 第 {} 次请求: {}", i + 1, current_url);

            let resp = client.head(&current_url).send().await?;

            if let Some(location) = resp.headers().get("location") {
                let location_str = location.to_str().unwrap_or("");
                debug!("[HTTP 重定向] Location: {}", location_str);

                // 处理相对路径重定向
                current_url = if location_str.starts_with("http") {
                    location_str.to_string()
                } else {
                    // 相对路径：基于当前 URL 构建完整 URL
                    match url::Url::parse(&current_url) {
                        Ok(base) => match base.join(location_str) {
                            Ok(full) => full.to_string(),
                            Err(_) => location_str.to_string(),
                        },
                        Err(_) => location_str.to_string(),
                    }
                };

                // 从重定向 URL 提取版本
                if version.is_none() {
                    version = self.extract_version(&current_url, version_extract_regex);
                }
            } else {
                debug!("[HTTP 重定向] 未找到 Location 头，重定向结束");
                // 最终响应没有 Location 头，尝试从最终 URL 提取版本
                if version.is_none() {
                    version = self.extract_version(&current_url, version_extract_regex);
                }
                break;
            }
        }

        if let Some(v) = &version {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v);
        } else {
            warn!("[版本检查] 未能从重定向 URL 提取版本: {}", upstream_url);
        }

        Ok(CheckResult {
            version,
            ..Default::default()
        })
    }
}

impl RedirectChecker {
    /// 从 URL 提取版本号
    fn extract_version(&self, url: &str, version_extract_regex: Option<&str>) -> Option<String> {
        if let Some(regex) = version_extract_regex {
            match extract_version_with_regex(url, regex) {
                Some(ver) => Some(ver),
                None => extract_version_from_url(url),
            }
        } else {
            extract_version_from_url(url)
        }
    }
}
