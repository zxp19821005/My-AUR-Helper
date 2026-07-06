/**
 * errors/network.rs - 网络相关错误转换
 *
 * 为 reqwest 和网络相关错误实现 From 转换
 */
use super::AppError;

/// 从 reqwest::Error 转换为 AppError
/// 根据错误类型细分：连接失败、超时、HTTP 错误等
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        // 连接失败
        if e.is_connect() {
            return AppError::NetworkConnect(format!("连接服务器失败: {}", e));
        }
        // 请求超时
        if e.is_timeout() {
            return AppError::NetworkTimeout(format!("请求超时: {}", e));
        }
        // DNS 解析失败
        if e.is_request() {
            let url = e.url().map(|u| u.as_str()).unwrap_or("未知 URL");
            return AppError::NetworkError(format!("请求失败 ({}): {}", url, e));
        }
        // 响应解析失败
        if e.is_decode() {
            return AppError::ParseError(format!("响应解析失败: {}", e));
        }
        // 其他网络错误
        AppError::NetworkError(e.to_string())
    }
}

/// 从 url::ParseError 转换为 AppError
impl From<url::ParseError> for AppError {
    fn from(e: url::ParseError) -> Self {
        AppError::ParseError(format!("URL 解析失败: {}", e))
    }
}
