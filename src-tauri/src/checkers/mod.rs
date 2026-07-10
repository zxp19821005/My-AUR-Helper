mod gitee;
mod github;
mod gitlab;
mod http;
mod manual;
mod redirect;
mod trait_def;
mod utils;

pub use trait_def::{CheckOptions, CheckResult, VersionChecker};

use crate::models::CheckerType;

#[derive(Debug, Clone, Default)]
pub struct CheckerSettings {
    pub github_token: Option<String>,
    pub gitee_token: Option<String>,
    pub gitlab_token: Option<String>,
}

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
