/**
 * logs.rs - 日志管理命令
 *
 * 提供日志的查询和清理功能
 */
use log::{debug, info};
use tauri::State;

use crate::errors::AppResult;
use crate::models::LogEntry;
use crate::AppState;

/// 获取日志列表
#[tauri::command]
pub async fn get_logs(state: State<'_, AppState>, limit: Option<i64>) -> AppResult<Vec<LogEntry>> {
    debug!("正在获取日志 (limit={:?})", limit);
    let db = state.db.lock()?;
    db.get_logs(limit.unwrap_or(100))
}

/// 清空所有日志
#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>) -> AppResult<()> {
    info!("正在清空所有日志");
    let db = state.db.lock()?;
    db.clear_logs()
}
