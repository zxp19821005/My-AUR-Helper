use anyhow::Result;            // 通用错误处理
use async_trait::async_trait; // 异步 trait 支持
use reqwest::Client;          // HTTP 客户端

use super::trait_def::VersionChecker; // 版本检查器 trait

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
    async fn check(&self, _client: &Client, _upstream_url: &str, _pkgname: &str) -> Result<Option<String>> {
        Ok(None) // 手动检查器不执行任何网络请求
    }
}
