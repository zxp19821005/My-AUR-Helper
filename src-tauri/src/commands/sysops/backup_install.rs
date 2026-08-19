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
 */
/// 路径校验与 sudoers 规则辅助函数（定义于同目录 backup_install_helpers.rs，
/// 在 sysops/mod.rs 中声明为同级模块，此处再导出供 cache_install / cache_cleanup 引用）
pub(crate) use super::backup_install_helpers::*;

use log::{error, info};
use std::path::PathBuf;
use tauri::State;

use crate::commands::sysops::pacman_lock::with_pacman_write_lock;
use crate::errors::{AppError, AppResult};
use crate::AppState;

/// 从包文件获取软件信息（pacman -Qip）
///
/// 路径必须经 `validate_package_path` 校验，防止读取任意文件。
#[tauri::command]
pub async fn get_package_file_info(
    state: State<'_, AppState>,
    full_path: String,
) -> AppResult<String> {
    let allowed_roots = {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
        vec![
            PathBuf::from(read_backup_dir(&db)),
            PathBuf::from("/var/cache/pacman/pkg"),
        ]
    };
    let safe_path = validate_package_path(&full_path, &allowed_roots)?;

    let output = tokio::process::Command::new("pacman")
        .args(["-Qip", &safe_path.to_string_lossy()])
        .output()
        .await
        .map_err(|e| AppError::SystemCommand(format!("执行 pacman 失败: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::errors::AppError::SystemCommand(format!(
            "pacman -Qip 失败: {}",
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 检测 sudoers 免密配置是否可用
///
/// 检查逻辑：
/// 1. 检查 /etc/sudoers.d/aur-helper-backup 文件是否存在
/// 2. 检查文件内容是否包含当前用户的 pacman -U 免密规则
/// 3. 如果文件存在且内容正确，返回 true（无需显示提示）
#[tauri::command]
pub async fn check_sudoers_config(state: State<'_, AppState>) -> AppResult<bool> {
    // 获取当前用户名
    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| AppError::SystemCommand(format!("获取用户名失败: {}", e)))?;
    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let backup_dir = {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
        read_backup_dir(&db)
    };

    // 检查 /etc/sudoers.d/aur-helper-backup 文件是否存在
    let sudoers_path = "/etc/sudoers.d/aur-helper-backup";
    match tokio::fs::read_to_string(sudoers_path).await {
        Ok(content) => Ok(has_pacman_install_rule(&content, &username, &backup_dir)),
        Err(_) => Ok(false), // 文件不存在，需要配置
    }
}

/// 获取 sudoers 配置命令
///
/// 将 pacman -U 的免密规则限定到备份目录（而非通配 *），缩小提权面。
#[tauri::command]
pub async fn get_sudoers_command(state: State<'_, AppState>) -> AppResult<String> {
    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| AppError::SystemCommand(format!("获取用户名失败: {}", e)))?;

    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let backup_dir = {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
        read_backup_dir(&db)
    };

    let rule = build_pacman_install_rules(&backup_dir);
    Ok(format!(
        "echo \"{} ALL=(ALL) NOPASSWD: {}\" | sudo tee /etc/sudoers.d/aur-helper-backup",
        username, rule
    ))
}

/// 安装备份包
///
/// 路径必须经 `validate_package_path` 校验，防止以 root 安装任意路径的恶意包。
#[tauri::command]
pub async fn install_backup_package(
    state: State<'_, AppState>,
    full_path: String,
) -> AppResult<String> {
    let allowed_roots = {
        let db = state
            .db
            .lock()
            .map_err(|e| AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?;
        vec![
            PathBuf::from(read_backup_dir(&db)),
            PathBuf::from("/var/cache/pacman/pkg"),
        ]
    };
    let safe_path = validate_package_path(&full_path, &allowed_roots)?;

    info!("[备份管理] 开始安装备份包: {}", safe_path.display());

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
        info!("[备份管理] 安装成功: {}", safe_path.display());
        Ok(stdout)
    } else {
        error!("[备份管理] 安装失败: {} - {}", safe_path.display(), stderr);
        Err(crate::errors::AppError::SystemCommand(format!(
            "安装失败:\n{}",
            stderr
        )))
    }
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
