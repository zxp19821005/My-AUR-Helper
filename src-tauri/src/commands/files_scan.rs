/**
 * files_scan.rs - 包文件扫描命令
 *
 * 扫描目录中的 .pkg.tar.zst/.xz 包文件并解析文件名
 */
use log::debug;
use tauri::command;
use tokio::fs;

use crate::errors::AppResult;

/// 包文件信息
#[derive(serde::Serialize)]
pub struct PkgFileInfo {
    pub filename: String,
    pub name: String,
    pub epoch: u64,
    pub pkgrel: String,
    pub arch: String,
    pub size: u64,
}

/// 解析 pacman 包文件名
fn parse_pkg_filename(filename: &str) -> Option<PkgFileInfo> {
    let base = if filename.ends_with(".pkg.tar.zst") {
        filename.trim_end_matches(".pkg.tar.zst")
    } else if filename.ends_with(".pkg.tar.xz") {
        filename.trim_end_matches(".pkg.tar.xz")
    } else {
        return None;
    };

    let parts: Vec<&str> = base.rsplitn(3, '-').collect();
    if parts.len() < 3 {
        return None;
    }
    let arch = parts[0].to_string();
    let pkgrel = parts[1].to_string();
    let name_version = parts[2];

    let (epoch, name_ver) = if let Some(colon_pos) = name_version.find(':') {
        let epoch_str = &name_version[..colon_pos];
        let epoch: u64 = epoch_str.parse().unwrap_or(0);
        (epoch, &name_version[colon_pos + 1..])
    } else {
        (0u64, name_version)
    };

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
        size: 0,
    })
}

/// 扫描目录中的 .pkg.tar 文件
#[command]
pub async fn scan_pkg_files(directory: String) -> AppResult<Vec<PkgFileInfo>> {
    debug!("正在扫描目录中的 PKG 文件: {}", directory);
    let mut entries = fs::read_dir(&directory).await?;
    let mut result = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            if filename.ends_with(".pkg.tar.zst") || filename.ends_with(".pkg.tar.xz") {
                let meta = fs::metadata(&path).await?;
                if let Some(mut pkg) = parse_pkg_filename(&filename) {
                    pkg.size = meta.len();
                    result.push(pkg);
                }
            }
        }
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}
