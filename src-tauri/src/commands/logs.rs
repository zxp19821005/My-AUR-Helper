/**
 * logs.rs - 日志管理命令
 *
 * 提供日志文件的读取和清理功能
 */
use log::{debug, info};
use tauri::State;

use crate::errors::AppResult;
use crate::AppState;

/// 获取日志列表（从文件读取，倒序）
#[tauri::command]
pub async fn get_logs(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> AppResult<Vec<crate::logger::FileLogEntry>> {
    debug!("正在获取日志 (limit={:?})", limit);
    let db = state.db.lock()?;
    let log_dir = db
        .get_setting("log_dir")
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_default()
                .join("com.zxp19821005.aur-helper")
                .join("logs")
                .to_string_lossy()
                .to_string()
        });
    let log_prefix = db
        .get_setting("log_prefix")
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "applog".to_string());

    let limit = limit.unwrap_or(500) as usize;
    crate::logger::read_log_entries(&log_dir, &log_prefix, limit)
}

/// 增量读取新增日志（类似 tail -f）
#[tauri::command]
pub async fn get_new_logs(
    state: State<'_, AppState>,
    last_position: u64,
    last_file: String,
    limit: Option<i64>,
) -> AppResult<NewLogsResponse> {
    // 静默读取，避免高频轮询产生大量无效日志
    let db = state.db.lock()?;
    let log_dir = db
        .get_setting("log_dir")
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_default()
                .join("com.zxp19821005.aur-helper")
                .join("logs")
                .to_string_lossy()
                .to_string()
        });
    let log_prefix = db
        .get_setting("log_prefix")
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "applog".to_string());

    let limit = limit.unwrap_or(100) as usize;
    let (entries, new_position, current_file) = crate::logger::read_new_log_entries(
        &log_dir,
        &log_prefix,
        last_position,
        &last_file,
        limit,
    )?;

    Ok(NewLogsResponse {
        entries,
        new_position,
        current_file,
    })
}

/// 增量读取日志的响应
#[derive(Debug, Clone, serde::Serialize)]
pub struct NewLogsResponse {
    pub entries: Vec<crate::logger::FileLogEntry>,
    pub new_position: u64,
    pub current_file: String,
}

/// 清空当天日志
#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>) -> AppResult<()> {
    info!("正在清空当天日志");
    let db = state.db.lock()?;
    let log_dir = db
        .get_setting("log_dir")
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_default()
                .join("com.zxp19821005.aur-helper")
                .join("logs")
                .to_string_lossy()
                .to_string()
        });
    let log_prefix = db
        .get_setting("log_prefix")
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "applog".to_string());

    crate::logger::clear_today_log(&log_dir, &log_prefix)
}
