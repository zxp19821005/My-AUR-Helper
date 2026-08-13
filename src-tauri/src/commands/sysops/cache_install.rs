/**
 * cache_install.rs - 缓存包安装和信息查询命令
 *
 * 功能：
 * - get_cache_package_info: 获取缓存包文件详细信息（pacman -Qip）
 * - install_cache_package: 安装缓存包（sudo pacman -U --noconfirm）
 *
 * 路径校验白名单与缓存清理命令保持一致：所有「启用的缓存目录」+ 系统缓存
 * /var/cache/pacman/pkg，防止读取 / 安装任意路径的恶意包。
 * sudoers 免密复用缓存清理规则（已包含允许对缓存目录执行 pacman -U 的规则）。
 */
use log::{error, info};
use std::path::PathBuf;
use tauri::State;

use crate::commands::fileops::cache_dirs::get_cache_dirs;
use crate::commands::sysops::backup_install::{
    build_pacman_install_rules, has_pacman_install_rule, validate_package_path,
};
use crate::commands::sysops::pacman_lock::with_pacman_write_lock;
use crate::db::Database;
use crate::errors::{AppError, AppResult};
use crate::AppState;

/// 收集缓存包路径校验允许的根目录（启用的缓存目录 + 系统缓存）
fn cache_allowed_roots(db: &Database) -> AppResult<Vec<PathBuf>> {
    let mut roots: Vec<PathBuf> = get_cache_dirs(db)?
        .into_iter()
        .map(|d| PathBuf::from(d.path))
        .collect();
    let system = PathBuf::from("/var/cache/pacman/pkg");
    if !roots.contains(&system) {
        roots.push(system);
    }
    Ok(roots)
}

/// 获取缓存包文件信息（pacman -Qip）
///
/// 路径必须经 `validate_package_path` 校验，防止读取任意文件。
#[tauri::command]
pub async fn get_cache_package_info(
    state: State<'_, AppState>,
    full_path: String,
) -> AppResult<String> {
    let allowed_roots = {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
        cache_allowed_roots(&db)?
    };
    let safe_path = validate_package_path(&full_path, &allowed_roots)?;

    let output = tokio::process::Command::new("pacman")
        .args(["-Qip", &safe_path.to_string_lossy()])
        .output()
        .await
        .map_err(|e| AppError::SystemCommand(format!("执行 pacman 失败: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::SystemCommand(format!(
            "pacman -Qip 失败: {}",
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 安装缓存包（sudo pacman -U --noconfirm）
///
/// 路径必须经 `validate_package_path` 校验，防止以 root 安装任意路径的恶意包。
#[tauri::command]
pub async fn install_cache_package(
    state: State<'_, AppState>,
    full_path: String,
) -> AppResult<String> {
    let allowed_roots = {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
        cache_allowed_roots(&db)?
    };
    let safe_path = validate_package_path(&full_path, &allowed_roots)?;

    info!("[缓存管理] 开始安装缓存包: {}", safe_path.display());

    let output = with_pacman_write_lock(|| async {
        tokio::process::Command::new("sudo")
            .args(["pacman", "-U", "--noconfirm", &safe_path.to_string_lossy()])
            .output()
            .await
    })
    .await
    .map_err(|e| AppError::SystemCommand(format!("执行安装失败: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        info!("[缓存管理] 安装成功: {}", safe_path.display());
        Ok(stdout)
    } else {
        error!("[缓存管理] 安装失败: {} - {}", safe_path.display(), stderr);
        Err(AppError::SystemCommand(format!("安装失败:\n{}", stderr)))
    }
}

/// 检测缓存安装 sudoers 免密配置是否可用
///
/// 与缓存清理共用 /etc/sudoers.d/aur-helper-backup，校验其中是否包含
/// 允许 `pacman -U --noconfirm` 作用于各缓存目录（含子目录）的规则。
#[tauri::command]
pub async fn check_cache_install_sudoers(state: State<'_, AppState>) -> AppResult<bool> {
    let dirs = {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
        let mut v: Vec<String> = get_cache_dirs(&db)?.into_iter().map(|d| d.path).collect();
        let system = "/var/cache/pacman/pkg".to_string();
        if !v.contains(&system) {
            v.push(system);
        }
        v
    };

    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| AppError::SystemCommand(format!("获取用户名失败: {}", e)))?;
    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let sudoers_path = "/etc/sudoers.d/aur-helper-backup";
    // 使用 -n 非交互模式：已配置免密时静默读取，未配置时立即失败而不弹出密码框
    let output = tokio::process::Command::new("sudo")
        .args(["-n", "cat", sudoers_path])
        .output()
        .await;

    let content = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Ok(false),
    };

    Ok(dirs
        .iter()
        .any(|d| has_pacman_install_rule(&content, &username, d)))
}

/// 获取缓存安装 sudoers 配置命令
///
/// 生成一条允许当前用户对「所有启用缓存目录 + 系统缓存」执行
/// `pacman -U` 免密的规则，写入 /etc/sudoers.d/aur-helper-backup。
#[tauri::command]
pub async fn get_cache_install_sudoers_command(state: State<'_, AppState>) -> AppResult<String> {
    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| AppError::SystemCommand(format!("获取用户名失败: {}", e)))?;
    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let dirs = {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
        let mut v: Vec<String> = get_cache_dirs(&db)?.into_iter().map(|d| d.path).collect();
        let system = "/var/cache/pacman/pkg".to_string();
        if !v.contains(&system) {
            v.push(system);
        }
        v
    };

    let pacman_rules: Vec<String> = dirs.iter().map(|d| build_pacman_install_rules(d)).collect();
    let rule = pacman_rules.join(", ");

    let sudoers_path = "/etc/sudoers.d/aur-helper-backup";
    // 使用 -n 非交互模式：已配置免密时静默读取，未配置时立即失败而不弹出密码框
    let existing = tokio::process::Command::new("sudo")
        .args(["-n", "cat", sudoers_path])
        .output()
        .await;

    match existing {
        Ok(o) if o.status.success() => {
            let content = String::from_utf8_lossy(&o.stdout);
            if dirs.iter().all(|d| has_pacman_install_rule(&content, &username, d)) {
                return Ok("sudoers 配置已包含缓存安装规则，无需重复配置".to_string());
            }
            Ok(format!(
                "echo \"{username} ALL=(ALL) NOPASSWD: {rule}\" | sudo tee -a /etc/sudoers.d/aur-helper-backup"
            ))
        }
        _ => Ok(format!(
            "echo \"{username} ALL=(ALL) NOPASSWD: {rule}\" | sudo tee /etc/sudoers.d/aur-helper-backup"
        )),
    }
}
