use reqwest::Client;
use log::info;

use crate::errors::AppResult;

/// 从 userscript 获取的代理信息
#[derive(Debug, Clone)]
pub struct FetchedProxy {
    pub url: String,               // 代理 URL
    pub region: Option<String>,    // 代理所在区域
    pub description: Option<String>, // 代理描述
}

/// Greasyfork 上 GitHub 加速用户脚本的 URL
const PROXY_SOURCE_URL: &str =
    "https://update.greasyfork.org/scripts/412245/Github%20%E5%A2%9E%E5%BC%BA%20-%20%E9%AB%98%E9%80%9F%E4%B8%8B%E8%BD%BD.user.js";

/// 从 Greasyfork 用户脚本获取代理列表
/// 解析 userscript 中的 download_url, clone_url, raw_url 数组
/// @param client - 复用的 HTTP 客户端
/// @returns 解析得到的代理列表
pub async fn fetch_proxy_list_from_userscript(client: &Client) -> AppResult<Vec<FetchedProxy>> {
    let resp = client.get(PROXY_SOURCE_URL).send().await?;
    let text = resp.text().await?;
    let mut proxies = Vec::new();
    // 从脚本文本中解析三个数组
    parse_userscript_arrays(&text, &mut proxies);
    info!("已从用户脚本获取 {} 个代理", proxies.len());
    Ok(proxies)
}

/// 解析 userscript 中的代理数组
/// 依次解析 download_url、clone_url 和 raw_url 数组
/// @param text - userscript 文件内容
/// @param proxies - 用于存放解析结果的可变引用
fn parse_userscript_arrays(text: &str, proxies: &mut Vec<FetchedProxy>) {
    if let Some(urls) = extract_array_entries(text, "download_url") {
        for entry in urls {
            if let Some(proxy) = parse_proxy_entry(&entry) {
                proxies.push(proxy);
            }
        }
    }
    if let Some(urls) = extract_array_entries(text, "clone_url") {
        for entry in urls {
            if let Some(proxy) = parse_proxy_entry(&entry) {
                proxies.push(proxy);
            }
        }
    }
    if let Some(urls) = extract_array_entries(text, "raw_url") {
        for entry in urls {
            if let Some(proxy) = parse_proxy_entry(&entry) {
                proxies.push(proxy);
            }
        }
    }
}

/// 从用户脚本文本中提取指定数组的所有条目
/// 处理 JavaScript 数组语法，包括嵌套和注释
/// @param text - 脚本文本内容
/// @param array_name - 要提取的数组变量名
/// @returns 提取到的数组条目字符串列表
fn extract_array_entries(text: &str, array_name: &str) -> Option<Vec<String>> {
    use regex::Regex;
    // 匹配 JavaScript 数组赋值语法：array_name = [...];
    let pattern = format!(r"(?s){} = \[(.*?)\];", regex::escape(array_name));
    let re = Regex::new(&pattern).ok()?;
    let captures = re.captures(text)?;
    let content = captures.get(1)?.as_str();

    let mut entries = Vec::new();
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // 手动解析数组条目，处理嵌套的 []
    while i < len {
        // 跳过空白字符
        if bytes[i] == b' ' || bytes[i] == b'\n' || bytes[i] == b'\r' || bytes[i] == b'\t' {
            i += 1;
            continue;
        }
        // 跳过单行注释
        if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // 遇到 [ 开始解析一个条目
        if bytes[i] == b'[' {
            let mut depth = 1;
            let entry_start = i + 1;
            let mut entry_end = entry_start;
            let mut found = false;
            let mut j = entry_start;
            // 跟踪括号深度，找到匹配的 ]
            while j < len && depth > 0 {
                match bytes[j] {
                    b'[' => depth += 1,
                    b']' => {
                        depth -= 1;
                        if depth == 0 {
                            entry_end = j;
                            found = true;
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            if found {
                entries.push(content[entry_start..entry_end].to_string());
                i = entry_end + 1;
            } else {
                i = j + 1;
            }
            continue;
        }
        i += 1;
    }

    if entries.is_empty() { None } else { Some(entries) }
}

/// 解析单个代理条目字符串
/// 格式：'url','region','description'
/// @param entry - 用逗号分隔的代理信息字符串
/// @returns 解析得到的 FetchedProxy，失败时返回 None
fn parse_proxy_entry(entry: &str) -> Option<FetchedProxy> {
    // 按 ',' 分割，最多 3 段
    let parts: Vec<&str> = entry.splitn(3, "','").collect();
    if parts.is_empty() {
        return None;
    }
    // 去除引号和空白
    let url = parts[0].trim().trim_matches('\'').to_string();
    let region = parts.get(1).map(|s| s.trim().trim_matches('\'').to_string());
    let description = parts.get(2).map(|s| s.trim().trim_matches('\'').to_string());
    if url.is_empty() {
        return None;
    }
    Some(FetchedProxy { url, region, description })
}
