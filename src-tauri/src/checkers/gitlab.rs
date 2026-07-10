use crate::errors::AppResult;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use super::trait_def::{CheckOptions, CheckResult, VersionChecker};
use super::utils::{clean_version, extract_version_with_regex};

pub struct GitLabChecker {
    token: Option<String>,
}

impl GitLabChecker {
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

#[async_trait]
impl VersionChecker for GitLabChecker {
    fn name(&self) -> &'static str {
        "gitlab"
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
        let parts: Vec<&str> = upstream_url
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .split('/')
            .collect();
        if parts.len() < 2 {
            debug!("[版本检查] 无法解析 GitLab URL: {}", upstream_url);
            return Ok(CheckResult::default());
        }
        let owner = parts[parts.len() - 2];
        let repo = parts[parts.len() - 1];

        let project_path = format!("{}%2F{}", owner, repo);
        let api_url = format!(
            "https://gitlab.com/api/v4/projects/{}/releases/permalink/latest",
            project_path
        );

        let mut req = client
            .get(&api_url)
            .header("User-Agent", "my-aur-helper/0.1");
        if let Some(token) = &self.token {
            req = req.header("PRIVATE-TOKEN", token);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            return Ok(CheckResult::default());
        }

        let data: serde_json::Value = resp.json().await?;
        let version = if let Some(tag) = data["tag_name"].as_str() {
            let ver = if let Some(regex) = version_extract_regex {
                extract_version_with_regex(tag, regex).unwrap_or_else(|| clean_version(tag))
            } else {
                clean_version(tag)
            };
            Some(ver)
        } else {
            None
        };

        if let Some(v) = &version {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v);
        }
        Ok(CheckResult {
            version,
            ..Default::default()
        })
    }
}
