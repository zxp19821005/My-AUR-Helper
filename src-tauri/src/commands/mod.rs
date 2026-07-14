/**
 * mod.rs - Tauri IPC 命令模块
 *
 * 定义所有前端可调用的 Tauri 命令
 * 每个子模块对应一个功能领域
 *
 * 安全审计说明（2026-07-14）：
 * - 已移除 files 模块：文件操作命令（read_file/delete_file/copy_file 等）
 *   未被前端使用，且存在路径遍历风险
 * - 已移除 sys_command 中的危险命令：run_command（任意命令执行）、
 *   install_package/remove_package（无验证的 sudo 调用）、
 *   makepkg（未使用的 makepkg 执行）、clean_cache/sync_database
 */

/// 备份管理命令
pub mod backup;

/// 扫描命令
pub mod scan;

/// 枚举值管理命令（License、编程语言）
pub mod enums;

/// 日志管理命令
pub mod logs;

/// 代理管理命令
pub mod proxy;

/// 代理工具函数（build_client、get_active_proxy）
pub mod proxy_utils;

/// 设置管理命令
pub mod settings;

/// 软件包 CRUD 和设置命令
pub mod software;

/// 软件包同步相关命令（AUR 同步、上游检查、PKGBUILD 同步）
pub mod software_sync;

/// 软件包上游版本检查相关命令
pub mod software_check;

/// 系统命令执行（仅保留安全命令：get_package_version、list_installed_packages）
pub mod sys_command;
