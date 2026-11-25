use crate::errors::{AppError, AppResult};
use crate::network::retry::{retry_with_backoff, DEFAULT_MAX_RETRIES};
use log::{debug, info, warn};
use reqwest::Client;

pub(crate) const AUR_RPC_URL: &str = "https://aur.archlinux.org/rpc/v5";

#[derive(Debug, Clone)]
pub struct AurPackageData {
    pub pkgname: String,
    pub pkgdesc: Option<String>,
    pub version: Option<String>,
    pub url: Option<String>,
    pub license: Option<String>,
    pub depends: Option<Vec<String>>,
    pub makedepends: Option<Vec<String>>,
    pub optdepends: Option<Vec<String>>,
    pub out_of_date: Option<bool>,
    pub last_modified: Option<i64>,
}

/// 判定一次 AUR 响应是否被反爬中间件（Anubis 等）拦截
///
/// 拦截特征：HTTP 200 但返回的不是 JSON（HTML 拦截页），或带非空 error 字段。
fn response_is_blocked(content_type: &str, text: &str) -> bool {
    let json = content_type.to_ascii_lowercase().contains("json");
    let html = text.trim_start().starts_with('<');
    !json || html
}

/// 使用指数退避重试，按维护者用户名拉取其维护的全部包名
///
/// 只做两次 search（maintainer + comaintainers），拿到去重后的包名列表即可，
/// 不再逐包请求 info —— 逐包 info 在包数量大（如上千个）时会触发
/// AUR 官方 4000 req/day/IP 限流与 Anubis 拦截，是此前「同步包数量不对」的根因之一。
/// 完整字段（依赖/许可证等）由调用方用批量 info 或元数据归档统一补全。
///
/// # 参数
/// - `client`: 已配置 User-Agent 的 HTTP 客户端
/// - `username`: AUR 维护者用户名
///
/// # 返回
/// - `Ok(Vec<AurPackageData>)`: 去重后的包名及 search 阶段能拿到的基础字段
pub async fn fetch_packages_by_user(
    client: &Client,
    username: &str,
) -> AppResult<Vec<AurPackageData>> {
    retry_with_backoff(DEFAULT_MAX_RETRIES, || {
        let client = client.clone();
        let username = username.to_string();
        async move { do_fetch_packages_by_user(&client, &username).await }
    })
    .await
}

async fn do_fetch_packages_by_user(
    client: &Client,
    username: &str,
) -> AppResult<Vec<AurPackageData>> {
    let mut all = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut succeeded = 0usize;

    for by_field in &["maintainer", "comaintainers"] {
        let url = format!("{}/search/{}?by={}", AUR_RPC_URL, username, by_field);
        debug!("请求 AUR search API: {}", url);
        let resp = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("  search({}) 请求失败: {}", by_field, e);
                continue;
            }
        };
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let status = resp.status();
        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                warn!("  search({}) 读取响应失败: {}", by_field, e);
                continue;
            }
        };
        if !status.is_success() || response_is_blocked(&content_type, &text) {
            warn!(
                "  search({}) 响应异常或被反爬拦截 (HTTP {}, content-type: {})",
                by_field, status, content_type
            );
            continue;
        }
        let data: serde_json::Value = match serde_json::from_str(&text) {
            Ok(d) => d,
            Err(e) => {
                warn!("  search({}) JSON 解析失败: {}", by_field, e);
                continue;
            }
        };
        if let Some(error) = data["error"].as_str() {
            if !error.is_empty() {
                warn!("  search({}) AUR 返回错误: {}", by_field, error);
                continue;
            }
        }
        if let Some(results) = data["results"].as_array() {
            succeeded += 1;
            info!("  search({}) 返回 {} 条结果", by_field, results.len());
            for item in results {
                let pkgname = item["Name"].as_str().unwrap_or("").to_string();
                if pkgname.is_empty() || !seen.insert(pkgname.clone()) {
                    continue;
                }
                all.push(AurPackageData {
                    pkgname,
                    pkgdesc: item["Description"].as_str().map(|s| s.to_string()),
                    version: item["Version"].as_str().map(|s| s.to_string()),
                    url: item["URL"].as_str().map(|s| s.to_string()),
                    license: None,
                    depends: None,
                    makedepends: None,
                    optdepends: None,
                    out_of_date: item["OutOfDate"].as_i64().map(|v| v != 0),
                    last_modified: item["LastModified"].as_i64(),
                });
            }
        }
    }

    // 两次 search 全部失败时明确报错，避免"返回 0 个包"被误判为"用户没有维护包"
    if succeeded == 0 {
        return Err(AppError::NetworkError(
            "AUR search 请求全部失败（可能被反爬拦截或网络异常），请检查 User-Agent 与网络/代理"
                .to_string(),
        ));
    }

    info!("search 阶段共获取 {} 个去重后的包名", all.len());
    Ok(all)
}

/// 使用指数退避重试获取单个包的 AUR 信息
pub async fn get_package_info(
    client: &Client,
    pkgname: &str,
) -> AppResult<Option<serde_json::Value>> {
    retry_with_backoff(DEFAULT_MAX_RETRIES, || {
        let client = client.clone();
        let pkgname = pkgname.to_string();
        async move { do_get_package_info(&client, &pkgname).await }
    })
    .await
}

async fn do_get_package_info(
    client: &Client,
    pkgname: &str,
) -> AppResult<Option<serde_json::Value>> {
    let url = format!("{}/info/{}", AUR_RPC_URL, pkgname);
    debug!("请求 AUR info API: {}", url);
    let resp = client.get(&url).send().await?;
    let data: serde_json::Value = resp.json().await?;
    if data["resultcount"].as_i64().unwrap_or(0) > 0 {
        let result = data["results"].as_array().and_then(|a| a.first().cloned());
        if result.is_some() {
            debug!("  info API 返回成功");
        }
        Ok(result)
    } else {
        debug!("  info API 返回空结果");
        Ok(None)
    }
}

/// 搜索 AUR 软件包（支持模糊搜索）
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `keyword`: 搜索关键字
///
/// # 返回
/// - `Vec<serde_json::Value>`: 匹配的包信息列表
pub async fn search_packages(client: &Client, keyword: &str) -> AppResult<Vec<serde_json::Value>> {
    let url = format!("{}/search/?search={}&type=info", AUR_RPC_URL, keyword);
    debug!("请求 AUR search API: {}", url);
    let resp = client.get(&url).send().await?;
    let data: serde_json::Value = resp.json().await?;
    if data["resultcount"].as_i64().unwrap_or(0) > 0 {
        let results = data["results"].as_array().cloned().unwrap_or_default();
        debug!("  search API 返回 {} 条结果", results.len());
        Ok(results)
    } else {
        debug!("  search API 返回空结果");
        Ok(Vec::new())
    }
}
