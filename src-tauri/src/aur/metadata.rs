/**
 * metadata.rs - AUR 全量元数据归档（批量版本基线）
 *
 * 功能：
 * - 一次请求拉取 packages-meta-ext-v1.json.gz，按包名建立索引，
 *   供批量同步在本地过滤，避免逐包打 AUR RPC v5（4000 req/day/IP 限流 + Anubis）
 * - 进程内 TTL 缓存（归档约 5 分钟更新）+ ETag / Last-Modified 条件请求
 * - 归档不可用时由 get_packages_info_smart 自动降级到 RPC v5 批量查询
 * - 下载 / 解压 / 解析各阶段均有 INFO 日志与耗时，避免长时间无输出被误判为卡死
 *
 * 为什么用 ext 版：
 * - packages-meta-v1.json.gz 是 type=search 格式，缺 Depends / MakeDepends /
 *   OptDepends / License；本项目 parse_aur_fields 依赖这些字段，用非 ext 版
 *   会把依赖与许可证信息静默清空。
 *
 * 为什么不新增 Rust 依赖：
 * - gzip 解压由系统 `gzip -dc` 在 spawn_blocking 中完成，Cargo.toml 保持零改动；
 *   Arch Linux base 必带 gzip。索引只保留 parse_aur_fields 所需字段以控制内存。
 */
mod parse;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use log::{info, warn};
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::{Client, StatusCode};
use serde_json::Value;

use crate::errors::{AppError, AppResult};

/// AUR 全量元数据归档地址（顶层为 JSON 数组，含依赖与许可证字段）
const ARCHIVE_URL: &str = "https://aur.archlinux.org/packages-meta-ext-v1.json.gz";
/// 归档官方更新间隔约 5 分钟，本地缓存 TTL 与之对齐
const CACHE_TTL: Duration = Duration::from_secs(300);

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
fn read_fresh_cache() -> Option<Arc<HashMap<String, Value>>> {
    let guard = cache_lock();
    guard
        .as_ref()
        .filter(|entry| entry.fetched_at.elapsed() < CACHE_TTL)
        .map(|entry| Arc::clone(&entry.index))
}

/// 读取上次响应的条件请求校验器
fn read_validators() -> (Option<String>, Option<String>) {
    let guard = cache_lock();
    match guard.as_ref() {
        Some(entry) => (entry.etag.clone(), entry.last_modified.clone()),
        None => (None, None),
    }
}

/// 304 Not Modified 时刷新 TTL 并返回复用的索引
fn touch_cache() -> Option<Arc<HashMap<String, Value>>> {
    let mut guard = cache_lock();
    guard.as_mut().map(|entry| {
        entry.fetched_at = Instant::now();
        Arc::clone(&entry.index)
    })
}

