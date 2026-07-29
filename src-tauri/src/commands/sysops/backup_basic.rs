/**
 * backup_basic.rs - 备份管理基础命令
 *
 * 功能：
 * - list_backup_software: 列出所有备份记录
 * - clear_backup_software: 清空备份表
 * - delete_backup: 删除单个备份记录（及对应文件）
 */
use log::info;
use tauri::State;
use tokio::fs;

use crate::errors::AppResult;
use crate::models::BackupSoftwareEntry;
use crate::AppState;

/// 列出所有备份记录（含软件包名称）
#[tauri::command]
pub async fn list_backup_software(
    state: State<'_, AppState>,
) -> AppResult<Vec<BackupSoftwareEntry>> {
    let db = state
        .db
        .lock()
        .map_err(|e| crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
    db.get_all_backup_entries()
}

/// 清空备份表
#[tauri::command]
pub async fn clear_backup_software(state: State<'_, AppState>) -> AppResult<usize> {
    let db = state
        .db
        .lock()
        .map_err(|e| crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
    let count = db.clear_backup_software()?;
    info!("[备份管理] 已清空备份表，删除 {} 条记录", count);
    Ok(count)
}

/// 删除单个备份记录（及对应文件）
#[tauri::command]
pub async fn delete_backup(
    state: State<'_, AppState>,
    id: i64,
    _backup_path: String,
) -> AppResult<()> {
    let record = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        let entries = db.get_all_backup_software()?;
        entries.iter().find(|e| e.id == Some(id)).cloned()
    };

    if let Some(entry) = record {
        let file_path = std::path::Path::new(&entry.full_path);
        if file_path.exists() {
            fs::remove_file(file_path).await.map_err(|e| {
                crate::errors::AppError::FileOperation(format!("删除文件失败: {}", e))
            })?;
        }

        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        db.delete_backup_software(id)?;

        info!("[备份管理] 已删除备份: {}", entry.full_path);
    }

    Ok(())
}
