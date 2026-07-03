/**
 * sys_command.rs - 系统命令执行
 *
 * 提供系统命令的执行功能
 * 包括：运行命令、安装/卸载软件包、清理缓存等
 */
use log::{info, debug};     // 日志记录
use std::path::Path;         // 文件路径操作
use tauri::command;          // Tauri 命令宏
use tokio::process::Command; // 异步系统命令执行

/// 命令输出结果
#[derive(serde::Serialize)]
pub struct CommandOutput {
    pub exit_code: i32,     // 进程退出码
    pub stdout: String,     // 标准输出内容
    pub stderr: String,     // 标准错误内容
}

/// 执行任意系统命令
/// @param command - 要执行的命令名
/// @param args - 命令参数列表
/// @returns 命令执行结果（退出码、标准输出、标准错误）
#[command]
pub async fn run_command(command: String, args: Vec<String>) -> Result<CommandOutput, String> {
    debug!("Running command: {} {:?}", command, args);
    let output = Command::new(&command)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("Failed to execute {}: {}", command, e))?;
    let exit_code = output.status.code().unwrap_or(-1);
    debug!("Command exited with code: {}", exit_code);
    Ok(CommandOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// 通过 pacman 安装软件包
/// @param pkgname - 要安装的包名
/// @returns 命令执行结果
#[command]
pub async fn install_package(pkgname: String) -> Result<CommandOutput, String> {
    info!("Installing package: {}", pkgname);
    let output = Command::new("sudo")
        .args(["pacman", "-S", "--noconfirm", &pkgname])
        .output()
        .await
        .map_err(|e| format!("Failed to run pacman: {}", e))?;
    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// 通过 pacman 卸载软件包
/// @param pkgname - 要卸载的包名
/// @returns 命令执行结果
#[command]
pub async fn remove_package(pkgname: String) -> Result<CommandOutput, String> {
    info!("Removing package: {}", pkgname);
    let output = Command::new("sudo")
        .args(["pacman", "-R", "--noconfirm", &pkgname])
        .output()
        .await
        .map_err(|e| format!("Failed to run pacman: {}", e))?;
    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// 清理 pacman 缓存
/// @param _keep_versions - 保留的版本数量（当前未使用，默认清理全部）
/// @returns 命令执行结果
#[command]
pub async fn clean_cache(_keep_versions: usize) -> Result<CommandOutput, String> {
    info!("Cleaning pacman cache");
    let output = Command::new("sudo")
        .args(["pacman", "-Sc", "--noconfirm"])
        .output()
        .await
        .map_err(|e| format!("Failed to run pacman: {}", e))?;
    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// 获取已安装包的版本号
/// @param pkgname - 包名
/// @returns 版本号字符串（如 "1.2.3-1"）
#[command]
pub async fn get_package_version(pkgname: String) -> Result<String, String> {
    debug!("Getting version for package: {}", pkgname);
    let output = Command::new("pacman")
        .args(["-Qi", &pkgname])
        .output()
        .await
        .map_err(|e| format!("Failed to run pacman: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "Package not installed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    // 解析 pacman -Qi 输出中的 Version 字段
    for line in stdout.lines() {
        if let Some(ver) = line.strip_prefix("Version        : ") {
            debug!("Package {} version: {}", pkgname, ver);
            return Ok(ver.to_string());
        }
    }
    Err("Could not parse version".to_string())
}

/// 列出所有已安装的包
/// @returns 已安装包名列表
#[command]
pub async fn list_installed_packages() -> Result<Vec<String>, String> {
    debug!("Listing installed packages");
    let output = Command::new("pacman")
        .args(["-Qq"])
        .output()
        .await
        .map_err(|e| format!("Failed to run pacman: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let packages: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
    info!("Found {} installed packages", packages.len());
    Ok(packages)
}

/// 同步 pacman 数据库
/// @returns 命令执行结果
#[command]
pub async fn sync_database() -> Result<CommandOutput, String> {
    info!("Syncing pacman database");
    let output = Command::new("sudo")
        .args(["pacman", "-Sy"])
        .output()
        .await
        .map_err(|e| format!("Failed to run pacman: {}", e))?;
    Ok(CommandOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// 在指定目录运行 makepkg 构建
/// @param directory - PKGBUILD 所在目录
/// @param args - makepkg 参数列表
/// @returns 命令执行结果
#[command]
pub async fn makepkg(directory: String, args: Vec<String>) -> Result<CommandOutput, String> {
    info!("Running makepkg in: {} with args: {:?}", directory, args);
    let dir = Path::new(&directory);
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", directory));
    }
    let output = Command::new("makepkg")
        .args(&args)
        .current_dir(dir) // 在指定目录中执行
        .output()
        .await
        .map_err(|e| format!("Failed to run makepkg: {}", e))?;
    let exit_code = output.status.code().unwrap_or(-1);
    info!("makepkg exited with code: {}", exit_code);
    Ok(CommandOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
