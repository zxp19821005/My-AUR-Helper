/**
 * api_checker.rs - GitHubAPIChecker 检查器实现
 *
 * 功能：通过 GitHub Release API 获取最新版本号。
 * 适用场景：
 * - 需要获取最新 release 版本（非 tag 列表）
 * - 支持二进制文件检查（check_binary_files 选项）
 * - 支持通过版本提取关键字过滤资产文件
 * - 如果最新版本没有匹配的二进制文件，自动回退查找历史版本
 *
 * 工作流程：
 * 1. 从 upstream_url 解析 owner 和 repo
 * 2. 如果是 -git 包，使用 git_describe 逻辑
 * 3. 如果启用 check_test_versions，遍历所有 releases 查找最新版本
 * 4. 否则直接获取 latest release
 * 5. 如果启用 check_binary_files，检查资产是否包含匹配的二进制文件
 * 6. 返回找到的最新版本
 */
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use crate::checkers::github::release::{check_github_release_latest, check_github_releases};
use crate::checkers::github::tags::check_github_tags;
use crate::checkers::github::repo_info::{fetch_github_repo_languages, fetch_github_repo_license};
use crate::checkers::github::git_describe::check_github_git_describe;
use crate::checkers::trait_def::{CheckOptions, CheckResult, VersionChecker};
use crate::checkers::utils::extract_owner_repo;
use crate::errors::AppResult;

/// GitHub API 检查器
/// 通过 Release API 获取最新版本，支持二进制文件检查
pub struct GitHubAPIChecker {
    /// GitHub API Token（可选，用于提高请求频率限制）
    token: Option<String>,
}

impl GitHubAPIChecker {
    /// 创建新的 GitHubAPIChecker 实例
    ///
    /// # 参数
    /// - `token`: GitHub Personal Access Token，可选
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

#[async_trait]
impl VersionChecker for GitHubAPIChecker {
    /// 返回检查器名称
    fn name(&self) -> &'static str {
        "github_api"
    }

    /// 执行版本检查
    ///
    /// # 参数
    /// - `client`: HTTP 客户端
    /// - `upstream_url`: 上游仓库 URL
    /// - `pkgname`: 软件包名称
    /// - `version_extract_regex`: 版本提取正则表达式（可选）
    ///   - 当 check_binary_files 启用时，此参数用作资产文件名过滤器
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
        info!(
            "[GitHub API] 认证: {}",
            if self.token.is_some() {
                "已配置 Token"
            } else {
                "未配置 Token"
            }
        );

        // 获取 license 信息
        let license = fetch_github_repo_license(client, &owner, &repo, self.token.as_deref())
            .await
            .unwrap_or_else(|e| {
                debug!("[版本检查] 获取 license 失败: {}", e);
                None
            });

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
            return Ok(CheckResult { version, license, language_names });
        }

        // 如果启用测试版本检查，遍历所有 releases
        if options.check_test_versions {
            let version = check_github_releases(
                client,
                &owner,
                &repo,
                self.token.as_deref(),
                version_extract_regex,
                true, // check_test_versions = true
                options.check_binary_files,
                pkgname,
            )
            .await?;
            if let Some(v) = &version {
                info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v);
            }
            return Ok(CheckResult { version, license, language_names });
        }

        // 默认：先获取 latest release
        let version = check_github_release_latest(
            client,
            &owner,
            &repo,
            self.token.as_deref(),
            version_extract_regex,
            options.check_binary_files,
            pkgname,
        ).await?;
        
        // 如果 releases 为空，且未启用二进制文件检查，则 fallback 到 tags API
        // 如果启用了二进制文件检查，则不应该 fallback 到 tags（因为 tags 没有资产信息）
        if version.is_none() && !options.check_binary_files {
            debug!(
                "[GitHub API] {}: 未找到 releases，尝试使用 tags API",
                pkgname
            );
            let tags_version = check_github_tags(
                client,
                &owner,
                &repo,
                self.token.as_deref(),
                version_extract_regex,
                options.check_test_versions,
            )
            .await?;
            
            if let Some(v) = &tags_version {
                info!("[版本检查] 检查完成: {} -> 上游版本={} (来自 tags)", pkgname, v);
            }
            return Ok(CheckResult { version: tags_version, license, language_names });
        }
        
        if let Some(v) = &version {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        Ok(CheckResult { version, license, language_names })
    }
}