/**
 * cache_cleanup.rs - 缓存清理命令
 *
 * 功能：
 * - clean_system_cache: 清理系统缓存 /var/cache/pacman/pkg（删除所有文件和目录）
 * - clean_custom_cache_dirs: 清理自定义 AUR 软件助手缓存目录（仅删除已知缓存产物）
 * - check_cache_cleanup_sudoers: 检测缓存清理 sudoers 配置
 * - get_cache_cleanup_sudoers_command: 获取缓存清理 sudoers 配置命令
 *
 * 安全修复 (R11): clean_custom_cache_dirs 仅删除已知缓存产物，
 * 防止误设目录导致用户数据丢失。
 */
use log::info;
use tauri::State;

use crate::commands::sysops::backup_install::build_pacman_install_rules;
use crate::errors::AppResult;
use crate::AppState;

/// 已知的缓存产物文件名后缀白名单
const CACHE_FILE_SUFFIXES: &[&str] = &[
    ".pkg.tar.zst",  // 二进制包文件
    ".SRCINFO",      // AUR 源码信息
    "PKGBUILD",      // AUR 构建脚本
];

/// 已知可安全删除的子目录名（构建缓存目录）
const CACHE_SUBDIR_WHITELIST: &[&str] = &[
    "pkg",
    "src",
    ".BUILD",
    ".SRCINFO",
];

/// 清理系统缓存 /var/cache/pacman/pkg
///
/// 删除目录中的所有文件和目录（包括 download-* 目录）
/// 需要 root 权限
#[tauri::command]
pub async fn clean_system_cache() -> AppResult<String> {
    info!("[缓存清理] 开始清理系统缓存 /var/cache/pacman/pkg");

    // 检查目录是否存在
    let path = std::path::Path::new("/var/cache/pacman/pkg");
    if !path.exists() {
        return Ok("系统缓存目录不存在，跳过清理".to_string());
    }

    // 使用 find 命令删除所有内容（处理 glob 和特殊字符）
    let output = tokio::process::Command::new("sudo")
        .args([
            "find",
            "/var/cache/pacman/pkg/",
            "-mindepth",
            "1",
            "-delete",
        ])
        .output()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("执行清理失败: {}", e)))?;

    if output.status.success() {
        info!("[缓存清理] 系统缓存清理完成");
        Ok("系统缓存清理完成".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(crate::errors::AppError::SystemCommand(format!(
            "清理系统缓存失败: {}",
            stderr.trim()
        )))
    }
}

