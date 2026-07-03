/**
 * files.rs - 文件操作命令
 *
 * 提供文件和目录的增删改查功能
 * 包括：复制、移动、删除、创建目录、读取文件、列出目录等
 */
use log::{info, debug};     // 日志记录
use std::path::Path;         // 文件路径操作
use std::time::UNIX_EPOCH;  // Unix 时间戳起始点
use tauri::command;          // Tauri 命令宏
use tokio::fs;               // 异步文件系统操作

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt; // Unix 权限扩展（用于获取文件权限位）

/// 目录条目信息
#[derive(serde::Serialize)]
pub struct DirEntry {
    pub name: String,              // 文件或目录名
    pub is_dir: bool,              // 是否为目录
    pub size: u64,                 // 文件大小（字节）
    pub modified_at: Option<String>, // 最后修改时间（格式化字符串）
}

/// 文件元信息
#[derive(serde::Serialize)]
pub struct FileMetadata {
    pub size: u64,                 // 文件大小（字节）
    pub modified_at: Option<String>, // 最后修改时间
    pub is_dir: bool,              // 是否为目录
    pub permissions: String,       // 权限字符串（如 "rw-"）
}

/// 包文件信息（用于解析 .pkg.tar.zst/.xz 文件名）
#[derive(serde::Serialize)]
pub struct PkgFileInfo {
    pub filename: String,  // 完整文件名
    pub name: String,      // 包名（不含版本）
    pub epoch: u64,        // epoch 版本号
    pub pkgrel: String,    // 包发布号
    pub arch: String,      // 架构（如 x86_64）
    pub size: u64,         // 文件大小
}

/// 批量删除结果
#[derive(serde::Serialize)]
pub struct BatchDeleteResult {
    pub deleted: usize,     // 成功删除的文件数
    pub errors: Vec<String>, // 删除失败的错误信息
}

/// 解析 pacman 包文件名
/// 格式：{name}-{epoch}:{version}-{pkgrel}-{arch}.pkg.tar.{zst|xz}
/// @param filename - 包文件名
/// @returns 解析后的包文件信息
fn parse_pkg_filename(filename: &str) -> Option<PkgFileInfo> {
    // 去除 .pkg.tar.zst 或 .pkg.tar.xz 后缀
    let base = if filename.ends_with(".pkg.tar.zst") {
        filename.trim_end_matches(".pkg.tar.zst")
    } else if filename.ends_with(".pkg.tar.xz") {
        filename.trim_end_matches(".pkg.tar.xz")
    } else {
        return None; // 不支持的文件格式
    };

    // 从右侧按 '-' 分割，最后三段分别是 arch、pkgrel 和 name-version
    let parts: Vec<&str> = base.rsplitn(3, '-').collect();
    if parts.len() < 3 {
        return None;
    }
    let arch = parts[0].to_string();
    let pkgrel = parts[1].to_string();
    let name_version = parts[2];

    // 处理 epoch（如 "1:1.0.0" 中的 "1:"）
    let (epoch, name_ver) = if let Some(colon_pos) = name_version.find(':') {
        let epoch_str = &name_version[..colon_pos];
        let epoch: u64 = epoch_str.parse().unwrap_or(0);
        (epoch, &name_version[colon_pos + 1..])
    } else {
        (0u64, name_version)
    };

    // 从 name-version 中分离包名（最后一个 '-' 前为包名）
    let name = if let Some(dash_pos) = name_ver.rfind('-') {
        name_ver[..dash_pos].to_string()
    } else {
        name_ver.to_string()
    };

    Some(PkgFileInfo {
        filename: filename.to_string(),
        name,
        epoch,
        pkgrel,
        arch,
        size: 0, // 由调用者填充实际大小
    })
}

