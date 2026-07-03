use log::info;     // 日志记录

use crate::scan;   // 扫描模块

/// 扫描系统缓存目录
/// 检测 paru、yay 和 pacman 的缓存目录
/// @returns 检测到的缓存目录列表（包含类型、路径、包数量和总大小）
#[tauri::command]
pub async fn scan_caches() -> Result<Vec<scan::DetectedCache>, String> {
    info!("Scanning system caches");
    let result = scan::detect_system_caches().await.map_err(|e| e.to_string())?;
    info!("Found {} cache directories", result.len());
    Ok(result)
}

/// 扫描指定目录内容
/// @param path - 要扫描的目录路径
/// @returns 目录条目列表（包含名称、类型、大小和修改时间）
#[tauri::command]
pub async fn scan_directory(path: String) -> Result<Vec<scan::DirEntry>, String> {
    info!("Scanning directory: {}", path);
    scan::list_directory(&path).await.map_err(|e| e.to_string())
}
