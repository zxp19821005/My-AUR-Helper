use crate::errors::AppResult;
use log::{debug, info, warn};
use reqwest::Client;

const AUR_RPC_URL: &str = "https://aur.archlinux.org/rpc/v5";

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

pub async fn fetch_packages_by_user(
    client: &Client,
    username: &str,
) -> AppResult<Vec<AurPackageData>> {
    let mut all = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for by_field in &["maintainer", "comaintainers"] {
        let url = format!("{}/search/{}?by={}", AUR_RPC_URL, username, by_field);
        debug!("请求 AUR search API: {}", url);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                if let Some(results) = data["results"].as_array() {
                    info!("  search({}) 返回 {} 条结果", by_field, results.len());
                    for item in results {
                        let pkgname = item["Name"].as_str().unwrap_or("").to_string();
                        if pkgname.is_empty() || !seen.insert(pkgname.clone()) {
                            continue;
                        }
                        debug!(
                            "  解析包: {} (仅search基础字段, info字段需二次请求)",
                            pkgname
                        );
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
        }
    }

    info!(
        "search阶段: 共获取 {} 个基础包名, 开始请求每个包的完整 info",
        all.len()
    );

    for pkg in &mut all {
        debug!("请求完整信息: {}", pkg.pkgname);
        if let Ok(Some(data)) = get_package_info(client, &pkg.pkgname).await {
            debug!(
                "info API 返回: {}",
                serde_json::to_string(&data).unwrap_or_default()
            );
            pkg.pkgdesc = data["Description"].as_str().map(|s| s.to_string());
            pkg.version = data["Version"].as_str().map(|s| s.to_string());
            pkg.url = data["URL"].as_str().map(|s| s.to_string());
            pkg.license = data["License"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            pkg.depends = data["Depends"].as_array().map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });
            pkg.makedepends = data["MakeDepends"].as_array().map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });
            pkg.optdepends = data["OptDepends"].as_array().map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });
            pkg.out_of_date = data["OutOfDate"].as_i64().map(|v| v != 0);
            pkg.last_modified = data["LastModified"].as_i64();
        }
    }

    info!("已从 AUR 获取 {} 个完整软件包信息", all.len());
    Ok(all)
}

pub async fn get_package_info(
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

pub async fn get_packages_info(
    client: &Client,
    pkgnames: &[String],
    batch_size: usize,
    batch_interval: u64,
) -> AppResult<Vec<serde_json::Value>> {
    if pkgnames.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_results = Vec::new();
    let chunks: Vec<&[String]> = pkgnames.chunks(batch_size).collect();
    let total_chunks = chunks.len();

    for (i, chunk) in chunks.iter().enumerate() {
        if i > 0 {
            info!(
                "[AUR 批量查询] 等待 {} 秒后继续下一批 ({}/{})",
                batch_interval,
                i + 1,
                total_chunks
            );
            tokio::time::sleep(std::time::Duration::from_secs(batch_interval)).await;
        }

        let url = format!("{}/info/", AUR_RPC_URL);
        let mut body = String::new();
        for name in *chunk {
            body.push_str(&format!("arg[]={}&", name));
        }
        body.pop();

        info!(
            "[AUR 批量查询] 第 {}/{} 批：查询 {} 个包",
            i + 1,
            total_chunks,
            chunk.len()
        );
        debug!("[AUR 批量查询] 请求体长度: {} 字节", body.len());

        match client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                debug!("[AUR 批量查询] 响应状态码: {}", status);

                if !status.is_success() {
                    warn!("[AUR 批量查询] 请求失败: HTTP {}", status);
                    continue;
                }

                match resp.text().await {
                    Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                        Ok(data) => {
                            let resultcount = data["resultcount"].as_i64().unwrap_or(0);
                            debug!("[AUR 批量查询] 返回 {} 个结果", resultcount);

                            if let Some(error) = data["error"].as_str() {
                                warn!("[AUR 批量查询] AUR 返回错误: {}", error);
                                continue;
                            }

                            if let Some(results) = data["results"].as_array() {
                                all_results.extend(results.iter().cloned());
                            }
                        }
                        Err(e) => {
                            warn!("[AUR 批量查询] JSON 解析失败: {}", e);
                            debug!("[AUR 批量查询] 响应内容: {}", &text[..text.len().min(500)]);
                        }
                    },
                    Err(e) => {
                        warn!("[AUR 批量查询] 读取响应体失败: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("[AUR 批量查询] 请求失败: {}", e);
            }
        }
    }

    info!("[AUR 批量查询] 完成，共获取 {} 个结果", all_results.len());
    Ok(all_results)
}
