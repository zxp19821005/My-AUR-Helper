/**
 * mod.rs - 软件包同步命令模块入口
 *
 * 本模块包含所有与软件包同步相关的 Tauri 命令：
 * - aur: AUR 信息同步和更新
 * - upstream: 上游版本检查（并行执行）
 * - pkgbuild: PKGBUILD 文件同步
 * - utils: 同步工具函数和类型定义
 *
 * 模块设计原则：
 * - mod.rs 仅负责模块声明和导出，不包含具体实现
 * - 每个子文件负责单一功能，保持代码可维护性
 * - 所有文件严格控制在 300 行以内
 */

/// AUR 信息同步和更新命令
pub mod aur;

/// 上游版本检查命令（并行执行）
pub mod upstream;

/// PKGBUILD 文件同步命令
pub mod pkgbuild;

/// 同步工具函数和类型定义
mod utils;

// 公开导出 Tauri 命令函数，供 lib.rs 注册使用
pub use aur::sync_from_aur;
pub use aur::update_aur_info;
pub use upstream::check_all_upstream;
pub use pkgbuild::sync_from_pkgbuild;