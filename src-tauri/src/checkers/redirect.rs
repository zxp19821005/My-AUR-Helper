use crate::errors::AppResult;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use super::trait_def::{CheckOptions, VersionChecker};
use super::utils::{extract_version_from_url, extract_version_with_regex};

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
    ) -> AppResult<Option<String>> {
        info!("[版本检查] 开始检查软件包: {} (检查器: {})", pkgname, self.name());
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);

        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(None);
        }

        let resp = client.head(upstream_url).send().await?;

        let result = if let Some(location) = resp.headers().get("location") {
            let location_str = location.to_str().unwrap_or("");
            debug!("[HTTP 重定向] Location: {}", location_str);

            if let Some(regex) = version_extract_regex {
                match extract_version_with_regex(location_str, regex) {
                    Some(ver) => Ok(Some(ver)),
                    None => Ok(extract_version_from_url(location_str)),
                }
            } else {
                Ok(extract_version_from_url(location_str))
            }
        } else {
            debug!("[HTTP 重定向] 未找到 Location 头");
            Ok(None)
        };

        if let Ok(Some(version)) = &result {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, version);
        }
        result
    }
}
