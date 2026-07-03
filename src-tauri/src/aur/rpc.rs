use anyhow::Result;    // 通用错误处理
use reqwest::Client;   // HTTP 客户端，用于发送 API 请求
use log::info;         // 日志记录

/// AUR RPC API 的基础 URL
const AUR_RPC_URL: &str = "https://aur.archlinux.org/rpc/v12";

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
    pub provides: Option<Vec<String>>,            // 提供的虚拟包
    pub conflicts: Option<Vec<String>>,           // 冲突的包
    pub replaces: Option<Vec<String>>,            // 替换的包
    pub votes: Option<i64>,                       // 投票数
    pub popularity: Option<f64>,                  // 人气值
    pub out_of_date: Option<bool>,                // 是否标记为过期
    pub submitted_by: Option<String>,             // 提交者用户名
    pub maintainers: Option<Vec<String>>,         // 维护者列表
}

/// 通过维护者名获取 AUR 包列表
/// @param client - 复用的 HTTP 客户端
/// @param username - AUR 维护者的用户名
/// @returns 该维护者维护的所有包的数据列表
pub async fn fetch_packages_by_maintainer(client: &Client, username: &str) -> Result<Vec<AurPackageData>> {
    // 构建 AUR RPC 搜索 URL，按维护者查询
    let url = format!("{}/search/{}?by=maintainer", AUR_RPC_URL, username);
    let resp = client.get(&url).send().await?;          // 发送 HTTP GET 请求
    let data: serde_json::Value = resp.json().await?;  // 解析 JSON 响应
    let mut packages = Vec::new();
    // 遍历结果数组，提取每个包的信息
    if let Some(results) = data["results"].as_array() {
        for item in results {
            let pkgname = item["Name"].as_str().unwrap_or("").to_string();
            if pkgname.is_empty() {
                continue; // 跳过无名称的条目
            }
            // 将 JSON 字段映射到 AurPackageData 结构体
            packages.push(AurPackageData {
                pkgname,
                pkgdesc: item["Description"].as_str().map(|s| s.to_string()),
                version: item["Version"].as_str().map(|s| s.to_string()),
                url: item["URL"].as_str().map(|s| s.to_string()),
                // License 字段是数组，取第一个元素
                license: item["License"].as_array()
                    .and_then(|a| a.first())
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                // 数组字段，过滤出所有字符串元素
                depends: item["Depends"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
                makedepends: item["MakeDepends"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
                optdepends: item["OptDepends"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
                provides: item["Provides"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
                conflicts: item["Conflicts"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
                replaces: item["Replaces"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
                votes: item["NumVotes"].as_i64(),
                popularity: item["Popularity"].as_f64(),
                // OutOfDate 为非 0 表示已过期
                out_of_date: item["OutOfDate"].as_i64().map(|v| v != 0),
                submitted_by: item["SubmittedBy"].as_str().map(|s| s.to_string()),
                maintainers: item["Maintainers"].as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()),
            });
        }
    }
    info!("Fetched {} packages from AUR", packages.len());
    Ok(packages)
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
