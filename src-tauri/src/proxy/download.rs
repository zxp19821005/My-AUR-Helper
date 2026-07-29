/**
 * download.rs - 代理文件下载模块
 *
 * 功能：
 * - 从配置的 URL 下载代理规则 JS 文件
 * - 保存到本地临时目录
 * - 支持进度提示和异常捕获
 */
use log::info;
use reqwest::Client;
use std::path::PathBuf;
use tokio::fs;

use crate::errors::AppResult;

/// 获取配置目录路径
fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.zxp19821005.aur-helper")
}

/// 获取临时目录路径
fn get_tmp_dir() -> PathBuf {
    get_config_dir().join("tmp")
}

/// 获取代理文件保存路径
pub fn get_proxy_file_path() -> PathBuf {
    get_tmp_dir().join("proxy_rules.js")
}

/// 下载代理文件
/// 从配置的 URL 下载代理规则 JS 文件到本地临时目录
/// @param client - HTTP 客户端
/// @param download_url - 下载地址
/// @returns 下载的文件路径
pub async fn download_proxy_file(client: &Client, download_url: &str) -> AppResult<PathBuf> {
    // 确保临时目录存在
    let tmp_dir = get_tmp_dir();
    fs::create_dir_all(&tmp_dir).await?;

    // 下载文件
    info!("开始下载代理文件: {}", download_url);
    let resp = client.get(download_url).send().await?;

    // 检查响应状态
    if !resp.status().is_success() {
        return Err(crate::errors::AppError::NetworkError(format!(
            "下载失败，状态码: {}",
            resp.status()
        )));
    }

    // 获取文件内容
    let content = resp.text().await?;

    // 保存到本地
    let file_path = get_proxy_file_path();
    fs::write(&file_path, &content).await?;

    info!("代理文件已保存到: {:?}", file_path);
    Ok(file_path)
}
