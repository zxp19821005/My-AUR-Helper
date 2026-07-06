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
    debug!("正在执行命令: {} {:?}", command, args);
    let output = Command::new(&command)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("执行 {} 失败: {}", command, e))?;
    let exit_code = output.status.code().unwrap_or(-1);
    debug!("命令退出码: {}", exit_code);
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
    info!("正在安装软件包: {}", pkgname);
    let output = Command::new("sudo")
        .args(["pacman", "-S", "--noconfirm", &pkgname])
        .output()
        .await
        .map_err(|e| format!("执行 pacman 失败: {}", e))?;
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
    info!("正在卸载软件包: {}", pkgname);
    let output = Command::new("sudo")
        .args(["pacman", "-R", "--noconfirm", &pkgname])
        .output()
        .await
        .map_err(|e| format!("执行 pacman 失败: {}", e))?;
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
    info!("正在清理 pacman 缓存");
    let output = Command::new("sudo")
        .args(["pacman", "-Sc", "--noconfirm"])
        .output()
        .await
        .map_err(|e| format!("执行 pacman 失败: {}", e))?;
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
    debug!("正在获取软件包版本: {}", pkgname);
    let output = Command::new("pacman")
        .args(["-Qi", &pkgname])
        .output()
        .await
        .map_err(|e| format!("执行 pacman 失败: {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "软件包未安装: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    // 解析 pacman -Qi 输出中的 Version 字段
    for line in stdout.lines() {
        if let Some(ver) = line.strip_prefix("Version        : ") {
            debug!("软件包 {} 版本: {}", pkgname, ver);
            return Ok(ver.to_string());
        }
    }
    Err("无法解析版本号".to_string())
}

/// 列出所有已安装的包
/// @returns 已安装包名列表
#[command]
pub async fn list_installed_packages() -> Result<Vec<String>, String> {
    debug!("正在列出已安装的软件包");
    let output = Command::new("pacman")
        .args(["-Qq"])
        .output()
        .await
        .map_err(|e| format!("执行 pacman 失败: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let packages: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
    info!("找到 {} 个已安装的软件包", packages.len());
    Ok(packages)
}

/// 同步 pacman 数据库
/// @returns 命令执行结果
#[command]
pub async fn sync_database() -> Result<CommandOutput, String> {
    info!("正在同步 pacman 数据库");
    let output = Command::new("sudo")
        .args(["pacman", "-Sy"])
        .output()
        .await
        .map_err(|e| format!("执行 pacman 失败: {}", e))?;
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
    info!("正在目录 {} 中运行 makepkg，参数: {:?}", directory, args);
    let dir = Path::new(&directory);
    if !dir.exists() {
        return Err(format!("目录不存在: {}", directory));
    }
    let output = Command::new("makepkg")
        .args(&args)
        .current_dir(dir) // 在指定目录中执行
        .output()
        .await
        .map_err(|e| format!("执行 makepkg 失败: {}", e))?;
    let exit_code = output.status.code().unwrap_or(-1);
    info!("makepkg 退出码: {}", exit_code);
    Ok(CommandOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
