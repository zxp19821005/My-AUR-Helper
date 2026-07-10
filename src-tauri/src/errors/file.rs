/**
 * errors/file.rs - 文件操作相关错误转换
 *
 * 为 std::io::Error 和文件操作相关错误实现 From 转换
 */
use super::AppError;

/// 从 std::io::Error 转换为 AppError
/// 根据错误类型细分：文件不存在、权限不足、其他 IO 错误
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            // 文件或目录不存在
            std::io::ErrorKind::NotFound => AppError::FileNotFound(e.to_string()),
            // 权限不足
            std::io::ErrorKind::PermissionDenied => AppError::FilePermission(e.to_string()),
            // 文件已存在
            std::io::ErrorKind::AlreadyExists => {
                AppError::FileOperation(format!("文件已存在: {}", e))
            }
            // 其他 IO 错误
            _ => AppError::FileOperation(e.to_string()),
        }
    }
}

/// 从 std::path::StripPrefixError 转换为 AppError
impl From<std::path::StripPrefixError> for AppError {
    fn from(e: std::path::StripPrefixError) -> Self {
        AppError::FileOperation(format!("路径前缀剥离失败: {}", e))
    }
}
