/**
 * scan/mod.rs - 缓存扫描模块
 *
 * 检测系统中常见的 AUR helper 缓存目录
 * 支持 paru、yay 和 pacman 的缓存扫描
 */
use anyhow::Result;          // 通用错误处理
use std::path::Path;         // 文件路径操作
use tokio::fs;               // 异步文件系统操作
use chrono::{DateTime, Utc}; // 时间处理

/// 检测到的缓存目录信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct DetectedCache {
    pub cache_type: String,     // 缓存类型（paru/yay/pacman）
    pub cache_path: String,     // 缓存目录路径
    pub package_count: usize,   // 缓存中的包文件数量
    pub total_size_bytes: u64,  // 缓存总大小（字节）
}

/// 目录条目信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct DirEntry {
    pub name: String,               // 名称
    pub is_dir: bool,               // 是否为目录
    pub size: u64,                  // 大小（字节）
    pub modified_at: Option<String>, // 最后修改时间
}

/// 检测系统中常见的缓存目录
/// 扫描 paru、yay 和 pacman 的缓存路径，统计包数量和总大小
/// @returns 检测到的缓存目录列表
pub async fn detect_system_caches() -> Result<Vec<DetectedCache>> {
    // 获取用户 home 目录
    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/home/zxp-archlinux".to_string());
    // 各 AUR helper 的缓存路径
    let paru_path = format!("{}/.cache/paru/clone", home);
    let yay_path = format!("{}/.cache/yay", home);
    let pacman_path = "/var/cache/pacman/pkg".to_string();
    let mut results = Vec::new();

    // 遍历三种缓存类型
    for (cache_type, path) in [
        ("paru", &paru_path),
        ("yay", &yay_path),
        ("pacman", &pacman_path),
    ] {
        let pb = Path::new(path);
        if pb.exists() {  // 只处理存在的目录
            let (count, size) = scan_cache_dir(pb).await;
            results.push(DetectedCache {
                cache_type: cache_type.to_string(),
                cache_path: path.clone(),
                package_count: count,
                total_size_bytes: size,
            });
        }
    }
    Ok(results)
}

/// 列出指定目录的内容
/// @param path - 要扫描的目录路径
/// @returns 目录条目列表（按名称排序）
pub async fn list_directory(path: &str) -> Result<Vec<DirEntry>> {
    let mut entries = fs::read_dir(path).await?;
    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let meta = entry.metadata().await?;
        // 将系统时间转换为格式化字符串
        let modified_at = meta.modified().ok().map(|t| {
            let dt: DateTime<Utc> = t.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        });
        result.push(DirEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: meta.is_dir(),
            size: meta.len(),
            modified_at,
        });
    }
    result.sort_by(|a, b| a.name.cmp(&b.name)); // 按名称排序
    Ok(result)
}

/// 扫描缓存目录，统计包文件数量和总大小
/// 检测 .pkg.tar 结尾的文件或 .pkg 扩展名的文件
/// @param path - 缓存目录路径
/// @returns (文件数量, 总字节数)
async fn scan_cache_dir(path: &Path) -> (usize, u64) {
    let mut count = 0;
    let mut total_size = 0;
    if let Ok(mut entries) = fs::read_dir(path).await {
        while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
                    // 匹配 .pkg 扩展名或包含 .pkg.tar 的文件名
                    if ext == "pkg" || entry_path.to_string_lossy().contains(".pkg.tar") {
                        count += 1;
                        if let Ok(meta) = fs::metadata(&entry_path).await {
                            total_size += meta.len();
                        }
                    }
                }
            }
        }
    }
    (count, total_size)
}
