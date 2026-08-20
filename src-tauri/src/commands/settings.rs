/**
 * settings.rs - 设置管理命令
 *
 * 提供应用设置的 CRUD 操作。
 * 读取走内存缓存（CacheDomain::Settings），写库后使缓存失效（下次读自动回源）。
 * 锁序约定：先 memory_cache 锁，后 db 锁，防止与缓存命令死锁。
 */
use log::{debug, info};
use tauri::State;

use crate::cache::CacheDomain;
use crate::errors::AppResult;
use crate::models::Setting;
use crate::AppState;

/// 获取所有设置（走 Settings 缓存，miss 时回源 DB 全量加载并填充缓存）
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<Vec<Setting>> {
    debug!("正在获取所有设置");
    let mut cache = state.memory_cache.lock()?;
    let result: Vec<Setting> = cache.get_or_load(CacheDomain::Settings, || {
        let db = state.db.lock()?;
        db.get_all_settings()
    })?;
    info!("已获取 {} 项设置", result.len());
    Ok(result)
}

/// 获取单个设置（从 Settings 缓存整域查找，未命中键回源时自动重建缓存）
#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> AppResult<Option<Setting>> {
    debug!("正在获取设置: {}", key);
    let mut cache = state.memory_cache.lock()?;
    let all: Vec<Setting> = cache.get_or_load(CacheDomain::Settings, || {
        let db = state.db.lock()?;
        db.get_all_settings()
    })?;
    Ok(all.into_iter().find(|s| s.key == key))
}

/// 设置配置值（如果 key 不存在则创建，存在则更新）
/// 写库成功后使 Settings 缓存失效，保证后续读取一致
/// 注意：不记录 value 内容，防止敏感信息（如代理凭据）泄露到日志
#[tauri::command]
pub async fn set_setting(state: State<'_, AppState>, key: String, value: String) -> AppResult<()> {
    debug!("正在设置配置: key={}, value_len={}", key, value.len());
    let mut cache = state.memory_cache.lock()?;
    let db = state.db.lock()?;
    db.set_setting(&key, &value)?;
    cache.invalidate(CacheDomain::Settings);
    info!("已设置配置: key={}", key);
    Ok(())
}

/// 应用日志轮转设置（运行时更新）
#[tauri::command]
pub async fn apply_log_settings(max_size: u64, max_files: usize) -> AppResult<()> {
    info!(
        "正在更新日志设置: 最大大小={}KB, 最大文件数={}",
        max_size / 1024,
        max_files
    );
    crate::logger::update_log_settings(max_size, max_files);
    Ok(())
}
