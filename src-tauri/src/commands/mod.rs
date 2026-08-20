//! mod.rs - Tauri IPC 命令模块
//!
//! 定义所有前端可调用的 Tauri 命令
//! 每个子模块对应一个功能领域
//!
//! 安全审计说明（2026-07-14）：
//! - 已移除 files 模块：文件操作命令（read_file/delete_file/copy_file 等）
//!   未被前端使用，且存在路径遍历风险
//! - 已移除 sys_command 中的危险命令：run_command（任意命令执行）、
//!   install_package/remove_package（无验证的 sudo 调用）、
//!   makepkg（未使用的 makepkg 执行）、clean_cache/sync_database

/// 文件操作命令模块（目录扫描、文件拷贝、备份操作等）
pub mod fileops;

/// 系统操作命令模块（软件安装、缓存清理、版本检查等）
pub mod sysops;

/// 枚举值管理命令（License、编程语言）
pub mod enums;

/// 日志管理命令
pub mod logs;

/// 代理管理命令
pub mod proxy;

/// 设置管理命令
pub mod settings;

/// 软件包 CRUD 和设置命令
pub mod software;

/// 仪表盘统计命令
pub mod dashboard;

/// 前端诊断日志转发命令（仅打印到终端，不写文件）
pub mod fe_log;
