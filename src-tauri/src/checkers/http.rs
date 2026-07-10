use crate::errors::AppResult;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use super::trait_def::{CheckOptions, CheckResult, VersionChecker};
use super::utils::{extract_version_from_html, extract_version_with_regex};

pub struct HttpChecker;

#[async_trait]
impl VersionChecker for HttpChecker {
    fn name(&self) -> &'static str {
        "http"
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

        let resp = client
            .get(upstream_url)
            .header("User-Agent", "my-aur-helper/0.1")
            .send()
            .await?;
        if !resp.status().is_success() {
            return Ok(CheckResult::default());
        }

        let body = resp.text().await?;

        let version = if let Some(regex) = version_extract_regex {
            match extract_version_with_regex(&body, regex) {
                Some(ver) => Some(ver),
                None => extract_version_from_html(&body),
            }
        } else {
            extract_version_from_html(&body)
        };

        if let Some(v) = &version {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        Ok(CheckResult {
            version,
            ..Default::default()
        })
    }
}
