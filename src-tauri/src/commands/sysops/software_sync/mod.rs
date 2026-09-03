//! software_sync/mod.rs - 软件包同步命令模块入口
//!
//! 本模块包含所有与软件包同步相关的 Tauri 命令：
//! - aur: AUR 信息同步和更新
//! - upstream: 上游版本检查（并行执行）
//! - pkgbuild: PKGBUILD 文件同步
//! - import_aur: 从 AUR 搜索并导入新包
//! - utils: 同步工具函数和类型定义
//! - cache: GitHub tags 缓存增量校验辅助函数
//!
//! 模块设计原则：
//! - mod.rs 仅负责模块声明和导出，不包含具体实现
//! - 每个子文件负责单一功能，保持代码可维护性
//! - 所有文件严格控制在 300 行以内

/// AUR 信息同步和更新命令
pub mod aur;

/// 从 AUR 搜索并导入新软件包
pub mod import_aur;
pub use import_aur::{import_aur_package, search_aur_packages};

/// GitHub tags 缓存增量校验辅助函数
pub mod cache;
pub use cache::check_github_cache;

/// REST 回落场景的缓存补写：
/// GraphQL 失败时 batch 不产生快照，仓库永不写缓存，需在此补齐
pub mod cache_fill;

/// GitHub tags 缓存写盘辅助函数（从 batch.rs 拆分以符合 300 行约束）
pub mod batch_cache;
pub use batch_cache::{query_valid_github_caches, write_github_tag_cache};

/// 上游版本检查命令（并行执行）
pub mod upstream;

/// 上游批量检查执行引擎（分类 + 分组并发）
pub mod batch;

/// 批量检查辅助函数（从 batch.rs 拆分以符合 300 行约束）
mod batch_helpers;

/// 批量检查类型定义与常量（从 batch.rs 拆分以符合 300 行约束）
pub mod batch_engine;

/// PKGBUILD 文件同步命令
pub mod pkgbuild;

/// 同步工具函数和类型定义
pub mod utils;

// 公开导出 Tauri 命令函数，供 lib.rs 注册使用
pub use aur::sync_from_aur;
pub use aur::update_aur_info;
pub use pkgbuild::sync_from_pkgbuild;
pub use upstream::check_all_upstream;
