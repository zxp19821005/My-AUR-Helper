/**
 * backup_install.rs - 备份包安装和信息查询命令
 *
 * 功能：
 * - get_package_file_info: 获取包文件详细信息
 * - check_sudoers_config: 检测 sudoers 免密配置
 * - get_sudoers_command: 生成 sudoers 配置命令
 * - install_backup_package: 安装备份包
 */
use log::{error, info};
use std::path::PathBuf;
use tauri::State;

use crate::errors::{AppError, AppResult};
use crate::AppState;

/// 合法的 pacman 包文件扩展名
const PKG_EXTENSIONS: &[&str] = &[
    ".pkg.tar.zst",
    ".pkg.tar.xz",
    ".pkg.tar",
    ".tar.zst",
    ".tar.xz",
];

/// 校验前端传入的包文件路径，防止路径遍历导致的任意文件读取 / 以 root 安装恶意包。
///
/// 校验项：
/// 1. 必须是绝对路径；
/// 2. 扩展名必须是合法的 pacman 包扩展名；
/// 3. 规范化（解析符号链接与 `..`）后必须是真实存在的普通文件；
/// 4. 规范化路径必须落在允许的根目录（备份目录或系统缓存目录）内。
///
/// @param full_path - 待校验的路径
/// @param allowed_roots - 允许的根目录（绝对路径）
/// @returns 校验通过的规范化路径
fn validate_package_path(full_path: &str, allowed_roots: &[PathBuf]) -> AppResult<PathBuf> {
    let raw = std::path::Path::new(full_path);
    if !raw.is_absolute() {
        return Err(AppError::InvalidInput(format!(
            "路径必须是绝对路径: {}",
            full_path
        )));
    }

    if !PKG_EXTENSIONS.iter().any(|ext| full_path.ends_with(ext)) {
        return Err(AppError::InvalidInput(format!(
            "仅允许操作 pacman 包文件: {}",
            full_path
        )));
    }

    let canon = std::fs::canonicalize(raw).map_err(|e| {
        AppError::InvalidInput(format!("无法访问路径 {}: {}", full_path, e))
    })?;
    if !canon.is_file() {
        return Err(AppError::InvalidInput(format!(
            "路径不是普通文件: {}",
            full_path
        )));
    }

    // 根目录本身可能含符号链接，逐一规范化后再做前缀比较
    let roots: Vec<PathBuf> = allowed_roots
        .iter()
        .map(|r| std::fs::canonicalize(r).unwrap_or_else(|_| r.clone()))
        .collect();
    if !roots.iter().any(|root| canon.starts_with(root)) {
        return Err(AppError::InvalidInput(format!(
            "路径不在允许的备份目录内: {}",
            full_path
        )));
    }

    Ok(canon)
}

/// 读取备份目录设置，缺失或为空时回退到默认路径
fn read_backup_dir(db: &impl std::ops::Deref<Target = crate::db::Database>) -> String {
    db.get_setting("backup_dir")
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/run/media/zxp/Backup/Linux/ZST".to_string())
}

/// 从包文件获取软件信息（pacman -Qip）
///
/// 路径必须经 `validate_package_path` 校验，防止读取任意文件。
#[tauri::command]
pub async fn get_package_file_info(
    state: State<'_, AppState>,
    full_path: String,
) -> AppResult<String> {
    let allowed_roots = {
        let db = state.db.lock().map_err(|e| {
            AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
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
        let db = state.db.lock().map_err(|e| {
            AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        read_backup_dir(&db)
    };

    // 检查 /etc/sudoers.d/aur-helper-backup 文件是否存在
    let sudoers_path = "/etc/sudoers.d/aur-helper-backup";
    match tokio::fs::read_to_string(sudoers_path).await {
        Ok(content) => {
            // 允许「限定备份目录」与「旧版通配 *」两种写法，保证向后兼容
            let scoped = format!(
                "{} ALL=(ALL) NOPASSWD: /usr/bin/pacman -U {}",
                username, backup_dir
            );
            let legacy = format!("{} ALL=(ALL) NOPASSWD: /usr/bin/pacman -U *", username);
            Ok(content.contains(&scoped) || content.contains(&legacy))
        }
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
        let db = state.db.lock().map_err(|e| {
            AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        read_backup_dir(&db)
    };

    Ok(format!(
        "echo \"{} ALL=(ALL) NOPASSWD: /usr/bin/pacman -U {}/*\" | sudo tee /etc/sudoers.d/aur-helper-backup",
        username, backup_dir
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
        let db = state.db.lock().map_err(|e| {
            AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;
        vec![
            PathBuf::from(read_backup_dir(&db)),
            PathBuf::from("/var/cache/pacman/pkg"),
        ]
    };
    let safe_path = validate_package_path(&full_path, &allowed_roots)?;

    info!("[备份管理] 开始安装备份包: {}", safe_path.display());

    let output = tokio::process::Command::new("sudo")
        .args(["pacman", "-U", "--noconfirm", &safe_path.to_string_lossy()])
        .output()
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
        let _ = std::fs::create_dir_all(&tmp);
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
        let _ = std::fs::create_dir_all(&root);

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
        let _ = std::fs::create_dir_all(&tmp);
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
