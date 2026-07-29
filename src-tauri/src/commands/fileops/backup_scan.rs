/**
 * backup_scan.rs - 备份目录扫描和数据库写入
 *
 * 功能：
 * - scan_backup_directory: 扫描备份目录并写入数据库
 * - list_backup_subdirectories: 获取所有不重复的子目录列表
 */
use log::{error, info};
use tauri::State;
use tokio::fs;

use super::backup_dedup::parse_pkg_filename;
use crate::errors::AppResult;
use crate::models::BackupSoftware;
use crate::AppState;

/// 递归扫描目录，收集所有 .pkg.tar.zst 文件
async fn scan_directory_recursive(
    dir: &std::path::Path,
    files: &mut Vec<std::path::PathBuf>,
) -> AppResult<()> {
    let mut entries = fs::read_dir(dir)
        .await
        .map_err(|e| crate::errors::AppError::FileOperation(format!("读取目录失败: {}", e)))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| crate::errors::AppError::FileOperation(format!("读取目录项失败: {}", e)))?
    {
        let path = entry.path();
        if path.is_dir() {
            Box::pin(scan_directory_recursive(&path, files)).await?;
        } else if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            if filename.ends_with(".pkg.tar.zst") {
                files.push(path);
            }
        }
    }
    Ok(())
}

/// 扫描备份目录并写入数据库
#[tauri::command]
pub async fn scan_backup_directory(
    state: State<'_, AppState>,
    backup_path: String,
) -> AppResult<usize> {
    info!("[备份管理] 开始扫描备份目录: {}", backup_path);

    let dir_path = std::path::Path::new(&backup_path);
    if !dir_path.exists() {
        return Err(crate::errors::AppError::FileOperation(format!(
            "备份目录不存在: {}",
            backup_path
        )));
    }

    let mut found_paths = Vec::new();
    scan_directory_recursive(dir_path, &mut found_paths).await?;
    info!("[备份管理] 找到 {} 个备份文件", found_paths.len());

    let mut scanned_files = Vec::new();
    for path in &found_paths {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        if let Some((name, epoch, version, pkgrel, arch)) = parse_pkg_filename(&filename) {
            let subdirectory = path
                .parent()
                .and_then(|p| p.strip_prefix(dir_path).ok())
                .map(|p| p.to_string_lossy().to_string())
                .filter(|s| !s.is_empty());
            let full_path = path.to_string_lossy().to_string();
            scanned_files.push((
                filename,
                name,
                epoch,
                version,
                pkgrel,
                arch,
                subdirectory,
                full_path,
            ));
        }
    }

    let mut count = 0;
    {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;

        for (filename, _name, epoch, version, pkgrel, arch, subdirectory, full_path) in
            &scanned_files
        {
            if let Ok(Some(_existing)) = db.get_backup_software_by_filename(filename) {
                continue;
            }

            let bs = BackupSoftware {
                id: None,
                filename: filename.clone(),
                epoch: *epoch,
                pkgver: version.clone(),
                pkgrel: pkgrel.clone(),
                arch: arch.clone(),
                subdirectory: subdirectory.clone(),
                full_path: full_path.clone(),
                created_at: None,
                updated_at: None,
            };

            match db.insert_backup_software(&bs) {
                Ok(_) => count += 1,
                Err(e) => {
                    error!("[备份管理] 插入备份记录失败 ({}): {}", filename, e);
                }
            }
        }
    }

    info!("[备份管理] 扫描完成，新增 {} 条备份记录", count);
    Ok(count)
}

/// 获取所有不重复的子目录列表
#[tauri::command]
pub async fn list_backup_subdirectories(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    let db = state
        .db
        .lock()
        .map_err(|e| crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
    db.get_backup_subdirectories()
}
