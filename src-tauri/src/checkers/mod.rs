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

pub use trait_def::VersionChecker; // 导出 trait，供外部使用

use crate::models::CheckerType; // 检查器类型枚举

/// 根据检查器类型获取对应的检查器实例
/// @param checker_type - 检查器类型枚举值
/// @returns 实现了 VersionChecker trait 的盒装对象
pub fn get_checker(checker_type: &CheckerType) -> Box<dyn VersionChecker> {
    match checker_type {
        CheckerType::GitHubRelease => Box::new(github::GitHubReleaseChecker),
        CheckerType::GitHubTag => Box::new(github::GitHubTagChecker),
        CheckerType::Gitee => Box::new(gitee::GiteeChecker),
        CheckerType::GitLab => Box::new(gitlab::GitLabChecker),
        CheckerType::Redirect => Box::new(redirect::RedirectChecker),
        CheckerType::Http => Box::new(http::HttpChecker),
        CheckerType::Manual => Box::new(manual::ManualChecker),
    }
}
