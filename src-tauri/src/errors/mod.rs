/**
 * errors/mod.rs - 统一错误处理模块入口
 *
 * 模块结构：
 * - error_type.rs — AppError 枚举定义和 AppResult 类型别名
 * - db.rs         — 数据库相关错误转换
 * - network.rs    — 网络相关错误转换
 * - file.rs       — 文件操作相关错误转换
 * - system.rs     — 系统命令相关错误转换
 */
mod db;
mod error_type;
mod file;
mod network;
mod system;

pub use error_type::{AppError, AppResult};
