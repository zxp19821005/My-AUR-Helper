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
use log::info;
use std::path::PathBuf;
use tauri::State;

use crate::commands::fileops::cache_dirs::get_cache_dirs;
use crate::commands::sysops::backup_install::{build_pacman_install_rules, validate_package_path};
use crate::commands::sysops::sudoers::{
    are_all_rules_effective, build_sudoers_append_command, is_any_rule_effective,
};
use crate::commands::sysops::system_cmd::{
    current_username, install_package_locked, pacman_package_info,
};
use crate::db::{lock_db, Database};
use crate::errors::AppResult;
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
#[specta::specta]
pub async fn get_cache_package_info(
    state: State<'_, AppState>,
    full_path: String,
) -> AppResult<String> {
    let allowed_roots = {
        let db = lock_db(&state.db)?;
        cache_allowed_roots(&db)?
    };
    let safe_path = validate_package_path(&full_path, &allowed_roots)?;

    pacman_package_info(&safe_path).await
}

/// 安装缓存包（sudo pacman -U --noconfirm）
///
/// 路径必须经 `validate_package_path` 校验，防止以 root 安装任意路径的恶意包。
#[tauri::command]
#[specta::specta]
pub async fn install_cache_package(
    state: State<'_, AppState>,
    full_path: String,
) -> AppResult<String> {
    let allowed_roots = {
        let db = lock_db(&state.db)?;
        cache_allowed_roots(&db)?
    };
    let safe_path = validate_package_path(&full_path, &allowed_roots)?;

    info!("[缓存管理] 开始安装缓存包: {}", safe_path.display());

    install_package_locked(&safe_path, "[缓存管理]").await
}

/// 检测缓存安装 sudoers 免密配置是否可用
///
/// 与缓存清理共用 /etc/sudoers.d/aur-helper-backup。旧实现用 `sudo -n cat`
/// 读取该文件，但 cat 不在免密白名单时必然失败、永远返回 false；
/// 改为解析 `sudo -n -l` 实际生效的 NOPASSWD 规则。
#[tauri::command]
#[specta::specta]
pub async fn check_cache_install_sudoers(state: State<'_, AppState>) -> AppResult<bool> {
    let dirs = {
        let db = lock_db(&state.db)?;
        let mut v: Vec<String> = get_cache_dirs(&db)?.into_iter().map(|d| d.path).collect();
        let system = "/var/cache/pacman/pkg".to_string();
        if !v.contains(&system) {
            v.push(system);
        }
        v
    };

    let bases: Vec<String> = dirs
        .iter()
        .map(|d| format!("/usr/bin/pacman -U --noconfirm {}", d.trim_end_matches('/')))
        .collect();
    Ok(is_any_rule_effective(&bases).await)
}

/// 获取缓存安装 sudoers 配置命令
///
/// 生成一条允许当前用户对「所有启用缓存目录 + 系统缓存」执行
/// `pacman -U` 免密的规则，写入 /etc/sudoers.d/aur-helper-backup。
#[tauri::command]
#[specta::specta]
pub async fn get_cache_install_sudoers_command(state: State<'_, AppState>) -> AppResult<String> {
    let username = current_username().await?;

    let dirs = {
        let db = lock_db(&state.db)?;
        let mut v: Vec<String> = get_cache_dirs(&db)?.into_iter().map(|d| d.path).collect();
        let system = "/var/cache/pacman/pkg".to_string();
        if !v.contains(&system) {
            v.push(system);
        }
        v
    };

    let pacman_rules: Vec<String> = dirs.iter().map(|d| build_pacman_install_rules(d)).collect();
    let rule = pacman_rules.join(", ");

    // 用 sudo -n -l 判断规则是否已生效（读文件不可靠，见 check_cache_install_sudoers）
    let bases: Vec<String> = dirs
        .iter()
        .map(|d| format!("/usr/bin/pacman -U --noconfirm {}", d.trim_end_matches('/')))
        .collect();
    if are_all_rules_effective(&bases).await {
        return Ok("sudoers 配置已包含缓存安装规则，无需重复配置".to_string());
    }

    let line = format!("{username} ALL=(ALL) NOPASSWD: {rule}");
    // 一律追加写入：与备份安装 / 缓存清理共用同一文件，覆盖会冲掉其它规则
    Ok(build_sudoers_append_command(&rule, &line))
}
