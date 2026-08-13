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
pub async fn scan_pkg_files(directory: &str) -> AppResult<Vec<PkgFileInfo>> {
    let mut result = Vec::new();
    let mut stack: Vec<std::path::PathBuf> = vec![std::path::PathBuf::from(directory)];

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
#[tauri::command]
pub async fn scan_pkg_files_cmd(directory: String) -> AppResult<Vec<PkgFileInfo>> {
    info!("扫描包文件: {}", directory);
    scan_pkg_files(&directory).await
}
