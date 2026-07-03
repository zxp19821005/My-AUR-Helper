use anyhow::Result;    // 通用错误处理
use reqwest::Client;   // HTTP 客户端，用于发送 API 请求
use log::info;         // 日志记录

/// AUR RPC API 的基础 URL（v5 接口）
const AUR_RPC_URL: &str = "https://aur.archlinux.org/rpc/v5";

/// AUR 包数据结构
/// 对应 AUR RPC v12 接口返回的包信息字段
#[derive(Debug, Clone)]
pub struct AurPackageData {
    pub pkgname: String,                          // 包名
    pub pkgdesc: Option<String>,                  // 包描述
    pub version: Option<String>,                  // 当前 AUR 中的版本号
    pub url: Option<String>,                      // 项目主页 URL
    pub license: Option<String>,                  // 许可证（取数组第一个）
    pub depends: Option<Vec<String>>,             // 运行时依赖
    pub makedepends: Option<Vec<String>>,         // 构建依赖
    pub optdepends: Option<Vec<String>>,          // 可选依赖
    pub out_of_date: Option<bool>,                // 是否标记为过期
}

/// 通过用户名获取 AUR 包列表（包括维护者和共同维护者）
/// @param client - 复用的 HTTP 客户端
/// @param username - AUR 用户名
/// @returns 该用户维护或共同维护的所有包的数据列表
pub async fn fetch_packages_by_user(client: &Client, username: &str) -> Result<Vec<AurPackageData>> {
    let mut all = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 分别查询维护者和共同维护者，合并去重
    for by_field in &["maintainer", "comaintainers"] {
        let url = format!("{}/search/{}?by={}", AUR_RPC_URL, username, by_field);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                if let Some(results) = data["results"].as_array() {
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
                            license: item["License"].as_array()
                                .and_then(|a| a.first())
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            depends: item["Depends"].as_array()
                                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
                            makedepends: item["MakeDepends"].as_array()
                                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
                            optdepends: item["OptDepends"].as_array()
                                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
                            out_of_date: item["OutOfDate"].as_i64().map(|v| v != 0),
                        });
                    }
                }
            }
        }
    }
    info!("Fetched {} packages from AUR (maintainer + comaintainer)", all.len());
    Ok(all)
}

/// 获取单个 AUR 包信息
/// @param client - 复用的 HTTP 客户端
/// @param pkgname - 要查询的 AUR 包名
/// @returns 包信息的 JSON Value（包含完整原始数据），如果包不存在则返回 None
pub async fn get_package_info(client: &Client, pkgname: &str) -> Result<Option<serde_json::Value>> {
    // 构建 AUR RPC info 接口 URL
    let url = format!("{}/info/{}", AUR_RPC_URL, pkgname);
    let resp = client.get(&url).send().await?;
    let data: serde_json::Value = resp.json().await?;
    // 检查结果数量，大于 0 说明找到了包
    if data["resultcount"].as_i64().unwrap_or(0) > 0 {
        Ok(data["results"].as_array().and_then(|a| a.first().cloned()))
    } else {
        Ok(None)
    }
}
