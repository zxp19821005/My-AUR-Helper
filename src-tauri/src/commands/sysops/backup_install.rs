/**
 * backup_install.rs - 备份包安装和信息查询命令
 *
 * 功能：
 * - get_package_file_info: 获取包文件详细信息
 * - check_sudoers_config: 检测 sudoers 免密配置
 * - get_sudoers_command: 生成 sudoers 配置命令
 * - install_backup_package: 安装备份包
 *
 * 路径校验与 sudoers 规则辅助函数见同目录 backup_install_helpers.rs
 * （以 pub(crate) 再导出，供 cache_install / cache_cleanup 跨模块复用）。
 * 系统子进程与 sudoers 公共方法见 system_cmd.rs / sudoers.rs。
 */
/// 路径校验与 sudoers 规则辅助函数（定义于同目录 backup_install_helpers.rs，
/// 在 sysops/mod.rs 中声明为同级模块，此处再导出供 cache_install / cache_cleanup 引用）
pub(crate) use super::backup_install_helpers::*;

use log::info;
use std::path::PathBuf;
use tauri::State;

use crate::commands::sysops::sudoers::{build_sudoers_append_command, is_rule_effective};
use crate::commands::sysops::system_cmd::{
    current_username, install_package_locked, pacman_package_info,
};
use crate::db::lock_db;
use crate::errors::AppResult;
use crate::AppState;

/// 从包文件获取软件信息（pacman -Qip）
///
/// 路径必须经 `validate_package_path` 校验，防止读取任意文件。
#[tauri::command]
#[specta::specta]
pub async fn get_package_file_info(
    state: State<'_, AppState>,
    full_path: String,
) -> AppResult<String> {
    let allowed_roots = {
        let db = lock_db(&state.db)?;
        vec![
            PathBuf::from(read_backup_dir(&db)),
            PathBuf::from("/var/cache/pacman/pkg"),
        ]
    };
    let safe_path = validate_package_path(&full_path, &allowed_roots)?;

    pacman_package_info(&safe_path).await
}

/// 检测 sudoers 免密配置是否可用
///
/// /etc/sudoers.d 目录权限为 0750，应用进程无法直接读取该文件；
/// 旧实现 `read_to_string` 必然失败并永远返回 false（一直误报「未配置」）。
/// 改为解析 `sudo -n -l` 输出的、当前用户实际生效的 NOPASSWD 规则。
#[tauri::command]
#[specta::specta]
pub async fn check_sudoers_config(state: State<'_, AppState>) -> AppResult<bool> {
    let backup_dir = {
        let db = lock_db(&state.db)?;
        read_backup_dir(&db)
    };

    let base = format!(
        "/usr/bin/pacman -U --noconfirm {}",
        backup_dir.trim_end_matches('/')
    );
    Ok(is_rule_effective(&base).await)
}

/// 获取 sudoers 配置命令
///
/// 将 pacman -U 的免密规则限定到备份目录（而非通配 *），缩小提权面。
/// 使用 `tee -a` 追加：该文件同时承载缓存安装与缓存清理规则，覆盖写会冲掉它们。
#[tauri::command]
#[specta::specta]
pub async fn get_sudoers_command(state: State<'_, AppState>) -> AppResult<String> {
    let username = current_username().await?;

    let backup_dir = {
        let db = lock_db(&state.db)?;
        read_backup_dir(&db)
    };

    let rule = build_pacman_install_rules(&backup_dir);
    let line = format!("{username} ALL=(ALL) NOPASSWD: {rule}");
    // 单次 sudo 追加，不覆盖缓存安装 / 缓存清理的规则；grep 保证幂等
    Ok(build_sudoers_append_command(&rule, &line))
}

/// 安装备份包
///
/// 路径必须经 `validate_package_path` 校验，防止以 root 安装任意路径的恶意包。
#[tauri::command]
#[specta::specta]
pub async fn install_backup_package(
    state: State<'_, AppState>,
    full_path: String,
) -> AppResult<String> {
    let allowed_roots = {
        let db = lock_db(&state.db)?;
        vec![
            PathBuf::from(read_backup_dir(&db)),
            PathBuf::from("/var/cache/pacman/pkg"),
        ]
    };
    let safe_path = validate_package_path(&full_path, &allowed_roots)?;

    info!("[备份管理] 开始安装备份包: {}", safe_path.display());

    install_package_locked(&safe_path, "[备份管理]").await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 校验：合法扩展名 + 落在允许根目录内的文件应通过
    #[test]
    fn validate_accepts_pkg_inside_root() {
        let tmp = std::env::temp_dir().join("bk_install_test_root");
        std::fs::create_dir_all(&tmp).expect("创建测试目录失败");
        let file = tmp.join("foo-1.0.0-1-x86_64.pkg.tar.zst");
        std::fs::write(&file, b"fake").unwrap();

        let roots = vec![tmp.clone()];
        let result = validate_package_path(&file.to_string_lossy(), &roots);
        assert!(result.is_ok(), "应放行备份目录内的合法包文件");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// 校验：根目录外的文件应被拒绝（防路径遍历/任意读取）
    #[test]
    fn validate_rejects_pkg_outside_root() {
        let root = std::env::temp_dir().join("bk_install_test_root_only");
        std::fs::create_dir_all(&root).expect("创建测试根目录失败");

        let outside = std::env::temp_dir().join("bk_install_test_outside.pkg.tar.zst");
        std::fs::write(&outside, b"fake").unwrap();

        let roots = vec![root.clone()];
        let result = validate_package_path(&outside.to_string_lossy(), &roots);
        assert!(result.is_err(), "应拒绝根目录外的文件");

        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_file(&outside);
    }

    /// 校验：非 pacman 包扩展名应被拒绝
    #[test]
    fn validate_rejects_non_pkg_extension() {
        let tmp = std::env::temp_dir().join("bk_install_test_ext");
        std::fs::create_dir_all(&tmp).expect("创建测试目录失败");
        let file = tmp.join("evil.sh");
        std::fs::write(&file, b"fake").unwrap();

        let roots = vec![tmp.clone()];
        let result = validate_package_path(&file.to_string_lossy(), &roots);
        assert!(result.is_err(), "应拒绝非包文件扩展名");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// 校验：相对路径应被拒绝
    #[test]
    fn validate_rejects_relative_path() {
        let roots = vec![std::path::PathBuf::from("/tmp")];
        let result = validate_package_path("relative/path.pkg.tar.zst", &roots);
        assert!(result.is_err(), "应拒绝相对路径");
    }
}