/// 清理自定义 AUR 软件助手缓存目录
///
/// 安全修复 (R11): 仅删除已知缓存产物（.pkg.tar.zst 文件及特定构建子目录），
/// 不再删除所有非隐藏内容，防止误设目录导致用户数据丢失。
#[tauri::command]
pub async fn clean_custom_cache_dirs(state: State<'_, AppState>) -> AppResult<String> {
    info!("[缓存清理] 开始清理自定义缓存目录");

    // 获取所有缓存目录配置
    let cache_dirs = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        crate::commands::fileops::cache_dirs::get_cache_dirs(&db)?
    };

    let mut cleaned_count = 0;
    let mut errors = Vec::new();

    for dir in &cache_dirs {
        // 跳过系统缓存目录（已单独处理）
        if dir.path == "/var/cache/pacman/pkg" {
            continue;
        }

        let path = std::path::Path::new(&dir.path);
        if !path.exists() {
            log::debug!("[缓存清理] 缓存目录不存在，跳过: {}", dir.path);
            continue;
        }

        // 安全修复 (R11): 仅读取目录内容，精确删除已知缓存产物
        match tokio::fs::read_dir(&dir.path).await {
            Ok(mut entries) => {
                while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                    let entry_path = entry.path();
                    let entry_name = entry.file_name().to_string_lossy().to_string();

                    // 跳过隐藏文件和文件夹（以 . 开头）
                    if entry_name.starts_with('.') {
                        log::debug!("[缓存清理] 跳过隐藏项: {}", entry_path.display());
                        continue;
                    }

                    let is_deletable = if entry_path.is_file() {
                        // 仅删除已知缓存产物文件
                        CACHE_FILE_SUFFIXES
                            .iter()
                            .any(|ext| entry_name.ends_with(ext))
                    } else if entry_path.is_dir() {
                        // 仅删除已知构建缓存子目录
                        CACHE_SUBDIR_WHITELIST.contains(&entry_name.as_str())
                    } else {
                        false
                    };

                    if !is_deletable {
                        log::debug!(
                            "[缓存清理] 跳过非缓存项: {} ({})",
                            entry_path.display(),
                            entry_name
                        );
                        continue;
                    }

                    // 删除已识别的缓存项
                    match tokio::fs::remove_dir_all(&entry_path).await {
                        Ok(_) => {
                            log::debug!("[缓存清理] 已删除目录: {}", entry_path.display());
                            cleaned_count += 1;
                        }
                        Err(_) => {
                            // 尝试删除文件
                            if let Err(e) = tokio::fs::remove_file(&entry_path).await {
                                errors.push(format!(
                                    "删除 {} 失败: {}",
                                    entry_path.display(),
                                    e
                                ));
                            } else {
                                log::debug!("[缓存清理] 已删除文件: {}", entry_path.display());
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(format!("读取目录 {} 失败: {}", dir.path, e));
            }
        }
    }

    let msg = if errors.is_empty() {
        format!("自定义缓存目录清理完成，共删除 {} 个条目", cleaned_count)
    } else {
        format!(
            "自定义缓存目录清理完成，删除 {} 个条目，{} 个错误",
            cleaned_count,
            errors.len()
        )
    };

    info!("[缓存清理] {}", msg);
    Ok(msg)
}

/// 检测缓存清理 sudoers 配置是否可用
///
/// 直接尝试执行清理命令来检测配置是否正确
#[tauri::command]
pub async fn check_cache_cleanup_sudoers() -> AppResult<bool> {
    // 检查系统缓存目录是否存在
    let path = std::path::Path::new("/var/cache/pacman/pkg");
    if !path.exists() {
        return Ok(true); // 目录不存在，无需清理
    }

    // 直接尝试执行清理命令（使用 -n 参数避免密码提示）
    let output = tokio::process::Command::new("sudo")
        .args(["-n", "rm", "-rf", "/var/cache/pacman/pkg/*"])
        .output()
        .await;

    match output {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

/// 获取缓存清理 sudoers 配置命令
///
/// 生成一个 sudoers 配置命令，允许当前用户免密执行缓存清理。
/// 其中 pacman -U 的免密范围限定到备份目录（而非通配 *），缩小提权面。
#[tauri::command]
pub async fn get_cache_cleanup_sudoers_command(state: State<'_, AppState>) -> AppResult<String> {
    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("获取用户名失败: {}", e)))?;

    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let backup_dir = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        db.get_setting("backup_dir")
            .ok()
            .flatten()
            .map(|s| s.value)
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "/run/media/zxp/Backup/Linux/ZST".to_string())
    };

    // 使用 sudo -n cat 检查文件是否已存在且包含缓存清理规则（非交互，避免弹出密码框）
    let sudoers_path = "/etc/sudoers.d/aur-helper-backup";
    let output = tokio::process::Command::new("sudo")
        .args(["-n", "cat", sudoers_path])
        .output()
        .await;

    match output {
        Ok(output) if output.status.success() => {
            let content = String::from_utf8_lossy(&output.stdout);

            if content.contains("/usr/bin/find /var/cache/pacman/pkg/") {
                // 已包含缓存清理规则
                return Ok("sudoers 配置已包含缓存清理规则，无需重复配置".to_string());
            }

            // 文件存在但不包含缓存清理规则，追加规则
            Ok(format!(
                "echo \"{} ALL=(ALL) NOPASSWD: /usr/bin/find /var/cache/pacman/pkg/* -mindepth 1 -delete\" | sudo tee -a /etc/sudoers.d/aur-helper-backup",
                username
            ))
        }
        _ => {
            // 文件不存在，创建新的配置文件（pacman -U 限定到备份目录，含子目录）
            let pacman_rule = build_pacman_install_rules(&backup_dir);
            Ok(format!(
                "echo \"{} ALL=(ALL) NOPASSWD: {}, /usr/bin/find /var/cache/pacman/pkg/* -mindepth 1 -delete\" | sudo tee /etc/sudoers.d/aur-helper-backup",
                username, pacman_rule
            ))
        }
    }
}

/// 清除所有 GitHub tags 缓存
#[tauri::command]
pub async fn clear_github_tag_cache(state: State<'_, AppState>) -> AppResult<i64> {
    let db = state.db.lock()?;
    let count = db.clear_all_caches()?;
    info!("[GitHub Cache] 已清除 {} 条缓存记录", count);
    Ok(count)
}

/// 清除已过期的 GitHub tags 缓存（超过 github_cache_ttl 秒的记录）
#[tauri::command]
pub async fn clear_expired_github_tag_cache(state: State<'_, AppState>) -> AppResult<i64> {
    let db = state.db.lock()?;
    let count = db.clear_expired_caches()?;
    info!("[GitHub Cache] 已清除 {} 条过期缓存记录", count);
    Ok(count)
}

/// 获取 GitHub tags 缓存统计信息
#[tauri::command]
pub async fn get_github_tag_cache_stats(state: State<'_, AppState>) -> AppResult<serde_json::Value> {
    let db = state.db.lock()?;
    let count = db.cache_count()?;
    let ttl = db.get_cache_ttl_seconds()?;
    Ok(serde_json::json!({ "count": count, "ttl_seconds": ttl }))
}
