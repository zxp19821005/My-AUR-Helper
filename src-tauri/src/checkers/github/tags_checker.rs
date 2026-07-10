/**
 * tags_checker.rs - GitHubTagsChecker 检查器实现
 *
 * 功能：通过 GitHub Tags API 获取仓库的所有 tags，并提取最新版本号。
 * 适用场景：
 * - 需要获取大量 tags 的场景（如 electron2-bin 可能需要几千个 tags）
 * - 支持版本提取关键字设置，可以过滤特定格式的 tag
 * - 支持检查测试版本（prerelease）选项
 *
 * 工作流程：
 * 1. 从 upstream_url 解析 owner 和 repo
 * 2. 如果是 -git 包，使用 git_describe 逻辑
 * 3. 否则调用 check_github_tags 分页获取所有 tags
 * 4. 使用版本提取正则或默认逻辑提取版本号
 * 5. 比较所有版本，返回最新版本
 */
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use crate::checkers::github::api::fetch_github_repo_languages;
use crate::checkers::github::git_describe::check_github_git_describe;
use crate::checkers::github::tags::check_github_tags;
use crate::checkers::trait_def::{CheckOptions, CheckResult, VersionChecker};
use crate::checkers::utils::extract_owner_repo;
use crate::errors::AppResult;

/// GitHub Tags 检查器
/// 通过分页获取所有 tags，提取并比较版本号
pub struct GitHubTagsChecker {
    /// GitHub API Token（可选，用于提高请求频率限制）
    token: Option<String>,
}

impl GitHubTagsChecker {
    /// 创建新的 GitHubTagsChecker 实例
    ///
    /// # 参数
    /// - `token`: GitHub Personal Access Token，可选
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

#[async_trait]
impl VersionChecker for GitHubTagsChecker {
    /// 返回检查器名称
    fn name(&self) -> &'static str {
        "github_tags"
    }

    /// 执行版本检查
    ///
    /// # 参数
    /// - `client`: HTTP 客户端
    /// - `upstream_url`: 上游仓库 URL
    /// - `pkgname`: 软件包名称
    /// - `version_extract_regex`: 版本提取正则表达式（可选）
    /// - `options`: 检查选项（是否检查测试版本、二进制文件等）
    ///
    /// # 返回
    /// - `Ok(CheckResult)`: 包含版本号和 license 信息
    /// - `Err(e)`: 检查过程中发生错误
    async fn check(
        &self,
        client: &Client,
        upstream_url: &str,
        pkgname: &str,
        version_extract_regex: Option<&str>,
        options: &CheckOptions,
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
        let (owner, repo) = match extract_owner_repo(upstream_url) {
            Some(pair) => pair,
            None => {
                debug!("[版本检查] 无法解析 GitHub URL: {}", upstream_url);
                return Ok(CheckResult::default());
            }
        };

        // 获取编程语言列表
        let language_names = fetch_github_repo_languages(client, &owner, &repo, self.token.as_deref())
            .await
            .unwrap_or_else(|e| {
                debug!("[版本检查] 获取 languages 失败: {}", e);
                vec![]
            });

        // -git 包使用 git describe 逻辑
        if pkgname.ends_with("-git") {
            let version = check_github_git_describe(client, &owner, &repo, self.token.as_deref(), pkgname).await?;
            if let Some(v) = &version {
                info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v);
            } else {
                debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
            }
            return Ok(CheckResult { version, license: None, language_names });
        }

        let version = check_github_tags(
            client,
            &owner,
            &repo,
            self.token.as_deref(),
            version_extract_regex,
            options.check_test_versions,
        )
        .await?;
        if let Some(v) = &version {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        Ok(CheckResult { version, license: None, language_names })
    }
}