/**
 * metadata.rs - AUR 全量元数据归档（批量版本基线）编排入口
 *
 * 功能：
 * - 一次请求拉取 packages-meta-ext-v1.json.gz，按包名建立索引，
 *   供批量同步在本地过滤，避免逐包打 AUR RPC v5（4000 req/day/IP 限流 + Anubis）
 * - 归档不可用时由 get_packages_info_smart 自动降级到 RPC v5 批量查询
 *
 * 工作流程（本文件仅保留编排，细节下沉到子模块）：
 * 1. cache::read_fresh_cache 命中 TTL 缓存 → 直接 parse::filter_by_names
 * 2. download::download_archive_with_retry 下载（重试 / 304 / 校验器）
 * 3. parse::parse_archive 在 spawn_blocking 中解压解析 → 写缓存 → 过滤
 * 4. download.rs 承担长超时 client 与重试循环；cache.rs 承担索引/校验器读写
 *
 * 为什么用 ext 版：
 * - packages-meta-v1.json.gz 是 type=search 格式，缺 Depends / MakeDepends /
 *   OptDepends / License；本项目 parse_aur_fields 依赖这些字段。
 *
 * 为什么不新增 Rust 依赖：
 * - gzip 解压由系统 `gzip -dc` 在 spawn_blocking 中完成，Cargo.toml 保持零改动。
 */
mod cache;
mod download;
mod parse;

#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::Instant;

use log::{info, warn};
use reqwest::Client;
use serde_json::Value;

use crate::errors::{AppError, AppResult};

use download::ArchiveFetch;

/// 从 AUR 元数据归档中按包名批量取出完整信息
///
/// 命中 TTL 缓存时零网络请求；归档未变化（304）时复用索引；
/// 否则下载并解压一次，之后写入缓存供后续同步复用。
///
/// # 参数
/// - `client`: 已配置 User-Agent 的 HTTP 客户端（归档下载实际用独立长超时 client）
/// - `pkgnames`: 需要取出的包名列表
///
/// # 返回
/// - `Ok(Vec<Value>)`: 与 RPC v5 `info` 结果形状一致的包信息列表
/// - `Err(AppError)`: 网络失败、被反爬拦截、解压或解析失败（调用方应降级）
pub async fn get_packages_from_archive(
    _client: &Client,
    pkgnames: &[String],
) -> AppResult<Vec<Value>> {
    if pkgnames.is_empty() {
        return Ok(Vec::new());
    }
    if let Some(index) = cache::read_fresh_cache() {
        info!("[AUR 元数据归档] 命中进程内缓存（TTL 300 秒），本次无需网络请求");
        return Ok(parse::filter_by_names(&index, pkgnames));
    }

    let started = Instant::now();
    match download::download_archive_with_retry().await? {
        ArchiveFetch::NotModified => {
            if let Some(index) = cache::touch_cache() {
                info!("[AUR 元数据归档] 304 Not Modified，复用本地索引");
                return Ok(parse::filter_by_names(&index, pkgnames));
            }
            Err(AppError::NetworkError(
                "归档返回 304 但本地缓存已失效，请降级到 RPC".to_string(),
            ))
        }
        ArchiveFetch::Fresh {
            bytes,
            etag,
            last_modified,
        } => {
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
            cache::write_cache(Arc::clone(&index), etag, last_modified);
            Ok(parse::filter_by_names(&index, pkgnames))
        }
    }
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
