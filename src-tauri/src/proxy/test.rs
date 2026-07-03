use anyhow::Result;    // 通用错误处理
use reqwest::Client;   // HTTP 客户端

/// 测试代理的延迟
/// 向代理的 GitHub 地址发送 HEAD 请求，测量响应时间
/// @param client - 复用的 HTTP 客户端
/// @param proxy_url - 代理服务器 URL
/// @returns 延迟毫秒数
pub async fn test_proxy_latency(client: &Client, proxy_url: &str) -> Result<i64> {
    // 构造测试 URL：代理地址 + GitHub 根路径
    let test_url = format!("{}/https://github.com", proxy_url.trim_end_matches('/'));
    let start = std::time::Instant::now();       // 记录开始时间
    let _resp = client.head(&test_url).send().await?; // 发送 HEAD 请求（轻量级）
    let latency = start.elapsed().as_millis() as i64; // 计算耗时毫秒数
    Ok(latency)
}
