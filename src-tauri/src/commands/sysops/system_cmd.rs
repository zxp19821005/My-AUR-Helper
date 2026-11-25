/**
 * system_cmd.rs - 系统命令公共封装（whoami / pacman -Qip / sudo pacman -U）
 *
 * 功能：
 * - current_username: 异步读取当前用户名（whoami）
 * - pacman_package_info: 查询本地 pacman 包文件元信息（pacman -Qip）
 * - install_package_locked: 在全局写锁保护下安装本地包（sudo pacman -U --noconfirm）
 *
 * 工作流程：
 * 1. 调用方先用 backup_install_helpers::validate_package_path 校验路径白名单；
 * 2. 再把规范化后的 &Path 交给本模块，避免在公共方法里重复校验；
 * 3. 安装类命令统一经 pacman_lock::with_pacman_write_lock 串行化，防止并发争抢 db.lck。
 *
 * 适用场景：备份安装 / 缓存安装两类 Tauri 命令共用的系统级子进程调用。
 * 全部基于 tokio::process::Command 异步实现，禁止替换为 std::process::Command，
 * 以免在 tokio worker 线程上阻塞 runtime；命令参数与抽取前逐字一致。
 */
use std::path::Path;

use log::{error, info};

use crate::commands::sysops::pacman_lock::with_pacman_write_lock;
use crate::errors::{AppError, AppResult};

/// 异步读取当前用户名（等价于 `whoami` 的 stdout 去首尾空白）。
///
/// 与抽取前保持一致：不额外校验退出码，仅取 stdout 并 trim。
///
/// # 返回
/// - `Ok(String)`: 当前用户名
/// - `Err(AppError::SystemCommand)`: 子进程无法启动
pub async fn current_username() -> AppResult<String> {
    let output = tokio::process::Command::new("whoami")
        .output()
        .await
        .map_err(|e| AppError::SystemCommand(format!("获取用户名失败: {}", e)))?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// 查询本地 pacman 包文件的元信息（`pacman -Qip <path>`）。
///
/// 路径必须由调用方事先通过 `validate_package_path` 校验，本函数不重复校验。
///
/// # 参数
/// - `package_path`: 已通过白名单校验的本地包文件路径
///
/// # 返回
/// - `Ok(String)`: pacman 的标准输出文本
/// - `Err(AppError::SystemCommand)`: 子进程启动失败或 pacman 返回非零退出码
pub async fn pacman_package_info(package_path: &Path) -> AppResult<String> {
    let output = tokio::process::Command::new("pacman")
        .args(["-Qip", &package_path.to_string_lossy()])
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

/// 在全局 pacman 写锁保护下安装本地包（`sudo pacman -U --noconfirm <path>`）。
///
/// 命令参数与抽取前逐字一致；`with_pacman_write_lock` 保证任意时刻只有一个
/// pacman 写事务运行，避免争抢 `/var/lib/pacman/db.lck`。
/// 成功/失败日志与统一错误格式在此收敛，调用点仅保留各自的「开始安装」日志。
///
/// # 参数
/// - `package_path`: 已通过白名单校验的本地包文件路径
/// - `log_context`: 日志前缀（如 `"[备份管理]"` / `"[缓存管理]"`），保留原有日志语境
///
/// # 返回
/// - `Ok(String)`: pacman 的标准输出
/// - `Err(AppError::SystemCommand)`: 子进程启动失败或安装失败
pub async fn install_package_locked(package_path: &Path, log_context: &str) -> AppResult<String> {
    let output = with_pacman_write_lock(|| async {
        tokio::process::Command::new("sudo")
            .args([
                "pacman",
                "-U",
                "--noconfirm",
                &package_path.to_string_lossy(),
            ])
            .output()
            .await
    })
    .await
    .map_err(|e| AppError::SystemCommand(format!("执行安装失败: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        info!("{} 安装成功: {}", log_context, package_path.display());
        Ok(stdout)
    } else {
        error!(
            "{} 安装失败: {} - {}",
            log_context,
            package_path.display(),
            stderr
        );
        Err(AppError::SystemCommand(format!("安装失败:\n{}", stderr)))
    }
}
