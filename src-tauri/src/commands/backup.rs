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

    // 写入数据库
    let mut count = 0;
    {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;

        for (filename, _name, epoch, version, pkgrel, arch, subdirectory, full_path) in
            &scanned_files
        {
            // 检查是否已存在
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
/// 对每个软件包（按 pkgname 分组），保留最新版本的备份文件，删除旧版本
/// 版本比较规则：epoch > version > pkgrel（与 pacman vercmp 一致）
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
    let pkg_map = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        let entries = db.get_all_backup_software()?;

        // 按包名分组: pkgname -> Vec<(id, filename, version_string, full_path)>
        let mut pkg_map: std::collections::HashMap<String, Vec<(i64, String, String, String)>> =
            std::collections::HashMap::new();

        for entry in &entries {
            if let Some(id) = entry.id {
                if let Some((name, epoch, version, pkgrel, _arch)) =
                    parse_pkg_filename(&entry.filename)
                {
                    let full_ver = if epoch > 0 {
                        format!("{}:{}-{}", epoch, version, pkgrel)
                    } else {
                        format!("{}-{}", version, pkgrel)
                    };
                    pkg_map.entry(name).or_default().push((
                        id,
                        entry.filename.clone(),
                        full_ver,
                        entry.full_path.clone(),
                    ));
                }
            }
        }
        pkg_map
    };

    // 第二阶段：对每个包名，用 vercmp 比较版本，收集需要删除的文件
    let mut files_to_delete: Vec<(String, i64, String)> = Vec::new();
    let compare_versions = crate::versions::comparison::compare_versions;
    for (_pkg_name, entries) in &pkg_map {
        if entries.len() <= 1 {
            continue;
        }
        // 找到最新版本（"最大" 的版本）
        let mut best_idx = 0;
        for i in 1..entries.len() {
            let cmp = compare_versions(&entries[i].2, &entries[best_idx].2);
            if cmp == crate::versions::comparison::VersionComparison::GreaterThan {
                best_idx = i;
            }
        }
        // 其余都删除
        for (i, (id, filename, _ver, full_path)) in entries.iter().enumerate() {
            if i != best_idx {
                files_to_delete.push((filename.clone(), *id, full_path.clone()));
                info!(
                    "[备份管理] 标记删除旧版本: {} (最新: {})",
                    filename, entries[best_idx].1
                );
            }
        }
    }

    info!(
        "[备份管理] 共 {} 个包存在多版本，需删除 {} 个旧文件",
        pkg_map.values().filter(|v| v.len() > 1).count(),
        files_to_delete.len()
    );

    // 第三阶段：删除磁盘文件（在锁外完成）
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

    // 第四阶段：删除数据库记录（在新锁内完成）
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
    // 先获取记录信息（含 full_path）
    let record = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        let entries = db.get_all_backup_software()?;
        entries.iter().find(|e| e.id == Some(id)).cloned()
    };

    if let Some(entry) = record {
        // 使用 full_path 字段直接定位文件
        let file_path = std::path::Path::new(&entry.full_path);
        if file_path.exists() {
            fs::remove_file(file_path).await.map_err(|e| {
                crate::errors::AppError::FileOperation(format!("删除文件失败: {}", e))
            })?;
        }

        // 删除数据库记录
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

/// 从包文件获取软件信息（pacman -Qip）
#[tauri::command]
pub async fn get_package_file_info(
    full_path: String,
) -> AppResult<String> {
    let output = tokio::process::Command::new("pacman")
        .args(["-Qip", &full_path])
        .output()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("执行 pacman 失败: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::errors::AppError::SystemCommand(format!(
            "pacman -Qip 失败: {}",
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 检测 sudoers 免密配置是否可用
#[tauri::command]
pub async fn check_sudoers_config() -> AppResult<bool> {
    let output = tokio::process::Command::new("sudo")
        .args(["-n", "true"])
        .output()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("检测 sudo 失败: {}", e)))?;

    Ok(output.status.success())
}

/// 获取 sudoers 配置命令
#[tauri::command]
pub async fn get_sudoers_command() -> AppResult<String> {
    // 获取当前用户名
    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("获取用户名失败: {}", e)))?;

    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(format!(
        "echo \"{} ALL=(ALL) NOPASSWD: /usr/bin/pacman -U *\" | sudo tee /etc/sudoers.d/aur-helper-backup",
        username
    ))
}

/// 安装备份包
#[tauri::command]
pub async fn install_backup_package(
    full_path: String,
) -> AppResult<String> {
    info!("[备份管理] 开始安装备份包: {}", full_path);

    let output = tokio::process::Command::new("sudo")
        .args(["pacman", "-U", "--noconfirm", &full_path])
        .output()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("执行安装失败: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        info!("[备份管理] 安装成功: {}", full_path);
        Ok(stdout)
    } else {
        error!("[备份管理] 安装失败: {} - {}", full_path, stderr);
        Err(crate::errors::AppError::SystemCommand(format!(
            "安装失败:\n{}",
            stderr
        )))
    }
}
