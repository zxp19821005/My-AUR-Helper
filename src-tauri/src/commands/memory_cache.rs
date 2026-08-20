/**
 * memory_cache.rs - 内存缓存管理命令
 *
 * 提供缓存运行状态查询、立即写盘、清空缓存三个命令，供前端设置页
 * 「内存缓存管理」分区调用。
 */
use log::{debug, info};
use tauri::State;

use crate::errors::AppResult;
use crate::models::MemoryCacheStats;
use crate::AppState;

/// 获取内存缓存运行状态（配置 + 各域状态）
#[tauri::command]
pub async fn get_memory_cache_stats(state: State<'_, AppState>) -> AppResult<MemoryCacheStats> {
    debug!("正在获取内存缓存状态");
    let cache = state.memory_cache.lock()?;
    let stats = cache.stats();
    info!(
        "内存缓存状态: 启用={}, 域数={}, 条目总数={}",
        stats.enabled,
        stats.domains.len(),
        stats.total_entries
    );
    Ok(stats)
}

/// 立即将脏缓存写盘
/// @returns 实际写入的缓存域数量
#[tauri::command]
pub async fn flush_memory_cache(state: State<'_, AppState>) -> AppResult<usize> {
    info!("正在立即写盘内存缓存");
    let mut cache = state.memory_cache.lock()?;
    let written = cache.flush()?;
    info!("内存缓存写盘完成: {} 个域", written);
    Ok(written)
}

/// 清空内存缓存与磁盘缓存文件
#[tauri::command]
pub async fn clear_memory_cache(state: State<'_, AppState>) -> AppResult<()> {
    info!("正在清空内存缓存");
    let mut cache = state.memory_cache.lock()?;
    cache.clear()?;
    info!("内存缓存已清空");
    Ok(())
}
