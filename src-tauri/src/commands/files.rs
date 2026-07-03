/**
 * files.rs - 文件操作命令
 *
 * 提供文件和目录的增删改查功能
 */
use log::{info, debug};
use std::path::Path;
use std::time::UNIX_EPOCH;
use tauri::command;
use tokio::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// 目录条目信息
#[derive(serde::Serialize)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified_at: Option<String>,
}

/// 文件元信息
#[derive(serde::Serialize)]
pub struct FileMetadata {
    pub size: u64,
    pub modified_at: Option<String>,
    pub is_dir: bool,
    pub permissions: String,
}

/// 批量删除结果
#[derive(serde::Serialize)]
pub struct BatchDeleteResult {
    pub deleted: usize,
    pub errors: Vec<String>,
}

/// 复制文件或目录
#[command]
pub async fn copy_file(src: String, dst: String) -> Result<(), String> {
    debug!("Copying file: {} -> {}", src, dst);
    let src_path = Path::new(&src);
    if !src_path.exists() {
        return Err(format!("Source file does not exist: {}", src));
    }
    if src_path.is_dir() {
        copy_dir_recursive(&src, &dst).await?;
    } else {
        if let Some(parent) = Path::new(&dst).parent() {
            fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }
        fs::copy(&src, &dst).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 递归复制目录
async fn copy_dir_recursive(src: &str, dst: &str) -> Result<(), String> {
    fs::create_dir_all(dst).await.map_err(|e| e.to_string())?;
    let mut entries = fs::read_dir(src).await.map_err(|e| e.to_string())?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let src_path = entry.path();
        let dst_path = Path::new(dst).join(entry.file_name());
        if src_path.is_dir() {
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
#[command]
pub async fn move_file(src: String, dst: String) -> Result<(), String> {
    debug!("Moving file: {} -> {}", src, dst);
    let src_path = Path::new(&src);
    if !src_path.exists() {
        return Err(format!("Source file does not exist: {}", src));
    }
    if let Some(parent) = Path::new(&dst).parent() {
        fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    fs::rename(&src, &dst).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 删除文件或目录
#[command]
pub async fn delete_file(path: String) -> Result<(), String> {
    debug!("Deleting file: {}", path);
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    if p.is_dir() {
        fs::remove_dir_all(&path).await.map_err(|e| e.to_string())?;
    } else {
        fs::remove_file(&path).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 删除目录（仅限目录）
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
#[command]
pub async fn create_directory(path: String) -> Result<(), String> {
    debug!("Creating directory: {}", path);
    fs::create_dir_all(&path).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// 读取文件内容为字符串
#[command]
pub async fn read_file(path: String) -> Result<String, String> {
    debug!("Reading file: {}", path);
    fs::read_to_string(&path).await.map_err(|e| e.to_string())
}

/// 列出目录内容
#[command]
pub async fn list_directory(path: String) -> Result<Vec<DirEntry>, String> {
    debug!("Listing directory: {}", path);
    let mut entries = fs::read_dir(&path).await.map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let metadata = entry.metadata().await.map_err(|e| e.to_string())?;
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
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

/// 检查文件是否存在
#[command]
pub async fn file_exists(path: String) -> Result<bool, String> {
    debug!("Checking file exists: {}", path);
    Ok(Path::new(&path).exists())
}

/// 获取文件元信息
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

/// 批量删除文件或目录
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
