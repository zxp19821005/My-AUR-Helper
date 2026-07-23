use log::info;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

use crate::errors::AppResult;

/// 备份配置
pub struct BackupConfig {
    pub cache_path: String,  // 缓存目录路径（源）
    pub backup_path: String, // 备份目录路径（目标）
}

/// 备份结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupResult {
    pub copied: usize,       // 复制成功的文件数
    pub removed: usize,      // 清理的旧版本文件数
    pub errors: Vec<String>, // 操作过程中的错误信息列表
}

/// 执行备份操作
/// 将缓存目录中的 .pkg.tar.zst 文件复制到备份目录，
/// 然后清理备份目录中每个包的旧版本（仅保留最新版本）
/// @param config - 备份配置（源目录和目标目录）
/// @returns 备份结果统计
pub async fn run_backup(config: &BackupConfig) -> AppResult<BackupResult> {
    let mut result = BackupResult {
        copied: 0,
        removed: 0,
        errors: Vec::new(),
    };
    let cache_path = Path::new(&config.cache_path);
    let backup_path = Path::new(&config.backup_path);
    // 如果缓存目录不存在，直接返回空结果
    if !cache_path.exists() {
        return Ok(result);
    }
    // 如果备份目录不存在，自动创建
    if !backup_path.exists() {
        fs::create_dir_all(backup_path).await?;
    }

    // 第一阶段：复制新文件到备份目录
    let mut cache_entries = fs::read_dir(cache_path).await?;
    while let Some(entry) = cache_entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                // 只处理 archlinux 包文件 (.pkg.tar.zst)
                if ext == "pkg.tar.zst" {
                    let filename = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if filename.is_empty() {
                        continue;
                    }
                    let backup_file = backup_path.join(&filename);
                    // 仅在备份文件不存在或源文件更新时复制
                    if !backup_file.exists() || is_newer(&path, &backup_file).await {
                        fs::copy(&path, &backup_file).await?;
                        result.copied += 1;
                        info!("已复制到备份: {}", filename);
                    }
                }
            }
        }
    }

    // 第二阶段：清理备份目录中的旧版本
    // 使用 HashMap 按包名分组，每个包只保留最新版本
    let mut backup_entries = fs::read_dir(backup_path).await?;
    let mut pkg_map: std::collections::HashMap<String, Vec<(std::time::SystemTime, PathBuf)>> =
        std::collections::HashMap::new();
    // 遍历备份目录，将所有包文件按包名分组
    while let Some(entry) = backup_entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "pkg.tar.zst" {
                    let filename = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if filename.is_empty() {
                        continue;
                    }
                    // 从文件名提取包名（版本号前的部分）
                    let pkg_name = filename
                        .split('-')
                        .take_while(|s| !s.chars().next().is_some_and(|c| c.is_ascii_digit()))
                        .collect::<Vec<_>>()
                        .join("-");
                    // 记录文件修改时间
                    if let Ok(meta) = fs::metadata(&path).await {
                        if let Ok(mtime) = meta.modified() {
                            pkg_map.entry(pkg_name).or_default().push((mtime, path));
                        }
                    }
                }
            }
        }
    }
    // 对每个包，只保留最新（修改时间最晚）的文件，删除其余旧版本
    for versions in pkg_map.values() {
        if versions.len() > 1 {
            let mut sorted = versions.clone();
            sorted.sort_by_key(|b| std::cmp::Reverse(b.0)); // 按时间降序排序
                                                            // 跳过第一个（最新版本），删除其余
            for (_, old_path) in sorted.iter().skip(1) {
                fs::remove_file(old_path).await?;
                result.removed += 1;
                info!("已删除旧备份版本: {}", old_path.display());
            }
        }
    }
    Ok(result)
}

/// 检查源文件是否比目标文件更新
/// @param src - 源文件路径
/// @param dst - 目标文件路径
/// @returns 如果源文件修改时间晚于目标文件则返回 true，任何错误时也返回 true（安全复制）
async fn is_newer(src: &Path, dst: &Path) -> bool {
    let src_meta = fs::metadata(src).await;
    let dst_meta = fs::metadata(dst).await;
    match (src_meta, dst_meta) {
        (Ok(s), Ok(d)) => {
            let src_time = s.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let dst_time = d.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            src_time > dst_time // 源文件更新则返回 true
        }
        _ => true, // 无法获取元信息时默认复制
    }
}
