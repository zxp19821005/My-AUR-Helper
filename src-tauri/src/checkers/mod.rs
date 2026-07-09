/**
 * checkers/mod.rs - 版本检查器模块
 *
 * 提供多种上游版本检查策略，每种策略实现 VersionChecker trait
 * 支持 GitHub、Gitee、GitLab、HTTP 重定向、HTML 解析和手动检查
 */
mod github;   // GitHub Release/Tag 检查器
mod gitee;    // Gitee 检查器
mod gitlab;   // GitLab 检查器
mod http;     // HTTP 页面解析检查器
mod manual;   // 手动检查器（返回 None）
mod redirect; // HTTP 重定向检查器
mod trait_def;// VersionChecker trait 定义
mod utils;    // 工具函数（URL 解析、版本号清理等）

pub use trait_def::VersionChecker;

use crate::models::CheckerType;

/// 检查器通用配置
/// 从数据库读取，传递给各检查器使用
#[derive(Debug, Clone, Default)]
pub struct CheckerSettings {
    /// GitHub Personal Access Token
    pub github_token: Option<String>,
    /// Gitee 私人令牌（access_token）
    pub gitee_token: Option<String>,
    /// GitLab Personal Access Token
    pub gitlab_token: Option<String>,
}

/// 根据检查器类型获取对应的检查器实例
pub fn get_checker(checker_type: &CheckerType, settings: CheckerSettings) -> Box<dyn VersionChecker> {
    match checker_type {
        CheckerType::GitHubRelease => Box::new(github::GitHubReleaseChecker::new(settings.github_token)),
        CheckerType::GitHubTag => Box::new(github::GitHubTagChecker::new(settings.github_token)),
        CheckerType::Gitee => Box::new(gitee::GiteeChecker::new(settings.gitee_token)),
        CheckerType::GitLab => Box::new(gitlab::GitLabChecker::new(settings.gitlab_token)),
        CheckerType::Redirect => Box::new(redirect::RedirectChecker),
        CheckerType::Http => Box::new(http::HttpChecker),
        CheckerType::Manual => Box::new(manual::ManualChecker),
    }
}
