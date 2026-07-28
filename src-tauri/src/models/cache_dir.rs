/**
 * cache_dir.rs - 缓存目录配置模型
 *
 * 定义 AUR 助手缓存目录配置的数据结构
 */
use serde::{Deserialize, Serialize};

/// 缓存目录配置
/// 对应数据库 cache_dirs 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheDir {
    /// 缓存目录 ID
    pub id: Option<i64>,
    /// 缓存目录名称（如 "系统缓存"、"paru 缓存"）
    pub name: String,
    /// 缓存目录路径
    pub path: String,
    /// 是否启用
    pub is_enabled: bool,
    /// 排序顺序
    pub sort_order: i32,
}
