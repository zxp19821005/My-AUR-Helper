use crate::errors::AppResult;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use super::trait_def::VersionChecker;

/// 手动检查器
/// 不对任何上游进行自动版本检查，始终返回 None
/// 用于那些无法通过 API 或页面解析获取版本号的软件包
pub struct ManualChecker;

#[async_trait]
impl VersionChecker for ManualChecker {
    fn name(&self) -> &'static str {
        "manual"
    }

    /// 手动检查 - 始终不返回版本信息
    /// 用户需要手动更新版本号
    async fn check(&self, _client: &Client, upstream_url: &str, pkgname: &str, _version_extract_regex: Option<&str>) -> AppResult<Option<String>> {
        info!("[版本检查] 开始检查软件包: {} (检查器: {})", pkgname, self.name());
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 手动检查器不执行网络请求，请用户手动更新版本号");
        Ok(None)
    }
}
