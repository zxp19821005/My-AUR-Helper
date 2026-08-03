/**
 * cache_backup.rs - 缓存包备份操作
 *
 * 功能：
 * - backup_cache_to_existing: 自动比较版本，将缓存中更新的包备份到已有备份记录所在的子目录
 * - backup_cache_to_subdirectory: 将缓存包备份到指定子目录
 */
use log::info;
use tauri::State;

use super::cache_dirs::{extract_pkgname_from_cache, find_cache_file, get_cache_dirs};
use crate::errors::AppResult;
use crate::models::{BackupSoftware, CacheSoftwareEntry};
use crate::versions::{compare_vercmp, VersionComparison};
use crate::AppState;

/// 自动比较版本，将缓存中更新的包备份到已有备份记录所在的子目录
///
/// 工作流程：
/// 1. 扫描所有缓存目录
/// 2. 获取所有备份记录
/// 3. 对于每个备份记录，检查是否有对应的缓存包
/// 4. 如果缓存包版本比备份包版本更新，则复制到备份目录
/// 5. 返回备份结果（成功数量和错误列表）
#[tauri::command]
pub async fn backup_cache_to_existing(
    state: State<'_, AppState>,
    backup_path: String,
) -> AppResult<(usize, Vec<String>)> {
    info!("[缓存备份] 开始自动比较版本并备份新版本");

    let mut success_count = 0;
    let mut errors = Vec::new();

    // 获取所有备份记录，按包名索引
    let backup_entries = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        db.get_all_backup_entries()?
    };

    // 构建 包名 -> 所有备份记录 的映射
    let mut pkg_backup_map = std::collections::HashMap::new();
    for entry in &backup_entries {
        pkg_backup_map
            .entry(entry.pkgname.clone())
            .or_insert_with(Vec::new)
            .push(entry.clone());
    }

    // 扫描所有缓存目录
    let cache_dirs = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        get_cache_dirs(&db)?
    };

    // 扫描所有缓存包
    let mut all_cache_packages: Vec<CacheSoftwareEntry> = Vec::new();
    for dir in &cache_dirs {
        match super::cache_scan::scan_cache_dir(dir).await {
            Ok(packages) => all_cache_packages.extend(packages),
            Err(e) => {
                log::error!("[缓存备份] 扫描缓存目录 {} 失败: {}", dir.name, e);
                errors.push(format!("扫描缓存目录 {} 失败: {}", dir.name, e));
            }
        }
    }

    info!(
        "[缓存备份] 扫描完成，找到 {} 个缓存包，{} 个备份记录",
        all_cache_packages.len(),
        backup_entries.len()
    );

    // 构建 包名 -> 最新缓存包 的映射
    let mut pkg_cache_map = std::collections::HashMap::new();
    for cache_entry in &all_cache_packages {
        let pkgname = cache_entry.pkgname.clone();
        let cache_version = format_cache_version(cache_entry);

        pkg_cache_map
            .entry(pkgname)
            .or_insert_with(|| (cache_entry.clone(), cache_version.clone()));
    }

    // 比较版本并备份
    for (pkgname, cache_entries) in &pkg_cache_map {
        if let Some(backup_list) = pkg_backup_map.get(pkgname) {
            let (cache_entry, cache_version) = cache_entries;

            // 找到备份表中版本最新的记录
            if let Some(latest_backup) = backup_list
                .iter()
                .max_by_key(|entry| format_backup_version(entry))
            {
                let backup_version = format_backup_version(latest_backup);

                // 比较缓存版本和备份版本
                let comparison = compare_vercmp(cache_version, &backup_version);

                if comparison == VersionComparison::GreaterThan {
                    // 缓存版本更新，需要备份
                    let filename = &cache_entry.filename;
                    let cache_file_path = find_cache_file(filename, &cache_dirs).await;

                    match cache_file_path {
                        Some(src_path) => {
                            // 确定目标子目录
                            let subdirectory = latest_backup.subdirectory.as_deref().unwrap_or("");
                            let target_dir = if subdirectory.is_empty() {
                                std::path::PathBuf::from(&backup_path)
                            } else {
                                std::path::PathBuf::from(&backup_path).join(subdirectory)
                            };

                            // 创建目标目录
                            if let Err(e) = tokio::fs::create_dir_all(&target_dir).await {
                                errors.push(format!(
                                    "创建目录失败 {}: {}",
                                    target_dir.display(),
                                    e
                                ));
                                continue;
                            }

                            let target_file = target_dir.join(filename);

                            // 复制文件
                            match tokio::fs::copy(&src_path, &target_file).await {
                                Ok(_) => {
                                    // 插入备份记录
                                    let bs = BackupSoftware {
                                        id: None,
                                        name: pkgname.clone(),
                                        filename: filename.clone(),
                                        epoch: cache_entry.epoch,
                                        pkgver: cache_entry.pkgver.clone(),
                                        pkgrel: cache_entry.pkgrel.clone(),
                                        arch: cache_entry.arch.clone(),
                                        subdirectory: Some(subdirectory.to_string()),
                                        full_path: target_file.to_string_lossy().to_string(),
                                    };
                                    let db = state.db.lock().map_err(|e| {
                                        crate::errors::AppError::DatabaseError(format!(
                                            "获取数据库锁失败: {}",
                                            e
                                        ))
                                    })?;
                                    db.insert_backup_software(&bs)?;
                                    success_count += 1;
                                    info!(
                                        "[缓存备份] 已备份 {} ({} -> {}) 到 {}",
                                        pkgname,
                                        backup_version,
                                        cache_version,
                                        target_dir.display()
                                    );
                                }
                                Err(e) => {
                                    errors.push(format!("复制文件失败 {}: {}", filename, e));
                                }
                            }
                        }
                        None => {
                            errors.push(format!("缓存文件不存在: {}", filename));
                        }
                    }
                } else if comparison == VersionComparison::Equal {
                    // 版本相同，跳过
                    log::debug!("[缓存备份] {} 版本相同 ({})，跳过", pkgname, cache_version);
                } else {
                    // 缓存版本较旧，跳过
                    log::debug!(
                        "[缓存备份] {} 缓存版本较旧 ({} < {})，跳过",
                        pkgname,
                        cache_version,
                        backup_version
                    );
                }
            }
        }
    }

    info!(
        "[缓存备份] 完成: 成功备份 {} 个新版本, 错误 {} 个",
        success_count,
        errors.len()
    );
    Ok((success_count, errors))
}

