/**
 * scan.rs - 文件与目录扫描模块
 *
 * 提供通用扫描功能，供备份管理、文件管理等模块调用：
 * - 递归目录扫描（scan_directory_recursive）
 * - .pkg.tar.zst 包文件扫描（scan_pkg_files）
 * - 单层目录列表（scan_directory）
 *
 * 注意：缓存目录路径由设置页面管理，不由本模块自动检测。
 */
use anyhow::Result;
use log::info;
use std::path::Path;
use tokio::fs;

// ════════════════════════════════════════════════════════════
// 数据结构
// ════════════════════════════════════════════════════════════

/// 目录条目信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct DirEntry {
    pub name: String,                // 文件/目录名称
    pub is_dir: bool,                // 是否为目录
}

/// .pkg.tar.zst 包文件信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct PkgFileInfo {
    pub filename: String,        // 完整文件名
    pub name: String,            // 包名
    pub epoch: Option<String>,   // epoch（可选）
    pub version: String,         // 版本号
    pub pkgrel: String,          // 发布号
    pub arch: String,            // 架构
    pub size: u64,               // 文件大小（字节）
}

// ════════════════════════════════════════════════════════════
// 目录扫描
// ════════════════════════════════════════════════════════════

/// 列出指定目录的内容（仅单层，不递归）
/// @param path - 要扫描的目录路径
/// @returns 目录条目列表（按名称排序）
pub async fn list_directory(path: &str) -> Result<Vec<DirEntry>> {
    let mut entries = fs::read_dir(path).await?;
    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        result.push(DirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: file_type.is_dir(),
        });
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

/// 递归扫描目录树，返回所有文件/目录的完整列表
/// @param path - 根目录路径
/// @param max_depth - 最大递归深度（0 表示不限制）
/// @returns 所有层级的目录条目列表
pub async fn list_directory_recursive(path: &str, max_depth: u32) -> Result<Vec<DirEntry>> {
    let mut result = Vec::new();
    scan_recursive_inner(Path::new(path), &mut result, 0, max_depth).await?;
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

/// 递归扫描内部实现
async fn scan_recursive_inner(
    dir: &Path,
    acc: &mut Vec<DirEntry>,
    depth: u32,
    max_depth: u32,
) -> Result<()> {
    if max_depth > 0 && depth >= max_depth {
        return Ok(());
    }
    if !dir.is_dir() {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let is_dir = file_type.is_dir();
        acc.push(DirEntry {
            name: entry.path().to_string_lossy().to_string(),
            is_dir,
        });
        if is_dir {
            Box::pin(scan_recursive_inner(&entry.path(), acc, depth + 1, max_depth)).await?;
        }
    }
    Ok(())
}

// ════════════════════════════════════════════════════════════
// .pkg.tar.zst 包文件扫描
// ════════════════════════════════════════════════════════════

/// 扫描指定目录中的 .pkg.tar.zst 包文件
/// 解析文件名提取包名、epoch、版本、发布号和架构信息
/// @param directory - 要扫描的目录路径
/// @returns 包文件信息列表
pub async fn scan_pkg_files(directory: &str) -> Result<Vec<PkgFileInfo>> {
    let mut entries = fs::read_dir(directory).await?;
    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            if let Some(pkg) = parse_pkg_filename(&filename) {
                if let Ok(meta) = fs::metadata(&path).await {
                    let mut info = pkg;
                    info.size = meta.len();
                    result.push(info);
                }
            }
        }
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

/// 解析 .pkg.tar.zst 文件名
/// 格式: {pkgname}-[{epoch}:]{pkgver}-{pkgrel}-{arch}.pkg.tar.zst
/// 示例:
///   electron8-bin-1:8.5.5-6-x86_64.pkg.tar.zst
///   linux-zen-6.12-1-x86_64.pkg.tar.zst
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
        (Some(ver_part[..pos].to_string()), ver_part[pos + 1..].to_string())
    } else {
        (None, ver_part)
    };

    Some(PkgFileInfo {
        filename: filename.to_string(),
        name,
        epoch,
        version,
        pkgrel,
        arch,
        size: 0,
    })
}

// ════════════════════════════════════════════════════════════
// Tauri 命令
// ════════════════════════════════════════════════════════════

/// 扫描指定目录内容（单层）
#[tauri::command]
pub async fn scan_directory(path: String) -> Result<Vec<DirEntry>, String> {
    info!("扫描目录: {}", path);
    list_directory(&path).await.map_err(|e| e.to_string())
}

/// 递归扫描目录树
#[tauri::command]
pub async fn scan_directory_recursive(
    path: String,
    max_depth: Option<u32>,
) -> Result<Vec<DirEntry>, String> {
    info!("递归扫描目录: {}", path);
    let depth = max_depth.unwrap_or(0);
    list_directory_recursive(&path, depth)
        .await
        .map_err(|e| e.to_string())
}

/// 扫描 .pkg.tar.zst 包文件
#[tauri::command]
pub async fn scan_pkg_files_cmd(directory: String) -> Result<Vec<PkgFileInfo>, String> {
    info!("扫描包文件: {}", directory);
    scan_pkg_files(&directory).await.map_err(|e| e.to_string())
}
