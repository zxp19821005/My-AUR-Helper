/**
 * cache_backup/subdirectory.rs - 备份缓存包到指定子目录
 *
 * 不检查备份表是否存在，直接复制到指定子目录并插入备份记录。
 */
use log::info;
use tauri::State;

use crate::commands::fileops::cache_dirs::{extract_pkgname_from_cache, find_cache_file, get_cache_dirs};
use crate::errors::AppResult;
use crate::models::BackupSoftware;
use crate::AppState;

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
