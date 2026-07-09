/**
 * mod.rs - Tauri IPC 命令模块
 *
 * 定义所有前端可调用的 Tauri 命令
 * 每个子模块对应一个功能领域
 */

/// 备份管理命令
pub mod backup;

/// 扫描命令
pub mod scan;

/// 枚举值管理命令（License、编程语言）
pub mod enums;

/// 文件操作命令
pub mod files;

/// 包文件扫描命令
pub mod files_scan;

/// 日志管理命令
pub mod logs;

/// 代理管理命令
pub mod proxy;

/// 设置管理命令
pub mod settings;

/// 软件包 CRUD 和设置命令
pub mod software;

/// 软件包 AUR 同步相关命令
pub mod software_sync;

/// 软件包 PKGBUILD 同步相关命令
pub mod software_sync_pkgbuild;

/// 软件包上游版本检查相关命令
pub mod software_check;

/// 系统命令执行
pub mod sys_command;
