use super::{gitee, github, gitlab, http, manual, redirect, trait_def::VersionChecker};
/**
 * factory.rs - 检查器工厂
 *
 * 根据检查器类型创建对应的 VersionChecker 实例
 */
use crate::models::CheckerType;

/// 检查器配置（包含各平台 Token）
#[derive(Debug, Clone, Default)]
pub struct CheckerSettings {
    pub github_token: Option<String>,
    pub gitee_token: Option<String>,
    pub gitlab_token: Option<String>,
}

/// 根据检查器类型创建对应的检查器实例
pub fn get_checker(
    checker_type: &CheckerType,
    settings: CheckerSettings,
) -> Box<dyn VersionChecker> {
    match checker_type {
        CheckerType::GitHubTags => Box::new(github::GitHubTagsChecker::new(settings.github_token)),
        CheckerType::GitHubAPI => Box::new(github::GitHubAPIChecker::new(settings.github_token)),
        CheckerType::Gitee => Box::new(gitee::GiteeChecker::new(settings.gitee_token)),
        CheckerType::GitLab => Box::new(gitlab::GitLabChecker::new(settings.gitlab_token)),
        CheckerType::Redirect => Box::new(redirect::RedirectChecker),
        CheckerType::Http => Box::new(http::HttpChecker),
        CheckerType::Manual => Box::new(manual::ManualChecker),
    }
}
