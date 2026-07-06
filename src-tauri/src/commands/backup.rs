use log::info;              // 日志记录
use tauri::State;           // Tauri 状态管理

use crate::backup;          // 备份模块
use crate::AppState;        // 应用状态

/// 执行备份操作
/// @param _state - Tauri 应用状态（当前未使用）
/// @param backup_path - 备份目标目录路径
/// @returns 备份结果（复制文件数和清理旧版本数）
#[tauri::command]
pub async fn run_backup(_state: State<'_, AppState>, backup_path: String) -> Result<backup::BackupResult, String> {
    info!("正在执行备份到: {}", backup_path);
    let config = backup::BackupConfig {
        cache_path: String::new(), // 缓存路径由备份模块内部处理
        backup_path,
    };
    let result = backup::run_backup(&config).await.map_err(|e| e.to_string())?;
    info!("备份完成: 已复制={}, 已清理={}", result.copied, result.removed);
    Ok(result)
}
