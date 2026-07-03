/**
 * settings.rs - 设置管理命令
 *
 * 提供应用设置的 CRUD 操作
 */
use log::{info, debug};     // 日志记录
use tauri::State;           // Tauri 状态管理

use crate::models::Setting; // 设置模型
use crate::AppState;        // 应用状态

/// 获取所有设置
/// @param state - Tauri 应用状态
/// @returns 所有设置项列表
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Vec<Setting>, String> {
    debug!("Getting all settings");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.get_all_settings().map_err(|e| e.to_string())?;
    info!("Got {} settings", result.len());
    Ok(result)
}

/// 获取单个设置
/// @param state - Tauri 应用状态
/// @param key - 设置键名
/// @returns 可选的设置项
#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<Setting>, String> {
    debug!("Getting setting: {}", key);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_setting(&key).map_err(|e| e.to_string())
}

/// 设置配置值（如果 key 不存在则创建，存在则更新）
/// @param state - Tauri 应用状态
/// @param key - 设置键名
/// @param value - 设置值
#[tauri::command]
pub async fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    info!("Setting {} = {}", key, value);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}
