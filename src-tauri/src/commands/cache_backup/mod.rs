/**
 * cache_backup/mod.rs - 缓存包备份和扫描命令模块
 *
 * 包含缓存管理相关的所有 Tauri 命令：
 * - dirs: 缓存目录配置获取、路径展开、文件查找等通用工具
 * - backup: 缓存包备份操作（备份到已有位置、指定子目录）
 * - scan: 缓存目录扫描、cache_software 表管理
 */
mod dirs;
mod backup;
mod scan;

pub use backup::*;
pub use dirs::{expand_tilde, find_cache_file, get_cache_dirs, CacheDir, extract_pkgname_from_cache};
pub use scan::*;