/// 写入新索引及其条件请求校验器
fn write_cache(
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

/// 从 AUR 元数据归档中按包名批量取出完整信息
///
/// 命中 TTL 缓存时零网络请求；归档未变化（304）时复用索引；
/// 否则下载并解压一次，之后写入缓存供后续同步复用。
///
/// # 参数
/// - `client`: 已配置 User-Agent 的 HTTP 客户端
/// - `pkgnames`: 需要取出的包名列表
///
/// # 返回
/// - `Ok(Vec<Value>)`: 与 RPC v5 `info` 结果形状一致的包信息列表
/// - `Err(AppError)`: 网络失败、被反爬拦截、解压或解析失败（调用方应降级）
pub async fn get_packages_from_archive(
    client: &Client,
    pkgnames: &[String],
) -> AppResult<Vec<Value>> {
    if pkgnames.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(index) = read_fresh_cache() {
        info!("[AUR 元数据归档] 命中进程内缓存（TTL 300 秒），本次无需网络请求");
        return Ok(parse::filter_by_names(&index, pkgnames));
    }

    // 校验器在 await 之前取出并脱离锁作用域，避免 MutexGuard 跨 await 破坏 Send
    let started = Instant::now();
    info!("[AUR 元数据归档] 开始下载 {}", ARCHIVE_URL);
    let (etag, last_modified) = read_validators();
    let mut request = client.get(ARCHIVE_URL);
    if let Some(ref value) = etag {
        request = request.header(IF_NONE_MATCH, value.as_str());
    }
    if let Some(ref value) = last_modified {
        request = request.header(IF_MODIFIED_SINCE, value.as_str());
    }

    let resp = request.send().await?;
    let status = resp.status();
    info!(
        "[AUR 元数据归档] 响应 HTTP {}，Content-Length: {:?}，连接耗时 {:?}",
        status,
        resp.content_length(),
        started.elapsed()
    );
    if status == StatusCode::NOT_MODIFIED {
        if let Some(index) = touch_cache() {
            info!("[AUR 元数据归档] 304 Not Modified，复用本地索引");
            return Ok(parse::filter_by_names(&index, pkgnames));
        }
        return Err(AppError::NetworkError(
            "归档返回 304 但本地缓存已失效，请降级到 RPC".to_string(),
        ));
    }
    if !status.is_success() {
        return Err(AppError::NetworkError(format!(
            "AUR 元数据归档请求失败: HTTP {}",
            status
        )));
    }

    let new_etag = resp
        .headers()
        .get(ETAG)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let new_last_modified = resp
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = resp.bytes().await?.to_vec();
    info!(
        "[AUR 元数据归档] 下载完成：{} 字节，耗时 {:?}，开始解压与 JSON 解析",
        bytes.len(),
        started.elapsed()
    );

    // 解压 + JSON 解析是 CPU 密集型同步工作，隔离到阻塞线程池
    let index = tokio::task::spawn_blocking(move || parse::parse_archive(&bytes))
        .await
        .map_err(|e| AppError::ParseError(format!("归档解析任务失败: {}", e)))??;

    info!(
        "[AUR 元数据归档] 建立基线索引：{} 个包，本次请求 {} 个包，总耗时 {:?}",
        index.len(),
        pkgnames.len(),
        started.elapsed()
    );
    let index = Arc::new(index);
    write_cache(Arc::clone(&index), new_etag, new_last_modified);
    Ok(parse::filter_by_names(&index, pkgnames))
}

/// 批量获取 AUR 包信息：元数据归档优先，失败自动降级到 RPC v5
///
/// # 参数
/// - `client`: 已配置 User-Agent 的 HTTP 客户端
/// - `pkgnames`: 待查询包名列表
/// - `batch_size`: 降级时单批 RPC 查询的最大包数
/// - `batch_interval`: 降级时批次间隔秒数
/// - `prefer_archive`: 是否优先使用归档（false 则直接走 RPC）
///
/// # 返回
/// - `Ok(Vec<Value>)`: 归档过滤结果或 RPC 批量的合并结果
/// - `Err(AppError)`: 归档与 RPC 均失败
pub async fn get_packages_info_smart(
    client: &Client,
    pkgnames: &[String],
    batch_size: usize,
    batch_interval: u64,
    prefer_archive: bool,
) -> AppResult<Vec<Value>> {
    if !prefer_archive {
        info!("[AUR 元数据归档] 设置项已关闭，直接走 RPC v5 分批查询");
        return super::rpc_batch::get_packages_info(client, pkgnames, batch_size, batch_interval)
            .await;
    }
    match get_packages_from_archive(client, pkgnames).await {
        Ok(results) => Ok(results),
        Err(e) => {
            warn!("[AUR 元数据归档] 不可用（{}），降级到 RPC v5 批量查询", e);
            info!(
                "[AUR 元数据归档] 降级参数：{} 个包 / 每批 {} 个 / 批次间隔 {} 秒",
                pkgnames.len(),
                batch_size,
                batch_interval
            );
            super::rpc_batch::get_packages_info(client, pkgnames, batch_size, batch_interval).await
        }
    }
}
