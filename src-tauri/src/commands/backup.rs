/**
 * backup.rs - 备份管理命令
 *
 * 功能：
 * - list_backup_software: 列出所有备份记录（含软件包名称）
 * - clear_backup_software: 清空备份表
 * - scan_backup_directory: 扫描备份目录并写入数据库
 * - deduplicate_backups: 软件去重（保留最新版本，删除旧文件和记录）
 * - delete_backup: 删除单个备份记录（及对应文件）
 */
use log::{error, info};
use tauri::State;
use tokio::fs;

use crate::errors::AppResult;
use crate::models::{BackupSoftware, BackupSoftwareEntry};
use crate::AppState;

/// 解析 .pkg.tar.zst 文件名
fn parse_pkg_filename(filename: &str) -> Option<(String, i64, String, String, String)> {
    let base = filename.strip_suffix(".pkg.tar.zst")?;
    let parts: Vec<&str> = base.rsplitn(3, '-').collect();
    if parts.len() < 3 {
        return None;
    }
    let arch = parts[0].to_string();
    let pkgrel = parts[1].to_string();
    let name_ver = parts[2];
    let dash_pos = name_ver.rfind('-')?;
    let name = name_ver[..dash_pos].to_string();
    let ver_part = name_ver[dash_pos + 1..].to_string();
    let (epoch, version) = if let Some(pos) = ver_part.find(':') {
        (
            ver_part[..pos].parse::<i64>().unwrap_or(0),
            ver_part[pos + 1..].to_string(),
        )
    } else {
        (0, ver_part)
    };
    Some((name, epoch, version, pkgrel, arch))
}

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

/// 扫描备份目录并写入数据库
///
/// 扫描指定备份目录中的 .pkg.tar.zst 文件，
/// 扫描目录，递归收集所有 .pkg.tar.zst 文件
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

/// 解析文件名并写入 backup_software 表
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

    // 递归扫描目录中的 .pkg.tar.zst 文件
    let mut found_paths = Vec::new();
    scan_directory_recursive(dir_path, &mut found_paths).await?;
    info!("[备份管理] 找到 {} 个备份文件", found_paths.len());

    let mut scanned_files = Vec::new();
    for path in &found_paths {
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        if let Some((name, epoch, _version, pkgrel, arch)) = parse_pkg_filename(&filename) {
            let subdirectory = path
                .parent()
                .and_then(|p| p.strip_prefix(dir_path).ok())
                .map(|p| p.to_string_lossy().to_string())
                .filter(|s| !s.is_empty());
            scanned_files.push((filename, name, epoch, pkgrel, arch, subdirectory));
        }
    }

    // 获取所有软件包名称到 ID 的映射
    let mut count = 0;
    {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;

        let all_software = db.get_all_software()?;
        let mut name_to_id = std::collections::HashMap::new();
        for sw in &all_software {
            name_to_id.insert(sw.pkgname.clone(), sw.software_id.unwrap_or(0));
        }

        for (filename, name, epoch, pkgrel, arch, subdirectory) in &scanned_files {
            // 检查是否已存在
            if let Ok(Some(_existing)) = db.get_backup_software_by_filename(filename) {
                continue;
            }

            // 查找对应的 software_id
            let software_id = name_to_id.get(name).copied();

            let bs = BackupSoftware {
                id: None,
                software_id,
                filename: filename.clone(),
                epoch: *epoch,
                pkgrel: pkgrel.clone(),
                arch: arch.clone(),
                subdirectory: subdirectory.clone(),
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

/// 备份去重结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeduplicateResult {
    /// 删除的文件数
    pub removed_files: usize,
    /// 删除的数据库记录数
    pub removed_records: usize,
    /// 错误信息
    pub errors: Vec<String>,
}

/// 软件去重
///
/// 对每个软件包，保留最新版本的备份文件，删除旧版本的文件和数据库记录
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

    // 第一阶段：获取所有备份记录并按包名分组（在锁内完成）
    let (pkg_map, _entries_map) = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        let entries = db.get_all_backup_software()?;

        let dir_path = std::path::Path::new(&backup_path);
        let mut pkg_map: std::collections::HashMap<
            String,
            Vec<(std::time::SystemTime, String, i64)>,
        > = std::collections::HashMap::new();
        let mut entries_map: std::collections::HashMap<i64, String> =
            std::collections::HashMap::new();

        for entry in &entries {
            if let Some(id) = entry.id {
                entries_map.insert(id, entry.filename.clone());
            }
            let file_path = dir_path.join(&entry.filename);
            if file_path.exists() {
                if let Ok(meta) = std::fs::metadata(&file_path) {
                    if let Ok(mtime) = meta.modified() {
                        let pkg_name = entry
                            .filename
                            .split('-')
                            .take_while(|s| !s.chars().next().is_some_and(|c| c.is_ascii_digit()))
                            .collect::<Vec<_>>()
                            .join("-");
                        pkg_map.entry(pkg_name).or_default().push((
                            mtime,
                            entry.filename.clone(),
                            entry.id.unwrap_or(0),
                        ));
                    }
                }
            }
        }
        (pkg_map, entries_map)
    };

    // 第二阶段：收集需要删除的文件（在锁外完成）
    let mut files_to_delete: Vec<(String, i64)> = Vec::new();
    for versions in pkg_map.values() {
        if versions.len() > 1 {
            let mut sorted = versions.clone();
            sorted.sort_by_key(|b| std::cmp::Reverse(b.0));
            for (_mtime, filename, id) in sorted.iter().skip(1) {
                files_to_delete.push((filename.clone(), *id));
            }
        }
    }

    // 第三阶段：删除磁盘文件（在锁外完成）
    let dir_path = std::path::Path::new(&backup_path);
    for (filename, _id) in &files_to_delete {
        let file_path = dir_path.join(filename);
        match fs::remove_file(&file_path).await {
            Ok(()) => {
                result.removed_files += 1;
                info!("[备份管理] 已删除旧备份文件: {}", filename);
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("删除文件失败 {}: {}", filename, e));
            }
        }
    }

    // 第四阶段：删除数据库记录（在新锁内完成）
    {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;

        for (_filename, id) in &files_to_delete {
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
    backup_path: String,
) -> AppResult<()> {
    // 先获取记录信息
    let filename = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        let entries = db.get_all_backup_software()?;
        entries
            .iter()
            .find(|e| e.id == Some(id))
            .map(|e| e.filename.clone())
    };

    if let Some(filename) = filename {
        // 删除磁盘文件
        let file_path = std::path::Path::new(&backup_path).join(&filename);
        if file_path.exists() {
            fs::remove_file(&file_path).await.map_err(|e| {
                crate::errors::AppError::FileOperation(format!("删除文件失败: {}", e))
            })?;
        }

        // 删除数据库记录
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        db.delete_backup_software(id)?;

        info!("[备份管理] 已删除备份: {}", filename);
    }

    Ok(())
}
