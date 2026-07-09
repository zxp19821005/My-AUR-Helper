mod github;
mod gitee;
mod gitlab;
mod http;
mod manual;
mod redirect;
mod trait_def;
mod utils;
mod git_describe;

pub use trait_def::{CheckOptions, VersionChecker};

use crate::models::CheckerType;

#[derive(Debug, Clone, Default)]
pub struct CheckerSettings {
    pub github_token: Option<String>,
    pub gitee_token: Option<String>,
    pub gitlab_token: Option<String>,
}

pub fn get_checker(checker_type: &CheckerType, settings: CheckerSettings) -> Box<dyn VersionChecker> {
    match checker_type {
        CheckerType::GitHubRelease => Box::new(github::GitHubReleaseChecker::new(settings.github_token)),
        CheckerType::GitHubTag => Box::new(github::GitHubTagChecker::new(settings.github_token)),
        CheckerType::Gitee => Box::new(gitee::GiteeChecker::new(settings.gitee_token)),
        CheckerType::GitLab => Box::new(gitlab::GitLabChecker::new(settings.gitlab_token)),
        CheckerType::Redirect => Box::new(redirect::RedirectChecker),
        CheckerType::Http => Box::new(http::HttpChecker),
        CheckerType::Manual => Box::new(manual::ManualChecker),
        CheckerType::GitDescribe => Box::new(git_describe::GitDescribeChecker),
    }
}
