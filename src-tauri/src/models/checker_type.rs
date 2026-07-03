use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 版本检查器类型枚举
/// 定义检查上游版本的不同策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheckerType {
    /// GitHub Release (ID: 1) - 通过 GitHub API 获取最新 release
    GitHubRelease,
    /// GitHub Tag (ID: 2) - 通过 GitHub API 获取最新 tag
    GitHubTag,
    /// Gitee (ID: 3) - 通过 Gitee API v5 获取最新 release
    Gitee,
    /// GitLab (ID: 4) - 通过 GitLab API v4 获取最新 release
    GitLab,
    /// HTTP 重定向 (ID: 5) - 从重定向 URL 中提取版本号
    Redirect,
    /// HTTP 页面解析 (ID: 6) - 从 HTML 页面内容中提取版本号
    Http,
    /// 手动检查 (ID: 7) - 不自动检查，由用户手动更新
    Manual,
}

impl CheckerType {
    /// 将枚举转换为数字 ID（用于数据库存储）
    pub fn as_id(&self) -> i32 {
        match self {
            CheckerType::GitHubRelease => 1,
            CheckerType::GitHubTag => 2,
            CheckerType::Gitee => 3,
            CheckerType::GitLab => 4,
            CheckerType::Redirect => 5,
            CheckerType::Http => 6,
            CheckerType::Manual => 7,
        }
    }

    /// 从数字 ID 创建枚举（用于数据库查询反序列化）
    /// @param id - 数字 ID
    /// @returns 对应的枚举值，未知 ID 默认返回 Manual
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => CheckerType::GitHubRelease,
            2 => CheckerType::GitHubTag,
            3 => CheckerType::Gitee,
            4 => CheckerType::GitLab,
            5 => CheckerType::Redirect,
            6 => CheckerType::Http,
            _ => CheckerType::Manual, // 默认值
        }
    }

    /// 获取所有检查器类型的完整列表
    /// @returns 包含所有变体的 Vec
    pub fn all() -> Vec<CheckerType> {
        vec![
            CheckerType::GitHubRelease,
            CheckerType::GitHubTag,
            CheckerType::Gitee,
            CheckerType::GitLab,
            CheckerType::Redirect,
            CheckerType::Http,
            CheckerType::Manual,
        ]
    }
}
