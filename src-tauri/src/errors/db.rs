/**
 * errors/db.rs - 数据库相关错误转换
 *
 * 为 rusqlite 和数据库相关错误实现 From 转换
 */
use super::AppError;
use std::sync::PoisonError;

/// 从 rusqlite::Error 转换为 AppError
/// 根据错误类型细分：文件不存在、SQL 错误、连接错误等
impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        match &e {
            // 数据库文件无法打开（文件不存在或路径错误）
            rusqlite::Error::SqliteFailure(err, msg) => {
                let code = err.extended_code;
                // SQLITE_CANTOPEN (14): 无法打开数据库文件
                if code == 14 {
                    let path = msg.as_deref().unwrap_or("未知路径");
                    return AppError::DatabaseNotFound(format!("{} ({})", e, path));
                }
                // SQLITE_CORRUPT (11): 数据库文件损坏
                if code == 11 {
                    return AppError::DatabaseCorrupted(e.to_string());
                }
                // SQLITE_READONLY (8): 只读错误，可能是权限问题
                if code == 8 {
                    return AppError::DatabaseError(format!("数据库只读: {}", e));
                }
                AppError::DatabaseError(e.to_string())
            }
            // 数据库连接失败
            rusqlite::Error::InvalidParameterName(_) | rusqlite::Error::InvalidQuery => {
                AppError::DatabaseError(format!("数据库参数错误: {}", e))
            }
            // 其他数据库错误
            _ => AppError::DatabaseError(e.to_string()),
        }
    }
}

/// 从 Mutex PoisonError 转换为 AppError
/// 数据库锁中毒通常由于 panic 导致
impl<T> From<PoisonError<T>> for AppError {
    fn from(e: PoisonError<T>) -> Self {
        AppError::DatabaseLocked(format!("数据库锁中毒: {}", e))
    }
}
