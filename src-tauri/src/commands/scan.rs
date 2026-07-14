/**
 * scan.rs - 包文件扫描模块
 *
 * 提供 .pkg.tar.zst 包文件扫描功能
 * 已移除未使用的目录扫描命令（scan_directory、scan_directory_recursive），
 * 这些命令存在路径遍历风险且未被前端使用
 */
use log::info;
use tokio::fs;

use crate::errors::AppResult;

// ════════════════════════════════════════════════════════════
// 数据结构
// ════════════════════════════════════════════════════════════

/// .pkg.tar.zst 包文件信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct PkgFileInfo {
    pub filename: String,
    pub name: String,
    pub epoch: Option<String>,
    pub version: String,
    pub pkgrel: String,
    pub arch: String,
    pub size: u64,
}

// ════════════════════════════════════════════════════════════
// .pkg.tar.zst 包文件扫描
// ════════════════════════════════════════════════════════════

/// 扫描指定目录中的 .pkg.tar.zst 包文件
pub async fn scan_pkg_files(directory: &str) -> AppResult<Vec<PkgFileInfo>> {
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

/// 扫描 .pkg.tar.zst 包文件
#[tauri::command]
pub async fn scan_pkg_files_cmd(directory: String) -> AppResult<Vec<PkgFileInfo>> {
    info!("扫描包文件: {}", directory);
    scan_pkg_files(&directory).await
}
