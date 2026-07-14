/**
 * sys_command.rs - 系统命令执行
 *
 * 提供安全的系统命令执行功能
 * 已移除任意命令执行（run_command）、sudo 调用（install_package/remove_package）
 * 和未使用的命令，消除安全风险
 */
use log::debug;
use tauri::command;

use crate::errors::{AppError, AppResult};

/// 包名合法字符正则：仅允许字母、数字、@、.、_、+、-
fn is_valid_pkgname(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || "@._+-".contains(c))
}

/// 获取已安装包的版本号
#[command]
pub async fn get_package_version(pkgname: String) -> AppResult<String> {
    if !is_valid_pkgname(&pkgname) {
        return Err(AppError::InvalidInput(format!(
            "无效的包名: {}",
            pkgname
        )));
    }
    debug!("正在获取软件包版本: {}", pkgname);
    let output = tokio::process::Command::new("pacman")
        .args(["-Qi", &pkgname])
        .output()
        .await?;
    if !output.status.success() {
        return Err(AppError::PackageNotFound(format!(
            "软件包未安装: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    for line in stdout.lines() {
        if let Some(ver) = line.strip_prefix("Version        : ") {
            debug!("软件包 {} 版本: {}", pkgname, ver);
            return Ok(ver.to_string());
        }
    }
    Err(AppError::ParseError("无法解析版本号".to_string()))
}

/// 列出所有已安装的包
#[command]
pub async fn list_installed_packages() -> AppResult<Vec<String>> {
    debug!("正在列出已安装的软件包");
    let output = tokio::process::Command::new("pacman")
        .args(["-Qq"])
        .output()
        .await?;
    if !output.status.success() {
        return Err(AppError::SystemCommand(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let packages: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
    debug!("找到 {} 个已安装的软件包", packages.len());
    Ok(packages)
}
