use reqwest::Client;

use crate::errors::AppResult;

/// 测试代理的延迟
/// 向代理的 GitHub 地址发送 HEAD 请求，测量响应时间
pub async fn test_proxy_latency(client: &Client, proxy_url: &str) -> AppResult<i64> {
    let test_url = format!("{}/https://github.com", proxy_url.trim_end_matches('/'));
    let start = std::time::Instant::now();
    let _resp = client.head(&test_url).send().await?;
    let latency = start.elapsed().as_millis() as i64;
    Ok(latency)
}
