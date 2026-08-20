/**
 * memory_cache_stats.rs - 内存缓存统计模型
 *
 * 提供给前端设置页「内存缓存运行状态」展示使用，经 Tauri IPC 序列化为 JSON。
 */
use serde::Serialize;

/// 单个缓存域的运行状态
#[derive(Debug, Clone, Serialize)]
pub struct CacheDomainStats {
    /// 域标识（对应 CacheDomain.file_name()）
    pub domain: String,
    /// 中文展示名
    pub label: String,
    /// 该域是否已加载到内存
    pub loaded: bool,
    /// 数据条目数（JSON 数组长度；非数组为 1）
    pub size: usize,
    /// 缓存创建时间（Unix 秒）
    pub created_at: Option<i64>,
    /// 过期时间（Unix 秒），0 表示永不过期
    pub expires_at: Option<i64>,
    /// 是否支持磁盘持久化
    pub persistent: bool,
    /// 磁盘缓存文件大小（字节），未写入时为 0
    pub file_size: u64,
}

/// 内存缓存整体统计
#[derive(Debug, Clone, Serialize)]
pub struct MemoryCacheStats {
    /// 是否启用内存缓存
    pub enabled: bool,
    /// 缓存条目上限
    pub max_entries: usize,
    /// 缓存有效期（秒），0 表示永不过期
    pub ttl_secs: i64,
    /// 自动写盘周期（秒），0 表示关闭定时写
    pub write_interval_secs: u64,
    /// 缓存写入目录
    pub cache_dir: String,
    /// 各缓存域状态
    pub domains: Vec<CacheDomainStats>,
    /// 全部域条目总数
    pub total_entries: usize,
}
