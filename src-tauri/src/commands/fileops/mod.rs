//! fileops/mod.rs - 文件操作命令模块
//!
//! 本模块包含所有与文件操作相关的 Tauri 命令：
//! - scan: 包文件扫描（.pkg.tar.zst 文件解析）
//! - cache_dirs: 缓存目录工具（路径展开、文件查找、目录获取）
//! - cache_scan: 缓存目录扫描和 cache_software 表管理
//! - cache_backup: 缓存包备份操作（备份到已有位置/指定子目录）
//! - backup_execute: 备份执行逻辑（文件复制、旧版本清理）
//! - backup_scan: 备份目录扫描和数据库写入
//! - backup_dedup: 备份去重逻辑（文件名解析、版本比较）
//!
//! 模块设计原则：
//! - mod.rs 仅负责模块声明和导出，不包含具体实现
//! - 每个子文件负责单一功能，保持代码可维护性
//! - 所有文件严格控制在 300 行以内

/// 包文件扫描（.pkg.tar.zst 文件解析）
pub mod scan;

/// 缓存目录工具（路径展开、文件查找、目录获取）
pub mod cache_dirs;

/// 缓存目录扫描和 cache_software 表管理
pub mod cache_scan;

/// 缓存包备份操作（备份到已有位置/指定子目录）
pub mod cache_backup;

/// 备份执行逻辑（文件复制、旧版本清理）
pub mod backup_execute;

/// 备份目录扫描和数据库写入
pub mod backup_scan;

/// 备份去重逻辑（文件名解析、版本比较）
pub mod backup_dedup;

// 公开导出 Tauri 命令函数，供 lib.rs 注册使用
pub use backup_dedup::{deduplicate_backups, parse_pkg_filename, DeduplicateResult};
pub use backup_execute::{run_backup, BackupConfig, BackupResult};
pub use backup_scan::{list_backup_subdirectories, scan_backup_directory};
pub use cache_backup::{backup_cache_to_existing, backup_cache_to_subdirectory};
pub use cache_dirs::{
    expand_tilde, extract_pkgname_from_cache, find_cache_file, get_cache_dirs, CacheDir,
};
pub use cache_scan::{
    clear_cache_software, list_cache_software, scan_all_cache_dirs, scan_cache_dir,
};
pub use scan::scan_pkg_files_cmd;
