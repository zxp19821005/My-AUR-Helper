/**
 * checker_type.rs - 版本检查器类型枚举（CheckerType）
 *
 * 功能：
 * - 定义支持的版本检查器类型，并与数据库存储的整型 ID 互转
 * - 新增检查器类型时，必须同步维护 as_id / from_id / all 三处映射，
 *   否则会出现「枚举↔ID 不一致」导致检查器工厂 get_checker 找不到对应实现
 */

use serde::{Deserialize, Serialize};

/// 版本检查器类型枚举
///
/// 与数据库 `software_info.checker_type_id` 字段对应的整型 ID 互转
/// （见 `as_id` / `from_id`）。`Browser` 为浏览器 JS 渲染检查器，需本机安装
/// Chromium / Chrome。新增类型时务必同步更新 `as_id`、`from_id`、`all` 三处。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(into = "i32", try_from = "i32")]
pub enum CheckerType {
    GitHubTags,
    GitHubAPI,
    Gitee,
    GitLab,
    Redirect,
    Http,
    Manual,
    Browser,
}

impl CheckerType {
    pub fn as_id(&self) -> i32 {
        match self {
            CheckerType::GitHubTags => 1,
            CheckerType::GitHubAPI => 2,
            CheckerType::Gitee => 3,
            CheckerType::GitLab => 4,
            CheckerType::Redirect => 5,
            CheckerType::Http => 6,
            CheckerType::Manual => 7,
            CheckerType::Browser => 8,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => CheckerType::GitHubTags,
            2 => CheckerType::GitHubAPI,
            3 => CheckerType::Gitee,
            4 => CheckerType::GitLab,
            5 => CheckerType::Redirect,
            6 => CheckerType::Http,
            7 => CheckerType::Manual,
            8 => CheckerType::Browser,
            _ => CheckerType::Manual,
        }
    }

    pub fn all() -> Vec<CheckerType> {
        vec![
            CheckerType::GitHubTags,
            CheckerType::GitHubAPI,
            CheckerType::Gitee,
            CheckerType::GitLab,
            CheckerType::Redirect,
            CheckerType::Http,
            CheckerType::Manual,
            CheckerType::Browser,
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
