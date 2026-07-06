/**
 * errors/mod.rs - 统一错误处理模块入口
 *
 * 定义应用级错误类型 AppError，提供结构化错误信息
 * 所有模块统一使用 AppResult<T> 作为返回类型
 *
 * 模块结构：
 * - db.rs      — 数据库相关错误转换
 * - network.rs — 网络相关错误转换
 * - file.rs    — 文件操作相关错误转换
 * - system.rs  — 系统命令相关错误转换
 */
mod db;
mod file;
mod network;
mod system;

use serde::Serialize;
use thiserror::Error;

/// 应用统一错误类型
/// 每个变体对应一类业务错误，序列化后前端可据 code 字段分类处理
#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    /// 数据库文件不存在
    #[error("数据库文件不存在: {0}")]
    #[serde(rename = "DATABASE_NOT_FOUND")]
    DatabaseNotFound(String),

    /// 数据库文件格式错误或损坏
    #[error("数据库文件格式错误: {0}")]
    #[serde(rename = "DATABASE_CORRUPTED")]
    DatabaseCorrupted(String),

    /// 数据库操作错误（SQL 执行失败等）
    #[error("数据库错误: {0}")]
    #[serde(rename = "DATABASE_ERROR")]
    DatabaseError(String),

    /// 数据库锁获取失败
    #[error("数据库锁获取失败: {0}")]
    #[serde(rename = "DATABASE_LOCKED")]
    DatabaseLocked(String),

    /// 网络连接失败
    #[error("网络连接失败: {0}")]
    #[serde(rename = "NETWORK_CONNECT")]
    NetworkConnect(String),

    /// 网络请求超时
    #[error("网络请求超时: {0}")]
    #[serde(rename = "NETWORK_TIMEOUT")]
    NetworkTimeout(String),

    /// 网络请求错误（HTTP 状态码非 2xx 等）
    #[error("网络请求错误: {0}")]
    #[serde(rename = "NETWORK_ERROR")]
    NetworkError(String),

    /// 文件或目录不存在
    #[error("文件不存在: {0}")]
    #[serde(rename = "FILE_NOT_FOUND")]
    FileNotFound(String),

    /// 文件操作失败（读写、复制、移动等）
    #[error("文件操作失败: {0}")]
    #[serde(rename = "FILE_OPERATION")]
    FileOperation(String),

    /// 文件权限不足
    #[error("文件权限不足: {0}")]
    #[serde(rename = "FILE_PERMISSION")]
    FilePermission(String),

    /// 系统命令执行失败
    #[error("系统命令执行失败: {0}")]
    #[serde(rename = "SYSTEM_COMMAND")]
    SystemCommand(String),

    /// 系统命令未找到
    #[error("系统命令未找到: {0}")]
    #[serde(rename = "COMMAND_NOT_FOUND")]
    CommandNotFound(String),

    /// 软件包不存在
    #[error("软件包不存在: {0}")]
    #[serde(rename = "PACKAGE_NOT_FOUND")]
    PackageNotFound(String),

    /// 配置错误（缺少必要配置项等）
    #[error("配置错误: {0}")]
    #[serde(rename = "CONFIG_ERROR")]
    ConfigError(String),

    /// AUR 同步失败
    #[error("AUR 同步失败: {0}")]
    #[serde(rename = "AUR_SYNC_ERROR")]
    AurSyncError(String),

    /// 版本检查失败
    #[error("版本检查失败: {0}")]
    #[serde(rename = "VERSION_CHECK_ERROR")]
    VersionCheckError(String),

    /// 备份操作失败
    #[error("备份操作失败: {0}")]
    #[serde(rename = "BACKUP_ERROR")]
    BackupError(String),

    /// 数据解析失败（JSON、PKGBUILD 等）
    #[error("数据解析失败: {0}")]
    #[serde(rename = "PARSE_ERROR")]
    ParseError(String),

    /// 其他未分类错误
    #[error("{0}")]
    #[serde(rename = "UNKNOWN")]
    Unknown(String),
}

/// 应用统一返回类型
pub type AppResult<T> = Result<T, AppError>;

/// 从字符串创建未知错误
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Unknown(s)
    }
}

/// 从 &str 创建未知错误
impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Unknown(s.to_string())
    }
}

/// 转换为字符串（方便 Tauri command 返回 String 错误）
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}
