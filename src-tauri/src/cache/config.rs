/**
 * config.rs - 内存缓存配置
 *
 * 从 settings 表读取缓存相关配置（启用开关 / 条目上限 / 有效期 / 写盘周期 / 写入目录），
 * 解析失败时回退默认值，保证任何设置缺失/非法都不会导致缓存模块不可用。
 */
use std::path::PathBuf;

use crate::db::Database;

/// 内存缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 是否启用内存缓存（关闭后命令层直接走 DB，行为与未引入缓存一致）
    pub enabled: bool,
    /// 缓存条目上限（LRU 淘汰；至少为 1）
    pub max_entries: usize,
    /// 缓存有效期（秒），0 表示永不过期
    pub ttl_secs: i64,
    /// 自动写盘周期（秒），0 表示关闭定时写（仅退出时写）
    pub write_interval_secs: u64,
    /// 缓存写入目录
    pub dir: PathBuf,
}

/// 各配置项在 settings 表中的键名（与 seed.rs 默认值保持一致）
pub const KEY_ENABLED: &str = "memory_cache_enabled";
pub const KEY_SIZE: &str = "memory_cache_size";
pub const KEY_TTL: &str = "memory_cache_ttl";
pub const KEY_WRITE_INTERVAL: &str = "memory_cache_write_interval";
pub const KEY_DIR: &str = "memory_cache_dir";

/// 默认缓存写入目录：~/.config/com.zxp19821005.aur-helper/cache
pub fn default_cache_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.zxp19821005.aur-helper")
        .join("cache")
}

impl CacheConfig {
    /// 从数据库读取缓存配置（解析失败回退默认值）
    /// @param db - 数据库连接
    /// @returns 缓存配置
    pub fn from_db(db: &Database) -> Self {
        let get = |key: &str, default: &str| -> String {
            db.get_setting(key)
                .ok()
                .flatten()
                .map(|s| s.value)
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| default.to_string())
        };

        let enabled = get(KEY_ENABLED, "true") == "true";
        // 条目上限至少 1（0 视为非法，回退 1），避免 LRU 淘汰后缓存完全失效
        let max_entries = get(KEY_SIZE, "100").parse::<usize>().unwrap_or(100).max(1);
        let ttl_secs = get(KEY_TTL, "300").parse::<i64>().unwrap_or(300).max(0);
        let write_interval_secs = get(KEY_WRITE_INTERVAL, "60")
            .parse::<u64>()
            .unwrap_or(60);
        let dir = expand_dir(&get(KEY_DIR, ""));

        Self {
            enabled,
            max_entries,
            ttl_secs,
            write_interval_secs,
            dir,
        }
    }

    /// 当前时间戳（Unix 秒）
    pub fn now_ts() -> i64 {
        chrono::Utc::now().timestamp()
    }
}

/// 展开缓存目录：空值或 `~` 前缀展开为默认/家目录路径
/// @param raw - 设置中的原始目录字符串
/// @returns 展开后的绝对路径
fn expand_dir(raw: &str) -> PathBuf {
    if raw.is_empty() {
        return default_cache_dir();
    }
    if raw == "~" {
        return dirs::home_dir().unwrap_or_else(default_cache_dir);
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(raw)
}
