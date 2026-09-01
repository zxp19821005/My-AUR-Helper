/**
 * scan.rs - 包文件扫描模块
 *
 * 提供 .pkg.tar.zst 包文件扫描功能
 * 已移除未使用的目录扫描命令（scan_directory、scan_directory_recursive），
 * 这些命令存在路径遍历风险且未被前端使用
 *
 * 安全修复 (R8): scan_pkg_files_cmd 仅允许扫描数据库配置的可信缓存目录
 */
use log::info;
use tauri::State;
use tokio::fs;

use crate::errors::AppResult;
use crate::AppState;

// ════════════════════════════════════════════════════════════
// 数据结构
// ════════════════════════════════════════════════════════════

/// .pkg.tar.zst 包文件信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct PkgFileInfo {
    pub filename: String,
    /// 文件绝对路径（递归扫描时含子目录，用于后续定位与复制）
    pub full_path: String,
    pub name: String,
    pub epoch: Option<String>,
    pub version: String,
    pub pkgrel: String,
    pub arch: String,
}

// ════════════════════════════════════════════════════════════
// .pkg.tar.zst 包文件扫描
// ════════════════════════════════════════════════════════════

/// 递归扫描指定目录（含子目录）中的 .pkg.tar.zst 包文件
///
/// 使用显式栈进行深度优先遍历；通过 `file_type()` 判断目录类型，
/// 对符号链接目录返回 false，从而天然排除符号链接、避免死循环。
/// 无权限访问的分支会被跳过而不中断整体扫描。
///
/// @param directory - 要扫描的目录路径（应为绝对路径）
pub async fn scan_pkg_files(directory: &str) -> AppResult<Vec<PkgFileInfo>> {
    scan_pkg_files_inner(std::path::Path::new(directory)).await
}

/// 内部实现：对已验证的 Path 进行扫描
async fn scan_pkg_files_inner(directory: &std::path::Path) -> AppResult<Vec<PkgFileInfo>> {
    let mut result = Vec::new();
    let mut stack: Vec<std::path::PathBuf> = vec![directory.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let mut entries = match fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => continue, // 无权限或目录不存在，跳过该分支
        };

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            // 使用 file_type 避免跟随符号链接导致无限递归
            let ft = entry.file_type().await?;
            if ft.is_dir() {
                // file_type().is_dir() 对符号链接为 false，天然排除符号链接目录
                stack.push(path);
            } else if ft.is_file() {
                let full_path = path.to_string_lossy().to_string();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if let Some(mut pkg) = parse_pkg_filename(filename) {
                        pkg.full_path = full_path;
                        result.push(pkg);
                    }
                }
            }
        }
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

/// 解析 .pkg.tar.zst 文件名
fn parse_pkg_filename(filename: &str) -> Option<PkgFileInfo> {
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
            Some(ver_part[..pos].to_string()),
            ver_part[pos + 1..].to_string(),
        )
    } else {
        (None, ver_part)
    };

    Some(PkgFileInfo {
        filename: filename.to_string(),
        full_path: String::new(),
        name,
        epoch,
        version,
        pkgrel,
        arch,
    })
}

// ════════════════════════════════════════════════════════════
// Tauri 命令
// ════════════════════════════════════════════════════════════

/// 扫描 .pkg.tar.zst 包文件
///
/// 安全修复 (R8): 路径必须与数据库配置的缓存目录之一规范后完全匹配或在其子目录下，
/// 防止任意目录递归扫描（如扫描 /etc/shadow 所在目录）。
#[tauri::command]
pub async fn scan_pkg_files_cmd(
    state: State<'_, AppState>,
    directory: String,
) -> AppResult<Vec<PkgFileInfo>> {
    // R8: 获取所有缓存目录
    let cache_dirs = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        crate::commands::fileops::cache_dirs::get_cache_dirs(&db)?
    };

    // R8: 路径必须是绝对路径
    let request_path = std::path::Path::new(&directory);
    if !request_path.is_absolute() {
        return Err(crate::errors::AppError::InvalidInput(format!(
            "扫描路径必须是绝对路径: {}",
            directory
        )));
    }

    // R8: canonicalize 解析真实路径
    let canon_request = std::fs::canonicalize(request_path)
        .map_err(|e| crate::errors::AppError::InvalidInput(format!("无法访问路径 {}: {}", directory, e)))?;

    // R8: 逐一与缓存目录比较（均先 canonicalize）
    let mut target_dir: Option<std::path::PathBuf> = None;
    for cache_dir in &cache_dirs {
        let canon_cache = match std::fs::canonicalize(&cache_dir.path) {
            Ok(p) => p,
            Err(_) => continue, // 缓存目录不存在或无权限，跳过
        };
        // 请求路径必须完全等于缓存目录，或在其直接子目录下（最多一层）
        if canon_request == canon_cache
            || (canon_request.starts_with(&canon_cache)
                && canon_request
                    .strip_prefix(&canon_cache)
                    .is_ok_and(|s| s.components().count() == 1))
        {
            target_dir = Some(canon_cache);
            break;
        }
    }

    let target = target_dir.ok_or_else(|| {
        crate::errors::AppError::InvalidInput(format!(
            "扫描路径不在允许的缓存目录列表中: {}",
            directory
        ))
    })?;

    info!("扫描包文件: {} (规范路径: {})", directory, target.display());
    scan_pkg_files_inner(&target).await
}
