/**
 * sysops/mod.rs - 系统操作命令模块
 *
 * 本模块包含所有与系统操作相关的 Tauri 命令：
 * - sys_command: 系统命令执行（获取包版本、列出已安装包）
 * - backup_install: 备份包安装和信息查询（pacman -Qip、sudoers 配置）
 * - backup_basic: 备份基础操作（查询、清空备份表、删除备份）
 * - software_check: 版本检查（上游版本检查、批量检查）
 * - software_sync: 软件包同步（AUR 同步、上游检查、PKGBUILD 同步）
 * - upstream_validate: 上游 URL 验证（批量验证 URL 可达性）
 *
 * 模块设计原则：
 * - mod.rs 仅负责模块声明和导出，不包含具体实现
 * - 每个子文件负责单一功能，保持代码可维护性
 * - 所有文件严格控制在 300 行以内
 */

/// 系统命令执行（获取包版本、列出已安装包）
pub mod sys_command;

/// 代理工具函数（HTTP 客户端构建、代理获取）
pub mod proxy_utils;

/// 备份包安装和信息查询（pacman -Qip、sudoers 配置）
pub mod backup_install;

/// 备份基础操作（查询、清空备份表、删除备份）
pub mod backup_basic;

/// 版本检查（上游版本检查、批量检查）
pub mod software_check;

/// 软件包同步（AUR 同步、上游检查、PKGBUILD 同步）
pub mod software_sync;

/// 上游 URL 验证（批量验证 URL 可达性）
pub mod upstream_validate;

// 公开导出 Tauri 命令函数，供 lib.rs 注册使用
pub use sys_command::{get_package_version, list_installed_packages};
pub use backup_install::{get_package_file_info, check_sudoers_config, get_sudoers_command, install_backup_package};
pub use backup_basic::{list_backup_software, clear_backup_software, delete_backup};
pub use software_check::{check_upstream_version, check_selected_upstream};
pub use software_sync::{sync_from_aur, update_aur_info, sync_from_pkgbuild, check_all_upstream};
pub use upstream_validate::validate_upstream_urls;
