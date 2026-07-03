use anyhow::Result;
use std::path::Path;
use tokio::fs;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Serialize)]
pub struct DetectedCache {
    pub cache_type: String,
    pub cache_path: String,
    pub package_count: usize,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified_at: Option<String>,
}

pub async fn detect_system_caches() -> Result<Vec<DetectedCache>> {
    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/home/zxp-archlinux".to_string());
    let paru_path = format!("{}/.cache/paru/clone", home);
    let yay_path = format!("{}/.cache/yay", home);
    let pacman_path = "/var/cache/pacman/pkg".to_string();
    let mut results = Vec::new();

    for (cache_type, path) in [
        ("paru", &paru_path),
        ("yay", &yay_path),
        ("pacman", &pacman_path),
    ] {
        let pb = Path::new(path);
        if pb.exists() {
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

pub async fn list_directory(path: &str) -> Result<Vec<DirEntry>> {
    let mut entries = fs::read_dir(path).await?;
    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let meta = entry.metadata().await?;
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
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

async fn scan_cache_dir(path: &Path) -> (usize, u64) {
    let mut count = 0;
    let mut total_size = 0;
    if let Ok(mut entries) = fs::read_dir(path).await {
        while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(ext) = entry_path.extension() {
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
