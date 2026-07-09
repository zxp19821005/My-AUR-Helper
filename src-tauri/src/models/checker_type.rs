use serde::{Deserialize, Serialize};

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