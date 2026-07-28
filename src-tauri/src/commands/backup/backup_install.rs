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

use crate::errors::AppResult;

/// 从包文件获取软件信息（pacman -Qip）
#[tauri::command]
pub async fn get_package_file_info(full_path: String) -> AppResult<String> {
    let output = tokio::process::Command::new("pacman")
        .args(["-Qip", &full_path])
        .output()
        .await
        .map_err(|e| {
            crate::errors::AppError::SystemCommand(format!("执行 pacman 失败: {}", e))
        })?;

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
pub async fn check_sudoers_config() -> AppResult<bool> {
    // 获取当前用户名
    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| {
            crate::errors::AppError::SystemCommand(format!("获取用户名失败: {}", e))
        })?;
    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // 检查 /etc/sudoers.d/aur-helper-backup 文件是否存在
    let sudoers_path = "/etc/sudoers.d/aur-helper-backup";
    match tokio::fs::read_to_string(sudoers_path).await {
        Ok(content) => {
            // 检查文件内容是否包含当前用户的 pacman -U 免密规则
            let expected_pattern = format!("{} ALL=(ALL) NOPASSWD: /usr/bin/pacman -U", username);
            Ok(content.contains(&expected_pattern))
        }
        Err(_) => Ok(false), // 文件不存在，需要配置
    }
}

/// 获取 sudoers 配置命令
#[tauri::command]
pub async fn get_sudoers_command() -> AppResult<String> {
    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| {
            crate::errors::AppError::SystemCommand(format!("获取用户名失败: {}", e))
        })?;

    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(format!(
        "echo \"{} ALL=(ALL) NOPASSWD: /usr/bin/pacman -U *\" | sudo tee /etc/sudoers.d/aur-helper-backup",
        username
    ))
}

/// 安装备份包
#[tauri::command]
pub async fn install_backup_package(full_path: String) -> AppResult<String> {
    info!("[备份管理] 开始安装备份包: {}", full_path);

    let output = tokio::process::Command::new("sudo")
        .args(["pacman", "-U", "--noconfirm", &full_path])
        .output()
        .await
        .map_err(|e| {
            crate::errors::AppError::SystemCommand(format!("执行安装失败: {}", e))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        info!("[备份管理] 安装成功: {}", full_path);
        Ok(stdout)
    } else {
        error!("[备份管理] 安装失败: {} - {}", full_path, stderr);
        Err(crate::errors::AppError::SystemCommand(format!(
            "安装失败:\n{}",
            stderr
        )))
    }
}