/// 复制文件或目录
/// @param src - 源路径
/// @param dst - 目标路径
#[command]
pub async fn copy_file(src: String, dst: String) -> Result<(), String> {
    debug!("Copying file: {} -> {}", src, dst);
    let src_path = Path::new(&src);
    if !src_path.exists() {
        return Err(format!("Source file does not exist: {}", src));
    }
    if src_path.is_dir() {
        copy_dir_recursive(&src, &dst).await?; // 目录递归复制
    } else {
        // 确保目标目录存在
        if let Some(parent) = Path::new(&dst).parent() {
            fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }
        fs::copy(&src, &dst).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 递归复制目录
/// @param src - 源目录路径
/// @param dst - 目标目录路径
async fn copy_dir_recursive(src: &str, dst: &str) -> Result<(), String> {
    fs::create_dir_all(dst).await.map_err(|e| e.to_string())?;
    let mut entries = fs::read_dir(src).await.map_err(|e| e.to_string())?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let src_path = entry.path();
        let dst_path = Path::new(dst).join(entry.file_name());
        if src_path.is_dir() {
            // 递归复制子目录
            let src_str = src_path.to_string_lossy().to_string();
            let dst_str = dst_path.to_string_lossy().to_string();
            Box::pin(copy_dir_recursive(&src_str, &dst_str)).await?;
        } else {
            fs::copy(&src_path, &dst_path).await.map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 移动文件或目录
/// @param src - 源路径
/// @param dst - 目标路径
#[command]
pub async fn move_file(src: String, dst: String) -> Result<(), String> {
    debug!("Moving file: {} -> {}", src, dst);
    let src_path = Path::new(&src);
    if !src_path.exists() {
        return Err(format!("Source file does not exist: {}", src));
    }
    // 确保目标父目录存在
    if let Some(parent) = Path::new(&dst).parent() {
        fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    fs::rename(&src, &dst).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 删除文件或目录
/// @param path - 要删除的路径
#[command]
pub async fn delete_file(path: String) -> Result<(), String> {
    debug!("Deleting file: {}", path);
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    if p.is_dir() {
        fs::remove_dir_all(&path).await.map_err(|e| e.to_string())?; // 递归删除目录
    } else {
        fs::remove_file(&path).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 删除目录（仅限目录）
/// @param path - 要删除的目录路径
#[command]
pub async fn delete_directory(path: String) -> Result<(), String> {
    debug!("Deleting directory: {}", path);
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("Directory does not exist: {}", path));
    }
    if !p.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }
    fs::remove_dir_all(&path).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 创建目录（支持递归创建）
/// @param path - 要创建的目录路径
#[command]
pub async fn create_directory(path: String) -> Result<(), String> {
    debug!("Creating directory: {}", path);
    fs::create_dir_all(&path).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 读取文件内容为字符串
/// @param path - 文件路径
/// @returns 文件文本内容
#[command]
pub async fn read_file(path: String) -> Result<String, String> {
    debug!("Reading file: {}", path);
    fs::read_to_string(&path).await.map_err(|e| e.to_string())
}

/// 列出目录内容
/// @param path - 目录路径
/// @returns 目录条目列表（按名称排序）
#[command]
pub async fn list_directory(path: String) -> Result<Vec<DirEntry>, String> {
    debug!("Listing directory: {}", path);
    let mut entries = fs::read_dir(&path).await.map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let metadata = entry.metadata().await.map_err(|e| e.to_string())?;
        // 将修改时间转换为格式化字符串
        let modified_at = metadata.modified().ok().and_then(|t| {
            t.duration_since(UNIX_EPOCH).ok().map(|d| {
                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default()
            })
        });
        result.push(DirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
            modified_at,
        });
    }
    result.sort_by(|a, b| a.name.cmp(&b.name)); // 按名称排序
    Ok(result)
}

/// 检查文件是否存在
/// @param path - 文件路径
/// @returns 是否存在
#[command]
pub async fn file_exists(path: String) -> Result<bool, String> {
    debug!("Checking file exists: {}", path);
    Ok(Path::new(&path).exists())
}

/// 获取文件元信息
/// @param path - 文件路径
/// @returns 文件元数据（大小、修改时间、类型、权限）
#[command]
pub async fn file_metadata(path: String) -> Result<FileMetadata, String> {
    debug!("Getting file metadata: {}", path);
    let meta = fs::metadata(&path).await.map_err(|e| e.to_string())?;
    let modified_at = meta.modified().ok().and_then(|t| {
        t.duration_since(UNIX_EPOCH).ok().map(|d| {
            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default()
        })
    });
    // 解析 Unix 文件权限为 rwx 格式
    let perms = meta.permissions();
    let perm_str = format!(
        "{}{}{}",
        if perms.mode() & 0o400 != 0 { "r" } else { "-" },
        if perms.mode() & 0o200 != 0 { "w" } else { "-" },
        if perms.mode() & 0o100 != 0 { "x" } else { "-" },
    );
    Ok(FileMetadata {
        size: meta.len(),
        modified_at,
        is_dir: meta.is_dir(),
        permissions: perm_str,
    })
}

/// 扫描目录中的 .pkg.tar 文件
/// @param directory - 要扫描的目录
/// @returns 包文件信息列表（按包名排序）
#[command]
pub async fn scan_pkg_files(directory: String) -> Result<Vec<PkgFileInfo>, String> {
    debug!("Scanning PKG files in: {}", directory);
    let mut entries = fs::read_dir(&directory).await.map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            if filename.ends_with(".pkg.tar.zst") || filename.ends_with(".pkg.tar.xz") {
                let meta = fs::metadata(&path).await.map_err(|e| e.to_string())?;
                if let Some(mut pkg) = parse_pkg_filename(&filename) {
                    pkg.size = meta.len(); // 填入实际文件大小
                    result.push(pkg);
                }
            }
        }
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

/// 批量删除文件或目录
/// @param paths - 要删除的路径列表
/// @returns 批量删除结果（成功数和错误信息）
#[command]
pub async fn batch_delete(paths: Vec<String>) -> Result<BatchDeleteResult, String> {
    info!("Batch deleting {} files", paths.len());
    let mut deleted = 0usize;
    let mut errors = Vec::new();
    for path in &paths {
        let p = Path::new(path);
        if !p.exists() {
            errors.push(format!("Not found: {}", path));
            continue;
        }
        // 根据类型选择删除方式
        let result = if p.is_dir() {
            fs::remove_dir_all(path).await
        } else {
            fs::remove_file(path).await
        };
        match result {
            Ok(()) => deleted += 1,
            Err(e) => errors.push(format!("{}: {}", path, e)),
        }
    }
    Ok(BatchDeleteResult { deleted, errors })
}
