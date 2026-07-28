/**
 * cache_dirs.rs - 缓存目录管理命令
 *
 * 功能：
 * - list_cache_dirs: 获取所有缓存目录配置
 * - add_cache_dir: 添加缓存目录
 * - update_cache_dir: 更新缓存目录
 * - delete_cache_dir: 删除缓存目录
 * - scan_all_cache_dirs: 扫描所有启用的缓存目录
 */
use log::info;
use tauri::State;

use crate::errors::AppResult;
use crate::models::CacheDir;
use crate::AppState;

/// 获取所有缓存目录配置
#[tauri::command]
pub async fn list_cache_dirs(state: State<'_, AppState>) -> AppResult<Vec<CacheDir>> {
    let db = state.db.lock().map_err(|e| {
        crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
    })?;
    db.get_all_cache_dirs()
}

/// 添加缓存目录
#[tauri::command]
pub async fn add_cache_dir(
    state: State<'_, AppState>,
    name: String,
    path: String,
    is_enabled: bool,
) -> AppResult<i64> {
    let db = state.db.lock().map_err(|e| {
        crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
    })?;

    // 获取当前最大排序值
    let dirs = db.get_all_cache_dirs()?;
    let max_order = dirs.iter().map(|d| d.sort_order).max().unwrap_or(0);

    let cache_dir = CacheDir {
        id: None,
        name,
        path,
        is_enabled,
        sort_order: max_order + 1,
    };

    let id = db.insert_cache_dir(&cache_dir)?;
    info!("[缓存管理] 添加缓存目录: id={}", id);
    Ok(id)
}

/// 更新缓存目录
#[tauri::command]
pub async fn update_cache_dir(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    path: String,
    is_enabled: bool,
) -> AppResult<()> {
    let db = state.db.lock().map_err(|e| {
        crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
    })?;

    let cache_dir = CacheDir {
        id: Some(id),
        name,
        path,
        is_enabled,
        sort_order: 0,
    };

    db.update_cache_dir(&cache_dir)?;
    info!("[缓存管理] 更新缓存目录: id={}", id);
    Ok(())
}

/// 删除缓存目录
#[tauri::command]
pub async fn delete_cache_dir(
    state: State<'_, AppState>,
    id: i64,
) -> AppResult<()> {
    let db = state.db.lock().map_err(|e| {
        crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
    })?;
    db.delete_cache_dir(id)?;
    info!("[缓存管理] 删除缓存目录: id={}", id);
    Ok(())
}

/// 扫描所有启用的缓存目录，返回扫描结果
#[tauri::command]
pub async fn scan_all_cache_dirs(
    state: State<'_, AppState>,
) -> AppResult<Vec<crate::commands::scan::PkgFileInfo>> {
    let dirs = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        db.get_enabled_cache_dirs()?
    };

    let mut all_packages = Vec::new();
    for dir in &dirs {
        let path = std::path::Path::new(&dir.path);
        if !path.exists() {
            log::warn!("[缓存管理] 缓存目录不存在: {}", dir.path);
            continue;
        }

        match crate::commands::scan::scan_pkg_files(&dir.path).await {
            Ok(mut packages) => {
                for pkg in &mut packages {
                    pkg.source_dir = Some(dir.name.clone());
                }
                log::info!("[缓存管理] 扫描 {} 完成，找到 {} 个包", dir.name, packages.len());
                all_packages.append(&mut packages);
            }
            Err(e) => {
                log::error!("[缓存管理] 扫描 {} 失败: {}", dir.name, e);
            }
        }
    }

    info!("[缓存管理] 所有缓存目录扫描完成，共找到 {} 个包", all_packages.len());
    Ok(all_packages)
}

/// 清空缓存表
#[tauri::command]
pub async fn clear_cache_software(state: State<'_, AppState>) -> AppResult<usize> {
    let db = state.db.lock().map_err(|e| {
        crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
    })?;
    db.clear_cache_software()
}
