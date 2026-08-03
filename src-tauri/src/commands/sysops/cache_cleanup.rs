/**
 * cache_cleanup.rs - 缓存清理命令
 *
 * 功能：
 * - clean_system_cache: 清理系统缓存 /var/cache/pacman/pkg
 * - clean_custom_cache_dirs: 清理自定义 AUR 软件助手缓存目录
 * - check_cache_cleanup_sudoers: 检测缓存清理 sudoers 配置
 * - get_cache_cleanup_sudoers_command: 获取缓存清理 sudoers 配置命令
 */
use log::info;
use tauri::State;

use crate::errors::AppResult;
use crate::AppState;

/// 清理系统缓存 /var/cache/pacman/pkg
///
/// 需要 root 权限，使用 sudo 执行 rm -rf 命令
#[tauri::command]
pub async fn clean_system_cache() -> AppResult<String> {
    info!("[缓存清理] 开始清理系统缓存 /var/cache/pacman/pkg");

    // 检查目录是否存在
    let path = std::path::Path::new("/var/cache/pacman/pkg");
    if !path.exists() {
        return Ok("系统缓存目录不存在，跳过清理".to_string());
    }

    // 使用 sudo rm -rf 清理目录内容（保留目录本身）
    let output = tokio::process::Command::new("sudo")
        .args(["rm", "-rf", "/var/cache/pacman/pkg/*"])
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
/// 遍历所有启用的缓存目录（除系统缓存外），删除其中的所有文件和文件夹
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

        // 删除目录中的所有内容（保留目录本身）
        match tokio::fs::read_dir(&dir.path).await {
            Ok(mut entries) => {
                while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                    let entry_path = entry.path();
                    match tokio::fs::remove_dir_all(&entry_path).await {
                        Ok(_) => {
                            log::debug!("[缓存清理] 已删除: {}", entry_path.display());
                            cleaned_count += 1;
                        }
                        Err(_) => {
                            // 尝试删除文件
                            if let Err(e) = tokio::fs::remove_file(&entry_path).await {
                                errors.push(format!("删除 {} 失败: {}", entry_path.display(), e));
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
/// 使用 sudo cat 读取 /etc/sudoers.d/aur-helper-backup 文件，检查是否包含当前用户的缓存清理免密规则
#[tauri::command]
pub async fn check_cache_cleanup_sudoers() -> AppResult<bool> {
    // 获取当前用户名
    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("获取用户名失败: {}", e)))?;
    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // 使用 sudo cat 读取 sudoers 文件（普通用户无权直接读取）
    let sudoers_path = "/etc/sudoers.d/aur-helper-backup";
    let output = tokio::process::Command::new("sudo")
        .args(["cat", sudoers_path])
        .output()
        .await;

    match output {
        Ok(output) if output.status.success() => {
            let content = String::from_utf8_lossy(&output.stdout);

            // 检查是否包含当前用户名的 NOPASSWD 规则
            let user_pattern = format!("{} ALL=(ALL) NOPASSWD:", username);
            if !content.contains(&user_pattern) {
                return Ok(false);
            }

            // 检查是否包含缓存清理的命令（/usr/bin/rm -rf /var/cache/pacman/pkg/*）
            // 注意：sudoers 文件中可能有多个命令用逗号分隔
            Ok(content.contains("/usr/bin/rm -rf /var/cache/pacman/pkg/*"))
        }
        _ => Ok(false), // 文件不存在或读取失败，需要配置
    }
}

/// 获取缓存清理 sudoers 配置命令
///
/// 生成一个 sudoers 配置命令，允许当前用户免密执行缓存清理
/// 如果文件已存在，会追加缓存清理规则
#[tauri::command]
pub async fn get_cache_cleanup_sudoers_command() -> AppResult<String> {
    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("获取用户名失败: {}", e)))?;

    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // 使用 sudo cat 检查文件是否已存在且包含缓存清理规则
    let sudoers_path = "/etc/sudoers.d/aur-helper-backup";
    let output = tokio::process::Command::new("sudo")
        .args(["cat", sudoers_path])
        .output()
        .await;

    match output {
        Ok(output) if output.status.success() => {
            let content = String::from_utf8_lossy(&output.stdout);

            if content.contains("/usr/bin/rm -rf /var/cache/pacman/pkg/*") {
                // 已包含缓存清理规则
                return Ok("sudoers 配置已包含缓存清理规则，无需重复配置".to_string());
            }

            // 文件存在但不包含缓存清理规则，追加规则
            Ok(format!(
                "echo \"{} ALL=(ALL) NOPASSWD: /usr/bin/rm -rf /var/cache/pacman/pkg/*\" | sudo tee -a /etc/sudoers.d/aur-helper-backup",
                username
            ))
        }
        _ => {
            // 文件不存在，创建新的配置文件
            Ok(format!(
                "echo \"{} ALL=(ALL) NOPASSWD: /usr/bin/pacman -U *, /usr/bin/rm -rf /var/cache/pacman/pkg/*\" | sudo tee /etc/sudoers.d/aur-helper-backup",
                username
            ))
        }
    }
}
