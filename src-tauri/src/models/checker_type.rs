use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(into = "i32", try_from = "i32")]
pub enum CheckerType {
    GitHubRelease,
    GitHubTag,
    Gitee,
    GitLab,
    Redirect,
    Http,
    Manual,
    GitDescribe,
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
            CheckerType::GitDescribe => 8,
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
            8 => CheckerType::GitDescribe,
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
            CheckerType::GitDescribe,
        ]
    }
}

impl From<CheckerType> for i32 {
    fn from(ct: CheckerType) -> Self {
        ct.as_id()
    }
}

impl TryFrom<i32> for CheckerType {
    type Error = String;
    fn try_from(id: i32) -> Result<Self, Self::Error> {
        Ok(CheckerType::from_id(id))
    }
}
