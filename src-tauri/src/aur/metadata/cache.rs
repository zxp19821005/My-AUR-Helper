/**
 * aur/metadata/cache.rs - AUR 元数据归档的进程内索引缓存
 *
 * 功能：
 * - 保存按包名建立的精简索引（Arc 共享，读取方廉价克隆）
 * - 记录 ETag / Last-Modified 条件请求校验器与抓取时刻
 * - 提供 TTL 命中读取、校验器读取、304 触碰刷新、新索引写入四类操作
 *
 * 工作流程：
 * 1. 全局 ARCHIVE_CACHE 用 OnceLock + Mutex 惰性初始化，锁中毒时恢复内部值
 * 2. read_fresh_cache 仅在 TTL（300 秒，与归档更新间隔对齐）内返回索引
 * 3. touch_cache 在 304 Not Modified 时刷新 TTL 并复用原索引
 *
 * 适用场景：批量同步时避免对同一归档反复下载/解析，降低网络与 CPU 开销。
 */
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::Value;

/// 归档官方更新间隔约 5 分钟，本地缓存 TTL 与之对齐
pub(super) const CACHE_TTL: Duration = Duration::from_secs(300);

/// 归档索引的进程内缓存条目
struct ArchiveCache {
    /// pkgname -> 精简后的 RPC 风格包信息
    index: Arc<HashMap<String, Value>>,
    /// 索引写入时刻
    fetched_at: Instant,
    /// 上次响应的 ETag
    etag: Option<String>,
    /// 上次响应的 Last-Modified
    last_modified: Option<String>,
}

/// 全局缓存；Arc 让读取方廉价克隆，避免复制全量索引
static ARCHIVE_CACHE: OnceLock<Mutex<Option<ArchiveCache>>> = OnceLock::new();

/// 取得全局缓存锁；锁中毒时恢复内部值而非 panic
fn cache_lock() -> std::sync::MutexGuard<'static, Option<ArchiveCache>> {
    ARCHIVE_CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 读取 TTL 内的缓存索引
pub(super) fn read_fresh_cache() -> Option<Arc<HashMap<String, Value>>> {
    let guard = cache_lock();
    guard
        .as_ref()
        .filter(|entry| entry.fetched_at.elapsed() < CACHE_TTL)
        .map(|entry| Arc::clone(&entry.index))
}

/// 读取上次响应的条件请求校验器
pub(super) fn read_validators() -> (Option<String>, Option<String>) {
    let guard = cache_lock();
    match guard.as_ref() {
        Some(entry) => (entry.etag.clone(), entry.last_modified.clone()),
        None => (None, None),
    }
}

/// 304 Not Modified 时刷新 TTL 并返回复用的索引
pub(super) fn touch_cache() -> Option<Arc<HashMap<String, Value>>> {
    let mut guard = cache_lock();
    guard.as_mut().map(|entry| {
        entry.fetched_at = Instant::now();
        Arc::clone(&entry.index)
    })
}

/// 写入新索引及其条件请求校验器
pub(super) fn write_cache(
    index: Arc<HashMap<String, Value>>,
    etag: Option<String>,
    last_modified: Option<String>,
) {
    let mut guard = cache_lock();
    *guard = Some(ArchiveCache {
        index,
        fetched_at: Instant::now(),
        etag,
        last_modified,
    });
}