/// 格式化缓存包版本字符串（用于版本比较）
fn format_cache_version(entry: &CacheSoftwareEntry) -> String {
    if entry.epoch > 0 {
        format!("{}:{}-{}", entry.epoch, entry.pkgver, entry.pkgrel)
    } else {
        format!("{}-{}", entry.pkgver, entry.pkgrel)
    }
}

/// 格式化备份包版本字符串（用于版本比较）
fn format_backup_version(entry: &crate::models::BackupSoftwareEntry) -> String {
    if entry.epoch > 0 {
        format!("{}:{}-{}", entry.epoch, entry.pkgver, entry.pkgrel)
    } else {
        format!("{}-{}", entry.pkgver, entry.pkgrel)
    }
}

/// 将缓存包备份到指定子目录
///
/// 不检查备份表是否存在，直接复制到指定子目录并插入备份记录。
#[tauri::command]
pub async fn backup_cache_to_subdirectory(
    state: State<'_, AppState>,
    filenames: Vec<String>,
    backup_path: String,
    subdirectory: String,
) -> AppResult<(usize, Vec<String>)> {
    info!(
        "[缓存备份] 开始备份 {} 个缓存包到子目录 {}",
        filenames.len(),
        subdirectory
    );

    let mut success_count = 0;
    let mut errors = Vec::new();

    // 获取启用的缓存目录
    let cache_dirs = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        get_cache_dirs(&db)?
    };

    // 确定目标目录
    let target_dir = if subdirectory.is_empty() {
        std::path::PathBuf::from(&backup_path)
    } else {
        std::path::PathBuf::from(&backup_path).join(&subdirectory)
    };

    // 创建目标目录
    if let Err(e) = tokio::fs::create_dir_all(&target_dir).await {
        return Err(crate::errors::AppError::FileOperation(format!(
            "创建目录失败: {}",
            e
        )));
    }

    for filename in &filenames {
        // 找到缓存文件的实际路径
        let cache_file_path = find_cache_file(filename, &cache_dirs).await;
        match cache_file_path {
            Some(src_path) => {
                let target_file = target_dir.join(filename);

                // 复制文件
                match tokio::fs::copy(&src_path, &target_file).await {
                    Ok(_) => {
                        // 插入备份记录
                        let pkgname = extract_pkgname_from_cache(filename).unwrap_or_default();
                        let bs = BackupSoftware {
                            id: None,
                            name: pkgname,
                            filename: filename.clone(),
                            epoch: 0,
                            pkgver: String::new(),
                            pkgrel: String::new(),
                            arch: String::new(),
                            subdirectory: if subdirectory.is_empty() {
                                None
                            } else {
                                Some(subdirectory.clone())
                            },
                            full_path: target_file.to_string_lossy().to_string(),
                        };
                        let db = state.db.lock().map_err(|e| {
                            crate::errors::AppError::DatabaseError(format!(
                                "获取数据库锁失败: {}",
                                e
                            ))
                        })?;
                        db.insert_backup_software(&bs)?;
                        success_count += 1;
                        info!("[缓存备份] 已备份 {} 到 {}", filename, target_dir.display());
                    }
                    Err(e) => {
                        errors.push(format!("复制文件失败 {}: {}", filename, e));
                    }
                }
            }
            None => {
                errors.push(format!("缓存文件不存在: {}", filename));
            }
        }
    }

    info!(
        "[缓存备份] 完成: 成功 {} 个, 失败 {} 个",
        success_count,
        errors.len()
    );
    Ok((success_count, errors))
}
