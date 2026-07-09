use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use crate::checkers::trait_def::{CheckOptions, VersionChecker};
use crate::checkers::utils::extract_owner_repo;
use crate::checkers::github::git_describe::check_github_git_describe;
use crate::checkers::github::tags::check_github_tags;
use crate::errors::AppResult;

pub struct GitHubTagsChecker {
    token: Option<String>,
}

impl GitHubTagsChecker {
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

#[async_trait]
impl VersionChecker for GitHubTagsChecker {
    fn name(&self) -> &'static str {
        "github_tags"
    }

    async fn check(
        &self,
        client: &Client,
        upstream_url: &str,
        pkgname: &str,
        version_extract_regex: Option<&str>,
        options: &CheckOptions,
    ) -> AppResult<Option<String>> {
        info!("[版本检查] 开始检查软件包: {} (检查器: {})", pkgname, self.name());
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);

        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(None);
        }
        let (owner, repo) = match extract_owner_repo(upstream_url) {
            Some(pair) => pair,
            None => {
                debug!("[版本检查] 无法解析 GitHub URL: {}", upstream_url);
                return Ok(None);
            }
        };

        // -git 包使用 git describe 逻辑
        if pkgname.ends_with("-git") {
            let result = check_github_git_describe(
                client, &owner, &repo, pkgname,
            ).await;
            if let Ok(Some(version)) = &result {
                info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, version);
            } else {
                debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
            }
            return result;
        }

        let result = check_github_tags(
            client, &owner, &repo, self.token.as_deref(),
            version_extract_regex, options.check_test_versions,
        ).await;
        if let Ok(Some(version)) = &result {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, version);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        result
    }
}