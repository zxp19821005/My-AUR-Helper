use anyhow::Result;
use reqwest::Client;

/// 测试代理延迟
pub async fn test_proxy_latency(client: &Client, proxy_url: &str) -> Result<i64> {
    let test_url = format!("{}/https://github.com", proxy_url.trim_end_matches('/'));
    let start = std::time::Instant::now();
    let _resp = client.head(&test_url).send().await?;
    let latency = start.elapsed().as_millis() as i64;
    Ok(latency)
}
