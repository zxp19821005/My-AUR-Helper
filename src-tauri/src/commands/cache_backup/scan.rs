/**
 * scan.rs - 缓存目录扫描和表管理命令
 *
 * 功能：
 * - clear_cache_software: 清空 cache_software 表
 * - list_cache_software: 直接从 cache_software 表读取所有记录（用于页面初始加载）
 * - scan_all_cache_dirs: 扫描所有启用的缓存目录，写入 cache_software 表
 */
use tauri::State;

use super::dirs::get_cache_dirs;
use crate::errors::AppResult;
use crate::models::{CacheSoftware, CacheSoftwareEntry};
use crate::AppState;

/// 清空 cache_software 表
///
/// @returns 删除的记录数
#[tauri::command]
pub fn clear_cache_software(state: State<'_, AppState>) -> AppResult<usize> {
    log::info!("[缓存管理] 清空 cache_software 表");
    let db = state
        .db
        .lock()
        .map_err(|e| crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
    let count = db.clear_cache_software()?;
    log::info!("[缓存管理] 已清空 cache_software 表，删除 {} 条记录", count);
    Ok(count)
}

/// 直接从 cache_software 表读取所有缓存记录（用于页面初始加载）
///
/// 页面打开时调用，不需要扫描磁盘，直接读取数据库存量数据
///
/// @returns 缓存记录列表（包含解析后的包名）
#[tauri::command]
pub fn list_cache_software(state: State<'_, AppState>) -> AppResult<Vec<CacheSoftwareEntry>> {
    log::debug!("[缓存管理] 从 cache_software 表读取所有记录");
    let db = state
        .db
        .lock()
        .map_err(|e| crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
    let entries = db.get_all_cache_entries()?;
    log::debug!("[缓存管理] 从 cache_software 表读取到 {} 条记录", entries.len());
    Ok(entries)
}

/// 扫描所有启用的缓存目录
///
/// 扫描流程：
/// 1. 从 settings 读取所有启用的缓存目录（展开 ~ 路径）
/// 2. 清空 cache_software 表旧记录
/// 3. 逐个目录扫描 .pkg.tar.zst 文件
/// 4. 将扫描结果写入 cache_software 表
/// 5. 返回扫描结果列表
#[tauri::command]
pub async fn scan_all_cache_dirs(
    state: State<'_, AppState>,
) -> AppResult<Vec<crate::commands::scan::PkgFileInfo>> {
    log::info!("[缓存管理] 开始扫描所有缓存目录");

    let dirs = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        get_cache_dirs(&db)?
    };

    log::info!("[缓存管理] 找到 {} 个启用的缓存目录", dirs.len());

    let mut all_packages: Vec<(String, Vec<crate::commands::scan::PkgFileInfo>)> = Vec::new();
    for dir in dirs {
        let path = std::path::Path::new(&dir.path);
        if !path.exists() {
            log::warn!("[缓存管理] 缓存目录不存在: {} ({})", dir.name, dir.path);
            continue;
        }

        log::info!("[缓存管理] 正在扫描目录: {} ({})", dir.name, dir.path);

        match crate::commands::scan::scan_pkg_files(&dir.path).await {
            Ok(mut packages) => {
                log::info!(
                    "[缓存管理] 扫描 {} 完成，找到 {} 个包",
                    dir.name,
                    packages.len()
                );
                for pkg in packages.iter_mut() {
                    pkg.source_dir = Some(dir.name.clone());
                }
                all_packages.push((dir.path.clone(), packages));
            }
            Err(e) => {
                log::error!("[缓存管理] 扫描 {} 失败: {}", dir.name, e);
            }
        }
    }

    let total_count: usize = all_packages.iter().map(|(_, p)| p.len()).sum();
    log::info!(
        "[缓存管理] 所有缓存目录扫描完成，共找到 {} 个包，开始写入数据库",
        total_count
    );

    {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;

        db.clear_cache_software()?;
        log::info!("[缓存管理] 已清空旧的 cache_software 记录");

        let mut inserted = 0;
        for (cache_directory, packages) in &all_packages {
            for pkg in packages {
                let epoch: i64 = pkg
                    .epoch
                    .as_ref()
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);
                let cs = CacheSoftware {
                    id: None,
                    software_id: 0,
                    filename: pkg.filename.clone(),
                    name: pkg.name.clone(),
                    epoch,
                    version: pkg.version.clone(),
                    pkgrel: pkg.pkgrel.clone(),
                    arch: pkg.arch.clone(),
                    size: pkg.size as i64,
                    source_dir: pkg.source_dir.clone(),
                    cache_directory: cache_directory.clone(),
                };
                match db.insert_cache_software(&cs) {
                    Ok(_) => inserted += 1,
                    Err(e) => log::error!(
                        "[缓存管理] 写入 cache_software 失败 {}: {}",
                        pkg.filename,
                        e
                    ),
                }
            }
        }
        log::info!("[缓存管理] 已写入 {} 条记录到 cache_software 表", inserted);
    }

    let result: Vec<crate::commands::scan::PkgFileInfo> = all_packages
        .into_iter()
        .flat_map(|(_, packages)| packages)
        .collect();
    log::info!("[缓存管理] 扫描任务完成，共返回 {} 个包", result.len());
    Ok(result)
}
