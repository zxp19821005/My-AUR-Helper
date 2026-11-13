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

use crate::commands::sysops::backup_install::{
    has_nopasswd_rule, sudo_privileges_text, CACHE_CLEANUP_ARGS, CACHE_CLEANUP_RULE,
};
use crate::errors::AppResult;
use crate::AppState;

/// 已知的缓存产物文件名后缀白名单
const CACHE_FILE_SUFFIXES: &[&str] = &[
    ".pkg.tar.zst", // 二进制包文件
    ".SRCINFO",     // AUR 源码信息
    "PKGBUILD",     // AUR 构建脚本
];

/// 已知可安全删除的子目录名（构建缓存目录）
const CACHE_SUBDIR_WHITELIST: &[&str] = &["pkg", "src", ".BUILD", ".SRCINFO"];

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

    // 命令必须与 CACHE_CLEANUP_RULE 完全一致（绝对路径 + 相同参数），
    // sudoers 才会命中 NOPASSWD 规则；-n 保证无免密时立即失败而非等待密码输入。
    // GUI 环境没有可交互终端，缺少 -n 时 sudo 会挂起等待密码，表现为页面「卡死」。
    let output = tokio::process::Command::new("sudo")
        .args(CACHE_CLEANUP_ARGS)
        .output()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("执行清理失败: {}", e)))?;

    if output.status.success() {
        info!("[缓存清理] 系统缓存清理完成");
        Ok("系统缓存清理完成".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(crate::errors::AppError::SystemCommand(format!(
            "清理系统缓存失败（sudo 免密未生效）: {}。请配置缓存清理 sudoers 规则后重试",
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
/// 通过 `sudo -n -l` 读取当前用户实际生效的免密规则，判断是否包含
/// `CACHE_CLEANUP_RULE`。旧实现执行 `sudo -n rm -rf ...` 探测：检测的命令
/// 与真正执行的 `find` 无关，且一旦 rm 恰好被免密就会真的删除缓存文件。
#[tauri::command]
pub async fn check_cache_cleanup_sudoers() -> AppResult<bool> {
    // 检查系统缓存目录是否存在
    let path = std::path::Path::new("/var/cache/pacman/pkg");
    if !path.exists() {
        return Ok(true); // 目录不存在，无需清理
    }

    Ok(match sudo_privileges_text().await {
        Some(text) => has_nopasswd_rule(&text, CACHE_CLEANUP_RULE),
        None => false,
    })
}

/// 获取缓存清理 sudoers 配置命令
///
/// 只生成「缓存清理」这一条规则的追加命令，不再整文件覆盖写入：
/// `/etc/sudoers.d/aur-helper-backup` 由备份安装 / 缓存安装 / 缓存清理三个入口共用，
/// 任意一处用 `tee`（覆盖）都会冲掉其它两处已配置的规则。
#[tauri::command]
pub async fn get_cache_cleanup_sudoers_command() -> AppResult<String> {
    if let Some(text) = sudo_privileges_text().await {
        if has_nopasswd_rule(&text, CACHE_CLEANUP_RULE) {
            return Ok("sudoers 配置已包含缓存清理规则，无需重复配置".to_string());
        }
    }

    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| crate::errors::AppError::SystemCommand(format!("获取用户名失败: {}", e)))?;
    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let line = format!("{username} ALL=(ALL) NOPASSWD: {CACHE_CLEANUP_RULE}");
    // 单次 sudo：root 内完成幂等判断 + 追加 + 修正权限，避免多次提权或吞掉密码提示。
    // 规则与命令行均不含单引号，可安全包在 sh -c 的单引号内。
    Ok(format!(
        "sudo sh -c 'grep -qF \"{CACHE_CLEANUP_RULE}\" /etc/sudoers.d/aur-helper-backup || printf \"%s\\n\" \"{line}\" >> /etc/sudoers.d/aur-helper-backup; chmod 440 /etc/sudoers.d/aur-helper-backup'"
    ))
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
pub async fn get_github_tag_cache_stats(
    state: State<'_, AppState>,
) -> AppResult<serde_json::Value> {
    let db = state.db.lock()?;
    let count = db.cache_count()?;
    let ttl = db.get_cache_ttl_seconds()?;
    Ok(serde_json::json!({ "count": count, "ttl_seconds": ttl }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// sudoers 规则与真正传给 sudo 的命令必须逐 token 一致，否则免密规则不会命中。
    /// `-n` 是 sudo 自身选项，不参与被授权命令的匹配，故从下标 1 开始拼接。
    #[test]
    fn cleanup_args_match_sudoers_rule() {
        assert_eq!(CACHE_CLEANUP_ARGS[0], "-n");
        assert_eq!(CACHE_CLEANUP_ARGS[1..].join(" "), CACHE_CLEANUP_RULE);
    }

    /// 规则不含 `*` 通配，避免 find 的路径参数被放宽到缓存目录之外。
    #[test]
    fn cleanup_rule_has_no_wildcard() {
        assert!(!CACHE_CLEANUP_RULE.contains('*'));
        assert!(CACHE_CLEANUP_RULE.starts_with("/usr/bin/find "));
    }

    /// `sudo -n -l` 输出中只有 NOPASSWD 行才算免密；通用 `(ALL : ALL) ALL` 不算。
    #[test]
    fn nopasswd_rule_detection_ignores_password_required_rules() {
        let text = "(ALL : ALL) ALL\n(ALL) NOPASSWD: /usr/bin/pacman -U --noconfirm /bak/*";
        assert!(!has_nopasswd_rule(text, CACHE_CLEANUP_RULE));
        let with_rule = format!("{text}\n(ALL) NOPASSWD: {CACHE_CLEANUP_RULE}");
        assert!(has_nopasswd_rule(&with_rule, CACHE_CLEANUP_RULE));
    }
}
