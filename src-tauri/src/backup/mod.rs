/**
 * backup/mod.rs - 备份管理模块
 *
 * 提供软件包备份文件的复制和旧版本清理功能
 */
mod execute; // 备份执行子模块

pub use execute::{run_backup, BackupConfig, BackupResult}; // 导出备份配置、结果类型和执行函数
