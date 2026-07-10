/**
 * settings.rs - 设置管理命令
 *
 * 提供应用设置的 CRUD 操作
 */
use log::{debug, info};
use tauri::State;

use crate::errors::AppResult;
use crate::models::Setting;
use crate::AppState;

/// 获取所有设置
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<Vec<Setting>> {
    debug!("正在获取所有设置");
    let db = state.db.lock()?;
    let result = db.get_all_settings()?;
    info!("已获取 {} 项设置", result.len());
    Ok(result)
}

/// 获取单个设置
#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> AppResult<Option<Setting>> {
    debug!("正在获取设置: {}", key);
    let db = state.db.lock()?;
    db.get_setting(&key)
}

/// 设置配置值（如果 key 不存在则创建，存在则更新）
#[tauri::command]
pub async fn set_setting(state: State<'_, AppState>, key: String, value: String) -> AppResult<()> {
    info!("正在设置 {} = {}", key, value);
    let db = state.db.lock()?;
    db.set_setting(&key, &value)
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
