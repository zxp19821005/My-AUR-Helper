use crate::errors::AppResult;
use async_trait::async_trait;
use log::info;
use reqwest::Client;

use super::trait_def::{CheckOptions, CheckResult, VersionChecker};

pub struct ManualChecker;

#[async_trait]
impl VersionChecker for ManualChecker {
    fn name(&self) -> &'static str {
        "manual"
    }

    async fn check(
        &self,
        _client: &Client,
        upstream_url: &str,
        pkgname: &str,
        _version_extract_regex: Option<&str>,
        _options: &CheckOptions,
    ) -> AppResult<CheckResult> {
        info!(
            "[版本检查] 开始检查软件包: {} (检查器: {})",
            pkgname,
            self.name()
        );
        info!("[版本检查] 手动检查器不执行网络请求，请用户手动更新版本号");
        info!("[版本检查] 上游URL: {}", upstream_url);
        Ok(CheckResult::default())
    }
}
