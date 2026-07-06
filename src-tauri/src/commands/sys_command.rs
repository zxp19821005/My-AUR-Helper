/**
 * sys_command.rs - 系统命令执行
 *
 * 提供系统命令的执行功能
 * 包括：运行命令、安装/卸载软件包、清理缓存等
 */
use log::{debug, info};
use std::path::Path;
use tauri::command;
use tokio::process::Command;

use crate::errors::{AppError, AppResult};

/// 命令输出结果
#[derive(serde::Serialize)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// 执行任意系统命令
#[command]
pub async fn run_command(command: String, args: Vec<String>) -> AppResult<CommandOutput> {
    debug!("正在执行命令: {} {:?}", command, args);
    let output = Command::new(&command).args(&args).output().await?;
    let exit_code = output.status.code().unwrap_or(-1);
    debug!("命令退出码: {}", exit_code);
    Ok(CommandOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// 通过 pacman 安装软件包
#[command]
pub async fn install_package(pkgname: String) -> AppResult<CommandOutput> {
    info!("正在安装软件包: {}", pkgname);
    let output = Command::new("sudo")
        .args(["pacman", "-S", "--noconfirm", &pkgname])
        .output()
        .await?;
    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// 通过 pacman 卸载软件包
#[command]
pub async fn remove_package(pkgname: String) -> AppResult<CommandOutput> {
    info!("正在卸载软件包: {}", pkgname);
    let output = Command::new("sudo")
        .args(["pacman", "-R", "--noconfirm", &pkgname])
        .output()
        .await?;
    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// 清理 pacman 缓存
#[command]
pub async fn clean_cache(_keep_versions: usize) -> AppResult<CommandOutput> {
    info!("正在清理 pacman 缓存");
    let output = Command::new("sudo")
        .args(["pacman", "-Sc", "--noconfirm"])
        .output()
        .await?;
    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// 获取已安装包的版本号
#[command]
pub async fn get_package_version(pkgname: String) -> AppResult<String> {
    debug!("正在获取软件包版本: {}", pkgname);
    let output = Command::new("pacman")
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
    let output = Command::new("pacman").args(["-Qq"]).output().await?;
    if !output.status.success() {
        return Err(AppError::SystemCommand(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let packages: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
    info!("找到 {} 个已安装的软件包", packages.len());
    Ok(packages)
}

/// 同步 pacman 数据库
#[command]
pub async fn sync_database() -> AppResult<CommandOutput> {
    info!("正在同步 pacman 数据库");
    let output = Command::new("sudo")
        .args(["pacman", "-Sy"])
        .output()
        .await?;
    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// 在指定目录运行 makepkg 构建
#[command]
pub async fn makepkg(directory: String, args: Vec<String>) -> AppResult<CommandOutput> {
    info!("正在目录 {} 中运行 makepkg，参数: {:?}", directory, args);
    let dir = Path::new(&directory);
    if !dir.exists() {
        return Err(AppError::FileNotFound(format!("目录不存在: {}", directory)));
    }
    let output = Command::new("makepkg")
        .args(&args)
        .current_dir(dir)
        .output()
        .await?;
    let exit_code = output.status.code().unwrap_or(-1);
    info!("makepkg 退出码: {}", exit_code);
    Ok(CommandOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
