/**
 * errors/system.rs - 系统命令和其他错误转换
 *
 * 为系统命令执行、JSON 解析等错误实现 From 转换
 */
use super::AppError;

/// 从 serde_json::Error 转换为 AppError
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::ParseError(format!("JSON 解析失败: {}", e))
    }
}

/// 从 regex::Error 转换为 AppError
impl From<regex::Error> for AppError {
    fn from(e: regex::Error) -> Self {
        AppError::ParseError(format!("正则表达式错误: {}", e))
    }
}

/// 从 anyhow::Error 转换为 AppError
/// anyhow 错误保留为字符串信息
impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Unknown(e.to_string())
    }
}

/// 从 Box<dyn std::error::Error> 转换为 AppError
impl From<Box<dyn std::error::Error>> for AppError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        AppError::Unknown(e.to_string())
    }
}

/// 从 Box<dyn std::error::Error + Send + Sync> 转换为 AppError
impl From<Box<dyn std::error::Error + Send + Sync>> for AppError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AppError::Unknown(e.to_string())
    }
}
