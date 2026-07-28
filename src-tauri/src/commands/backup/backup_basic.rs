/**
 * backup_basic.rs - 备份管理基础命令
 *
 * 功能：
 * - list_backup_software: 列出所有备份记录
 * - clear_backup_software: 清空备份表
 * - scan_backup_directory: 扫描备份目录并写入数据库
 * - deduplicate_backups: 软件去重（保留最新版本）
 * - delete_backup: 删除单个备份记录（及对应文件）
 * - list_backup_subdirectories: 获取子目录列表
 */
use log::{error, info};
use tauri::State;
use tokio::fs;

use super::dedup::{collect_files_to_delete, collect_pkg_map, parse_pkg_filename, DeduplicateResult};
use crate::errors::AppResult;
use crate::models::{BackupSoftware, BackupSoftwareEntry};
use crate::AppState;

/// 列出所有备份记录（含软件包名称）
#[tauri::command]
pub async fn list_backup_software(
    state: State<'_, AppState>,
) -> AppResult<Vec<BackupSoftwareEntry>> {
    let db = state.db.lock().map_err(|e| {
        crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
    })?;
    db.get_all_backup_entries()
}

/// 清空备份表
#[tauri::command]
pub async fn clear_backup_software(state: State<'_, AppState>) -> AppResult<usize> {
    let db = state.db.lock().map_err(|e| {
        crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
    })?;
    let count = db.clear_backup_software()?;
    info!("[备份管理] 已清空备份表，删除 {} 条记录", count);
    Ok(count)
}

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
                filename, name, epoch, version, pkgrel, arch, subdirectory, full_path,
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

/// 软件去重：保留最新版本，删除旧版本文件和记录
#[tauri::command]
pub async fn deduplicate_backups(
    state: State<'_, AppState>,
    backup_path: String,
) -> AppResult<DeduplicateResult> {
    info!("[备份管理] 开始软件去重: {}", backup_path);

    let mut result = DeduplicateResult {
        removed_files: 0,
        removed_records: 0,
        errors: Vec::new(),
    };

    let pkg_map = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        collect_pkg_map(&db)?
    };

    let files_to_delete = collect_files_to_delete(&pkg_map);

    for (_filename, _id, full_path) in &files_to_delete {
        let file_path = std::path::Path::new(full_path);
        match fs::remove_file(file_path).await {
            Ok(()) => {
                result.removed_files += 1;
                info!("[备份管理] 已删除旧备份文件: {}", full_path);
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("删除文件失败 {}: {}", full_path, e));
            }
        }
    }

    {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;

        for (_filename, id, _full_path) in &files_to_delete {
            match db.delete_backup_software(*id) {
                Ok(()) => {
                    result.removed_records += 1;
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("删除数据库记录失败 id={}: {}", id, e));
                }
            }
        }
    }

    info!(
        "[备份管理] 去重完成: 删除 {} 个文件, {} 条记录",
        result.removed_files, result.removed_records
    );
    Ok(result)
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

/// 获取所有不重复的子目录列表
#[tauri::command]
pub async fn list_backup_subdirectories(
    state: State<'_, AppState>,
) -> AppResult<Vec<String>> {
    let db = state.db.lock().map_err(|e| {
        crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
    })?;
    db.get_backup_subdirectories()
}
