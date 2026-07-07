use async_trait::async_trait;
use reqwest::Client;

use crate::errors::AppResult;

/// 版本检查器 trait
/// 所有版本检查策略必须实现此 trait
#[async_trait]
pub trait VersionChecker: Send + Sync {
    /// 获取检查器名称
    fn name(&self) -> &'static str;

    /// 检查上游版本
    /// @param client - HTTP 客户端
    /// @param upstream_url - 上游 URL
    /// @param pkgname - 软件包名称
    /// @param version_extract_regex - 可选的版本提取正则表达式
    /// @returns 可选的版本字符串，None 表示无法确定版本
    async fn check(&self, client: &Client, upstream_url: &str, pkgname: &str, version_extract_regex: Option<&str>) -> AppResult<Option<String>>;
}
