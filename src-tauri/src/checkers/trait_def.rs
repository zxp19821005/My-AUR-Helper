use anyhow::Result;       // 通用错误处理
use async_trait::async_trait; // 异步 trait 支持
use reqwest::Client;      // HTTP 客户端

/// 版本检查器 trait
/// 所有版本检查策略必须实现此 trait
#[async_trait]
pub trait VersionChecker: Send + Sync {
    /// 获取检查器名称
    /// @returns 检查器的字符串标识符
    fn name(&self) -> &'static str;

    /// 检查上游版本
    /// @param client - 复用的 HTTP 客户端
    /// @param upstream_url - 上游项目的 URL
    /// @param pkgname - 本地包名（某些检查器可能需要用于 API 查询）
    /// @returns 可选的版本字符串，None 表示无法确定版本
    async fn check(&self, client: &Client, upstream_url: &str, pkgname: &str) -> Result<Option<String>>;
}
