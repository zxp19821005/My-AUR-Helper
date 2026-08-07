use log::info;
use reqwest::Client;

use crate::errors::AppResult;
use crate::models::ProxyType;

/// 从 userscript 获取的代理信息
#[derive(Debug, Clone)]
pub struct FetchedProxy {
    pub url: String,                 // 代理 URL
    pub region: Option<String>,      // 代理所在区域
    pub description: Option<String>, // 代理描述
    pub proxy_type: ProxyType,       // 代理类型
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
/// 依次解析 download_url_us、clone_url、clone_ssh_url 和 raw_url 数组
/// @param text - userscript 文件内容
/// @param proxies - 用于存放解析结果的可变引用
fn parse_userscript_arrays(text: &str, proxies: &mut Vec<FetchedProxy>) {
    // 优先匹配 download_url_us，回退到 download_url（兼容旧版脚本）
    for array_name in &["download_url_us", "download_url"] {
        if let Some(urls) = extract_array_entries(text, array_name) {
            for entry in urls {
                if let Some(mut proxy) = parse_proxy_entry(&entry) {
                    proxy.proxy_type = ProxyType::Download;
                    proxies.push(proxy);
                }
            }
            break;
        }
    }
    if let Some(urls) = extract_array_entries(text, "clone_url") {
        for entry in urls {
            if let Some(mut proxy) = parse_proxy_entry(&entry) {
                proxy.proxy_type = ProxyType::Clone;
                proxies.push(proxy);
            }
        }
    }
    // 优先匹配 clone_ssh_url，回退到 ssh_url
    for array_name in &["clone_ssh_url", "ssh_url"] {
        if let Some(urls) = extract_array_entries(text, array_name) {
            for entry in urls {
                if let Some(mut proxy) = parse_proxy_entry(&entry) {
                    proxy.proxy_type = ProxyType::Ssh;
                    proxies.push(proxy);
                }
            }
            break;
        }
    }
    if let Some(urls) = extract_array_entries(text, "raw_url") {
        for entry in urls {
            if let Some(mut proxy) = parse_proxy_entry(&entry) {
                proxy.proxy_type = ProxyType::Raw;
                proxies.push(proxy);
            }
        }
    }
}

/// 从用户脚本文本中提取指定数组的所有条目
/// 使用括号深度计数精确匹配数组边界，处理多数组声明语法
/// @param text - 脚本文本内容
/// @param array_name - 要提取的数组变量名
/// @returns 提取到的数组条目字符串列表（每个条目是 [...] 的内容）
fn extract_array_entries(text: &str, array_name: &str) -> Option<Vec<String>> {
    use regex::Regex;

    // 1. 找到数组声明起始位置：array_name = [
    let pattern = format!(r"{}\s*=\s*\[", regex::escape(array_name));
    let re = Regex::new(&pattern).ok()?;
    let m = re.find(text)?;

    // 2. 从 '[' 开始括号深度计数，找到匹配的 ']'
    let bytes = text.as_bytes();
    let start = m.start() + m.as_str().len() - 1; // '[' 的位置
    let mut depth = 1;
    let mut end = start + 1;

    while end < bytes.len() && depth > 0 {
        // 跳过单行注释（注释中的括号不应参与计数）
        if bytes[end] == b'/' && end + 1 < bytes.len() && bytes[end + 1] == b'/' {
            while end < bytes.len() && bytes[end] != b'\n' {
                end += 1;
            }
            continue;
        }
        match bytes[end] {
            b'[' => depth += 1,
            b']' => depth -= 1,
            _ => {}
        }
        if depth > 0 {
            end += 1;
        }
    }

    if depth != 0 {
        return None;
    }

    let content = &text[start + 1..end];

    // 3. 解析数组内容中的 [...] 条目（手动跟踪括号深度）
    let mut entries = Vec::new();
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;

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

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// 解析单个代理条目字符串
/// 格式：'url','region','description'（支持逗号后带空格）
/// @param entry - 用逗号分隔的代理信息字符串
/// @returns 解析得到的 FetchedProxy，失败时返回 None
fn parse_proxy_entry(entry: &str) -> Option<FetchedProxy> {
    // 先按逗号分割，再去除每个部分的引号和空白
    let parts: Vec<&str> = entry.split(',').collect();
    if parts.is_empty() {
        return None;
    }
    // 去除引号和空白，并规整为干净的代理基址
    let url = super::parse::normalize_proxy_url(
        &parts[0].trim().trim_matches('\'').trim_matches('"').to_string(),
    );
    let region = parts
        .get(1)
        .map(|s| s.trim().trim_matches('\'').trim_matches('"').to_string());
    let description = parts
        .get(2)
        .map(|s| s.trim().trim_matches('\'').trim_matches('"').to_string());
    // 过滤无效 URL：至少以 http 或 ssh 开头
    if url.is_empty() || (!url.starts_with("http") && !url.starts_with("ssh")) {
        return None;
    }
    Some(FetchedProxy {
        url,
        region,
        description,
        proxy_type: ProxyType::Download, // 默认值，在 parse_userscript_arrays 中被覆盖
    })
}
