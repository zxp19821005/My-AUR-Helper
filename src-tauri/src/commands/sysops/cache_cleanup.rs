/**
 * cache_cleanup.rs - 缓存清理命令
 *
 * 功能：
 * - clean_system_cache: 清理系统缓存 /var/cache/pacman/pkg（删除未安装的包文件）
 * - clean_custom_cache_dirs: 清理自定义 AUR 软件助手缓存目录（删除构建目录，保留隐藏文件）
 * - check_cache_cleanup_sudoers: 检测缓存清理 sudoers 配置
 * - get_cache_cleanup_sudoers_command: 获取缓存清理 sudoers 配置命令
 *
 * 参考 paru -Scc 的行为：
 * - 系统缓存：只清理未安装的包文件
 * - AUR 缓存：清理构建目录，保留隐藏文件和 VCS 源
 */
use log::info;
use tauri::State;

use crate::errors::AppResult;
use crate::AppState;

/// 清理系统缓存 /var/cache/pacman/pkg
///
/// 只清理未安装的包文件（参考 pacman -Scc 的行为）
/// 需要 root 权限
#[tauri::command]
pub async fn clean_system_cache() -> AppResult<String> {
    info!("[缓存清理] 开始清理系统缓存 /var/cache/pacman/pkg");

    // 检查目录是否存在
    let path = std::path::Path::new("/var/cache/pacman/pkg");
    if !path.exists() {
        return Ok("系统缓存目录不存在，跳过清理".to_string());
    }

    // 获取已安装的包列表
    let installed_output = tokio::process::Command::new("pacman")
        .args(["-Qq"])
        .output()
        .await
        .map_err(|e| {
            crate::errors::AppError::SystemCommand(format!("获取已安装包列表失败: {}", e))
        })?;

    if !installed_output.status.success() {
        return Err(crate::errors::AppError::SystemCommand(
            "获取已安装包列表失败".to_string(),
        ));
    }

    let installed_pkgs: Vec<String> = String::from_utf8_lossy(&installed_output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    info!("[缓存清理] 已安装包数量: {}", installed_pkgs.len());

    // 读取缓存目录
    let mut entries = tokio::fs::read_dir(path)
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("读取缓存目录失败: {}", e)))?;

    let mut removed_count = 0;
    let mut errors = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("读取目录项失败: {}", e)))?
    {
        let filename = entry.file_name().to_string_lossy().to_string();

        // 只处理 .pkg.tar.zst 文件
        if !filename.ends_with(".pkg.tar.zst") {
            continue;
        }

        // 从文件名提取包名（格式：name-version-pkgrel-arch.pkg.tar.zst）
        let pkgname = extract_pkgname_from_filename(&filename);
        if let Some(name) = pkgname {
            // 如果包已安装，跳过（保留已安装包的缓存）
            if installed_pkgs.contains(&name) {
                log::debug!("[缓存清理] 保留已安装包的缓存: {}", filename);
                continue;
            }
        }

        // 删除未安装包的缓存文件（需要 sudo 权限）
        let file_path = entry.path();
        let output = tokio::process::Command::new("sudo")
            .args(["rm", "-f", &file_path.to_string_lossy()])
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                log::debug!("[缓存清理] 已删除: {}", file_path.display());
                removed_count += 1;
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                errors.push(format!(
                    "删除 {} 失败: {}",
                    file_path.display(),
                    stderr.trim()
                ));
            }
            Err(e) => {
                errors.push(format!("删除 {} 失败: {}", file_path.display(), e));
            }
        }
    }

    let msg = if errors.is_empty() {
        format!(
            "系统缓存清理完成，删除 {} 个未安装包的缓存文件",
            removed_count
        )
    } else {
        format!(
            "系统缓存清理完成，删除 {} 个文件，{} 个错误",
            removed_count,
            errors.len()
        )
    };

    info!("[缓存清理] {}", msg);
    Ok(msg)
}

/// 从 .pkg.tar.zst 文件名中提取包名
fn extract_pkgname_from_filename(filename: &str) -> Option<String> {
    // 文件名格式：name-version-pkgrel-arch.pkg.tar.zst
    let base = filename.strip_suffix(".pkg.tar.zst")?;
    let parts: Vec<&str> = base.rsplitn(3, '-').collect();
    if parts.len() < 3 {
        return None;
    }
    // parts[0] = arch, parts[1] = pkgrel, parts[2] = name-version
    let name_ver = parts[2];
    let dash_pos = name_ver.rfind('-')?;
    let name = name_ver[..dash_pos].to_string();
    Some(name)
}

/// 清理自定义 AUR 软件助手缓存目录
///
/// 只删除构建目录内容，保留隐藏文件和文件夹（参考 paru -Scc 的行为）
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

        // 读取目录内容
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

                    // 删除非隐藏的文件和文件夹
                    match tokio::fs::remove_dir_all(&entry_path).await {
                        Ok(_) => {
                            log::debug!("[缓存清理] 已删除目录: {}", entry_path.display());
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
/// 生成一个 sudoers 配置命令，允许当前用户免密执行缓存清理
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
