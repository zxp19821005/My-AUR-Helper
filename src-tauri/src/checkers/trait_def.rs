use async_trait::async_trait;
use reqwest::Client;

use crate::errors::AppResult;

#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    pub check_test_versions: bool,
    pub check_binary_files: bool,
}

/// 版本检查结果
#[derive(Debug, Clone, Default)]
pub struct CheckResult {
    /// 版本号
    pub version: Option<String>,
    /// License 列表（JSON 数组字符串，如 `["MIT", "GPL-3.0"]`）
    pub license: Option<String>,
    /// 编程语言名称列表（由检查器从上游获取）
    pub language_names: Vec<String>,
}

#[async_trait]
pub trait VersionChecker: Send + Sync {
    fn name(&self) -> &'static str;

    async fn check(
        &self,
        client: &Client,
        upstream_url: &str,
        pkgname: &str,
        version_extract_regex: Option<&str>,
        options: &CheckOptions,
    ) -> AppResult<CheckResult>;
}