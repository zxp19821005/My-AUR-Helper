use anyhow::Result;
use reqwest::Client;
use log::info;

/// 获取的代理信息
#[derive(Debug, Clone)]
pub struct FetchedProxy {
    pub url: String,
    pub region: Option<String>,
    pub description: Option<String>,
}

const PROXY_SOURCE_URL: &str =
    "https://update.greasyfork.org/scripts/412245/Github%20%E5%A2%9E%E5%BC%BA%20-%20%E9%AB%98%E9%80%9F%E4%B8%8B%E8%BD%BD.user.js";

/// 从 userscript 获取代理列表
pub async fn fetch_proxy_list_from_userscript(client: &Client) -> Result<Vec<FetchedProxy>> {
    let resp = client.get(PROXY_SOURCE_URL).send().await?;
    let text = resp.text().await?;
    let mut proxies = Vec::new();
    parse_userscript_arrays(&text, &mut proxies);
    info!("Fetched {} proxies from userscript", proxies.len());
    Ok(proxies)
}

/// 解析 userscript 中的数组
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

/// 提取数组条目
fn extract_array_entries(text: &str, array_name: &str) -> Option<Vec<String>> {
    use regex::Regex;
    let pattern = format!(r"(?s){} = \[(.*?)\];", regex::escape(array_name));
    let re = Regex::new(&pattern).ok()?;
    let captures = re.captures(text)?;
    let content = captures.get(1)?.as_str();

    let mut entries = Vec::new();
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b' ' || bytes[i] == b'\n' || bytes[i] == b'\r' || bytes[i] == b'\t' {
            i += 1;
            continue;
        }
        if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'[' {
            let mut depth = 1;
            let entry_start = i + 1;
            let mut entry_end = entry_start;
            let mut found = false;
            let mut j = entry_start;
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

/// 解析代理条目
fn parse_proxy_entry(entry: &str) -> Option<FetchedProxy> {
    let parts: Vec<&str> = entry.splitn(3, "','").collect();
    if parts.is_empty() {
        return None;
    }
    let url = parts[0].trim().trim_matches('\'').to_string();
    let region = parts.get(1).map(|s| s.trim().trim_matches('\'').to_string());
    let description = parts.get(2).map(|s| s.trim().trim_matches('\'').to_string());
    if url.is_empty() {
        return None;
    }
    Some(FetchedProxy { url, region, description })
}
