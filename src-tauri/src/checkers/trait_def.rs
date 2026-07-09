use async_trait::async_trait;
use reqwest::Client;

use crate::errors::AppResult;

#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    pub check_test_versions: bool,
    pub check_binary_files: bool,
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
    ) -> AppResult<Option<String>>;
}
