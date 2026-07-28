/**
 * backup.rs - 缓存包备份操作
 *
 * 功能：
 * - backup_cache_to_existing: 将缓存包备份到已有备份记录所在的子目录
 * - backup_cache_to_subdirectory: 将缓存包备份到指定子目录
 */
use log::info;
use tauri::State;

use super::dirs::{find_cache_file, get_cache_dirs, extract_pkgname_from_cache};
use crate::errors::AppResult;
use crate::models::{BackupSoftware};
use crate::AppState;

/// 将缓存包备份到已有备份记录所在的子目录
///
/// 如果软件名在备份表中存在，则将缓存文件复制到该备份记录所在的子目录。
/// 如果不存在，跳过该文件。
#[tauri::command]
pub async fn backup_cache_to_existing(
    state: State<'_, AppState>,
    filenames: Vec<String>,
    backup_path: String,
) -> AppResult<(usize, Vec<String>)> {
    info!(
        "[缓存备份] 开始备份 {} 个缓存包到已有备份位置",
        filenames.len()
    );

    let mut success_count = 0;
    let mut errors = Vec::new();

    // 获取所有备份记录，按包名索引
    let backup_entries = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        db.get_all_backup_entries()?
    };

    // 构建 包名 -> 最新备份记录 的映射
    let mut pkg_backup_map = std::collections::HashMap::new();
    for entry in &backup_entries {
        pkg_backup_map
            .entry(entry.pkgname.clone())
            .or_insert_with(|| entry.clone());
    }

    // 扫描缓存目录获取文件路径
    let cache_dirs = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        get_cache_dirs(&db)?
    };

    for filename in &filenames {
        // 从文件名提取包名
        let pkgname = extract_pkgname_from_cache(filename);
        let pkgname = match pkgname {
            Some(name) => name,
            None => {
                errors.push(format!("无法从文件名解析包名: {}", filename));
                continue;
            }
        };

        // 检查是否在备份表中存在
        if let Some(existing) = pkg_backup_map.get(&pkgname) {
            // 找到缓存文件的实际路径
            let cache_file_path = find_cache_file(filename, &cache_dirs).await;
            match cache_file_path {
                Some(src_path) => {
                    // 确定目标子目录
                    let subdirectory = existing.subdirectory.as_deref().unwrap_or("");
                    let target_dir = if subdirectory.is_empty() {
                        std::path::PathBuf::from(&backup_path)
                    } else {
                        std::path::PathBuf::from(&backup_path).join(subdirectory)
                    };

                    // 创建目标目录
                    if let Err(e) = tokio::fs::create_dir_all(&target_dir).await {
                        errors.push(format!("创建目录失败 {}: {}", target_dir.display(), e));
                        continue;
                    }

                    let target_file = target_dir.join(filename);

                    // 复制文件
                    match tokio::fs::copy(&src_path, &target_file).await {
                        Ok(_) => {
                            // 插入备份记录
                            let bs = BackupSoftware {
                                id: None,
                                filename: filename.clone(),
                                epoch: 0,
                                pkgver: String::new(),
                                pkgrel: String::new(),
                                arch: String::new(),
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
        } else {
            errors.push(format!("{} 不在备份表中，已跳过", filename));
        }
    }

    info!(
        "[缓存备份] 完成: 成功 {} 个, 失败 {} 个",
        success_count,
        errors.len()
    );
    Ok((success_count, errors))
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
                        let bs = BackupSoftware {
                            id: None,
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
