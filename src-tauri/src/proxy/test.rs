/**
 * test.rs - 代理延迟测试模块
 *
 * 功能：
 * - 测试代理的延迟和连通性
 * - 支持按代理类型使用不同的测试方法
 * - 记录测试结果到数据库
 */
use reqwest::Client;

use crate::errors::{AppError, AppResult};
use crate::models::ProxyType;

/// 测试代理的延迟（通用方法）
/// 向代理的 GitHub 地址发送 HEAD 请求，测量响应时间
/// 仅当响应状态码为 2xx 时视为成功
pub async fn test_proxy_latency(client: &Client, proxy_url: &str) -> AppResult<i64> {
    let test_url = format!("{}/https://github.com", proxy_url.trim_end_matches('/'));
    let start = std::time::Instant::now();
    let resp = client.head(&test_url).send().await?;
    let latency = start.elapsed().as_millis() as i64;
    let status = resp.status();
    if !status.is_success() {
        return Err(AppError::NetworkError(format!(
            "代理返回错误状态码: {}",
            status
        )));
    }
    Ok(latency)
}

/// 根据代理类型测试代理
/// @param client - HTTP 客户端
/// @param proxy_url - 代理 URL
/// @param proxy_type - 代理类型
/// @param test_url - 测试地址（可选，如果为 None 则使用默认测试地址）
/// @returns 测试结果（延迟毫秒数）
pub async fn test_proxy_by_type(
    client: &Client,
    proxy_url: &str,
    proxy_type: &ProxyType,
    test_url: Option<&str>,
) -> AppResult<i64> {
    match proxy_type {
        ProxyType::Download => {
            // 下载代理：使用 HEAD 请求测试
            let url = test_url.unwrap_or("https://github.com/zxp19821005/My_AUR_Files/releases/latest/download/README.md");
            let full_url = format!("{}/{}", proxy_url.trim_end_matches('/'), url.trim_start_matches('/'));
            let start = std::time::Instant::now();
            let resp = client.head(&full_url).send().await?;
            let latency = start.elapsed().as_millis() as i64;
            let status = resp.status();
            if !status.is_success() {
                return Err(AppError::NetworkError(format!(
                    "下载代理测试失败，状态码: {}",
                    status
                )));
            }
            Ok(latency)
        }
        ProxyType::Clone => {
            // 克隆代理：使用 HEAD 请求测试
            let url = test_url.unwrap_or("https://github.com/zxp19821005/My_AUR_Files.git");
            let full_url = format!("{}/{}", proxy_url.trim_end_matches('/'), url.trim_start_matches('/'));
            let start = std::time::Instant::now();
            let resp = client.head(&full_url).send().await?;
            let latency = start.elapsed().as_millis() as i64;
            let status = resp.status();
            if !status.is_success() {
                return Err(AppError::NetworkError(format!(
                    "克隆代理测试失败，状态码: {}",
                    status
                )));
            }
            Ok(latency)
        }
        ProxyType::Raw => {
            // RAW 代理：使用 GET 请求测试
            let url = test_url.unwrap_or("https://raw.githubusercontent.com/zxp19821005/My_AUR_Files/main/README.md");
            let full_url = format!("{}/{}", proxy_url.trim_end_matches('/'), url.trim_start_matches('/'));
            let start = std::time::Instant::now();
            let resp = client.get(&full_url).send().await?;
            let latency = start.elapsed().as_millis() as i64;
            let status = resp.status();
            if !status.is_success() {
                return Err(AppError::NetworkError(format!(
                    "RAW 代理测试失败，状态码: {}",
                    status
                )));
            }
            Ok(latency)
        }
        ProxyType::Ssh => {
            // SSH 代理：使用 HEAD 请求测试（通过 HTTPS 网关）
            let _url = test_url.unwrap_or("ssh://git@ssh.github.com:443/zxp19821005/My_AUR_Files");
            // SSH 代理通常需要特殊处理，这里简化为测试连通性
            let start = std::time::Instant::now();
            let resp = client.head(proxy_url).send().await?;
            let latency = start.elapsed().as_millis() as i64;
            let status = resp.status();
            if !status.is_success() {
                return Err(AppError::NetworkError(format!(
                    "SSH 代理测试失败，状态码: {}",
                    status
                )));
            }
            Ok(latency)
        }
    }
}
