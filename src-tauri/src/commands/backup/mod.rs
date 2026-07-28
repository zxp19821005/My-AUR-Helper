/**
 * backup/mod.rs - 备份管理命令模块
 *
 * 包含备份管理相关的所有 Tauri 命令：
 * - dedup: 去重逻辑（文件名解析、版本比较）
 * - backup_basic: 基础操作（查询、扫描、去重、删除）
 * - backup_install: 安装和信息查询操作
 */
mod backup_basic;
mod backup_install;
mod dedup;

pub use backup_basic::*;
pub use backup_install::*;
pub use dedup::DeduplicateResult;
