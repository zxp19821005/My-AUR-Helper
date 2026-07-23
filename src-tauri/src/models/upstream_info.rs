use serde::{Deserialize, Serialize};

/// 上游 URL 验证状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UpstreamUrlStatus {
    /// URL 可达，返回 200
    Ok,
    /// URL 返回 404
    NotFound,
    /// URL 返回 403 或 401
    Forbidden,
    /// URL 返回重定向 (3xx)
    Redirected,
    /// URL 返回服务器错误 (5xx)
    ServerError,
    /// 连接超时
    Timeout,
    /// 连接错误（DNS 解析失败、连接被拒绝等）
    ConnectionError,
    /// 其他错误
    OtherError,
}

impl UpstreamUrlStatus {
    /// 从字符串解析状态
    pub fn from_str(s: &str) -> Self {
        match s {
            "ok" => Self::Ok,
            "not_found" => Self::NotFound,
            "forbidden" => Self::Forbidden,
            "redirected" => Self::Redirected,
            "server_error" => Self::ServerError,
            "timeout" => Self::Timeout,
            "connection_error" => Self::ConnectionError,
            _ => Self::OtherError,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::NotFound => "not_found",
            Self::Forbidden => "forbidden",
            Self::Redirected => "redirected",
            Self::ServerError => "server_error",
            Self::Timeout => "timeout",
            Self::ConnectionError => "connection_error",
            Self::OtherError => "other_error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamInfo {
    pub software_id: i64,
    pub upstream_version: Option<String>,
    pub upstream_license_id: Option<String>,
    pub last_checked: Option<i64>,
    pub upstream_url_status: Option<UpstreamUrlStatus>,
}
