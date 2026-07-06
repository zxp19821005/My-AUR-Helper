/**
 * logs.rs - 日志管理命令
 *
 * 提供日志的查询和清理功能
 */
use log::{info, debug};     // 日志记录
use tauri::State;           // Tauri 状态管理

use crate::models::LogEntry; // 日志条目模型
use crate::AppState;         // 应用状态

/// 获取日志列表
/// @param state - Tauri 应用状态（包含数据库连接）
/// @param limit - 返回的最大日志数量，默认 100
/// @returns 日志条目列表（按创建时间降序）
#[tauri::command]
pub async fn get_logs(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<LogEntry>, String> {
    debug!("正在获取日志 (limit={:?})", limit);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_logs(limit.unwrap_or(100)) // 默认返回最近 100 条
        .map_err(|e| e.to_string())
}

/// 清空所有日志
/// @param state - Tauri 应用状态
#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>) -> Result<(), String> {
    info!("正在清空所有日志");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_logs().map_err(|e| e.to_string())
}
