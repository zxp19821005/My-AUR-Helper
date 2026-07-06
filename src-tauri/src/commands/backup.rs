/**
 * backup.rs - 备份管理命令
 */
use log::info;
use tauri::State;

use crate::backup;
use crate::errors::AppResult;
use crate::AppState;

/// 执行备份操作
#[tauri::command]
pub async fn run_backup(_state: State<'_, AppState>, backup_path: String) -> AppResult<backup::BackupResult> {
    info!("正在执行备份到: {}", backup_path);
    let config = backup::BackupConfig {
        cache_path: String::new(),
        backup_path,
    };
    let result = backup::run_backup(&config).await?;
    info!("备份完成: 已复制={}, 已清理={}", result.copied, result.removed);
    Ok(result)
}
