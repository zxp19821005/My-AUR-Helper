//! backup_dedup.rs - 备份去重逻辑
//!
//! 功能：
//! - collect_pkg_map: 按包名分组所有备份记录
//! - collect_files_to_delete: 比较版本，收集需要删除的旧版本文件
//! - DeduplicateResult: 去重结果结构体
//! - deduplicate_backups: 软件去重（保留最新版本，删除旧版本文件和记录）
use crate::db::Database;
use crate::errors::AppResult;
use log::info;
use tauri::State;
use tokio::fs;

use crate::AppState;

/// 备份去重结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeduplicateResult {
    pub removed_files: usize,
    pub removed_records: usize,
    pub errors: Vec<String>,
}

/// 单条备份包条目（替代 (i64, String, String, String) tuple，消除类型复杂度告警）
#[derive(Clone)]
pub struct BackupPkgEntry {
    /// 备份记录 ID
    pub id: i64,
    /// 文件名
    pub filename: String,
    /// 完整版本字符串（epoch:version-pkgrel）
    pub version: String,
    /// 文件完整路径
    pub full_path: String,
}

/// 按包名分组的备份条目映射（type alias 消除复杂类型重复）
pub type PkgBackupMap = std::collections::HashMap<String, Vec<BackupPkgEntry>>;

/// 解析 .pkg.tar.zst 文件名（提取包名、epoch、版本、pkgrel、架构）
pub fn parse_pkg_filename(filename: &str) -> Option<(String, i64, String, String, String)> {
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

/// 从数据库读取所有备份记录并按包名分组
pub fn collect_pkg_map(db: &Database) -> AppResult<PkgBackupMap> {
    let entries = db.get_all_backup_software()?;
    let mut pkg_map: PkgBackupMap = std::collections::HashMap::new();

    for entry in &entries {
        if let Some(id) = entry.id {
            if let Some((name, epoch, version, pkgrel, _arch)) = parse_pkg_filename(&entry.filename)
            {
                let full_ver = if epoch > 0 {
                    format!("{}:{}-{}", epoch, version, pkgrel)
                } else {
                    format!("{}-{}", version, pkgrel)
                };
                pkg_map.entry(name).or_default().push(BackupPkgEntry {
                    id,
                    filename: entry.filename.clone(),
                    version: full_ver,
                    full_path: entry.full_path.clone(),
                });
            }
        }
    }
    Ok(pkg_map)
}

/// 比较版本，收集需要删除的旧版本文件
pub fn collect_files_to_delete(pkg_map: &PkgBackupMap) -> Vec<BackupPkgEntry> {
    let mut files_to_delete = Vec::new();
    let compare_versions = crate::versions::comparison::compare_versions;

    for entries in pkg_map.values() {
        if entries.len() <= 1 {
            continue;
        }
        let mut best_idx = 0;
        for i in 1..entries.len() {
            let cmp = compare_versions(&entries[i].version, &entries[best_idx].version);
            if cmp == crate::versions::comparison::VersionComparison::GreaterThan {
                best_idx = i;
            }
        }
        for (i, entry) in entries.iter().enumerate() {
            if i != best_idx {
                files_to_delete.push(entry.clone());
                info!(
                    "[备份管理] 标记删除旧版本: {} (最新: {})",
                    entry.filename, entries[best_idx].filename
                );
            }
        }
    }

    info!(
        "[备份管理] 共 {} 个包存在多版本，需删除 {} 个旧文件",
        pkg_map.values().filter(|v| v.len() > 1).count(),
        files_to_delete.len()
    );

    files_to_delete
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

    for entry in &files_to_delete {
        let file_path = std::path::Path::new(&entry.full_path);
        match fs::remove_file(file_path).await {
            Ok(()) => {
                result.removed_files += 1;
                info!("[备份管理] 已删除旧备份文件: {}", entry.full_path);
            }
            Err(e) => {
                result
                    .errors
                    .push(format!("删除文件失败 {}: {}", entry.full_path, e));
            }
        }
    }

    {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;

        for entry in &files_to_delete {
            match db.delete_backup_software(entry.id) {
                Ok(()) => {
                    result.removed_records += 1;
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("删除数据库记录失败 id={}: {}", entry.id, e));
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
