/**
 * parse.rs - 代理文件解析模块
 *
 * 功能：
 * - 读取已下载的代理规则 JS 文件
 * - 解析代理规则并提取有效代理信息
 * - 将解析结果写入数据库
 */
use log::info;
use tokio::fs;

use crate::errors::{AppError, AppResult};
use crate::models::{ProxyInfo, ProxyType};

use super::download::get_proxy_file_path;

/// 从 URL 提取代理名称
fn extract_proxy_name(url: &str) -> String {
    // 尝试从 URL 中提取有意义的名称
    if let Some(domain) = url.split("://").nth(1) {
        if let Some(first_part) = domain.split('/').next() {
            return first_part.to_string();
        }
    }
    url.to_string()
}

/// 解析代理文件
/// 读取已下载的代理规则 JS 文件，解析其中的代理信息
/// @returns 解析得到的代理信息列表
pub async fn parse_proxy_file() -> AppResult<Vec<ProxyInfo>> {
    let file_path = get_proxy_file_path();

    // 检查文件是否存在
    if !file_path.exists() {
        return Err(AppError::FileNotFound(format!(
            "代理文件不存在: {:?}",
            file_path
        )));
    }

    // 读取文件内容
    let content = fs::read_to_string(&file_path)
        .await
        .map_err(|e| AppError::FileOperation(format!("读取代理文件失败: {}", e)))?;

    // 解析代理信息
    let proxies = parse_js_content(&content)?;

    info!("从代理文件解析出 {} 个代理", proxies.len());
    Ok(proxies)
}

/// 解析 JavaScript 内容中的代理信息
fn parse_js_content(content: &str) -> AppResult<Vec<ProxyInfo>> {
    let mut proxies = Vec::new();

    // 解析 download_url 数组
    if let Some(download_urls) = extract_array(content, "download_url") {
        for url in download_urls {
            let url = url.trim().trim_matches('\'').trim_matches('"').to_string();
            if !url.is_empty() && url.starts_with("http") {
                proxies.push(ProxyInfo {
                    proxy_id: None,
                    proxy_name: extract_proxy_name(&url),
                    proxy_type: ProxyType::Download,
                    url,
                    is_active: true,
                });
            }
        }
    }

    // 解析 clone_url 数组
    if let Some(clone_urls) = extract_array(content, "clone_url") {
        for url in clone_urls {
            let url = url.trim().trim_matches('\'').trim_matches('"').to_string();
            if !url.is_empty() && url.starts_with("http") {
                proxies.push(ProxyInfo {
                    proxy_id: None,
                    proxy_name: extract_proxy_name(&url),
                    proxy_type: ProxyType::Clone,
                    url,
                    is_active: true,
                });
            }
        }
    }

    // 解析 raw_url 数组
    if let Some(raw_urls) = extract_array(content, "raw_url") {
        for url in raw_urls {
            let url = url.trim().trim_matches('\'').trim_matches('"').to_string();
            if !url.is_empty() && url.starts_with("http") {
                proxies.push(ProxyInfo {
                    proxy_id: None,
                    proxy_name: extract_proxy_name(&url),
                    proxy_type: ProxyType::Raw,
                    url,
                    is_active: true,
                });
            }
        }
    }

    // 解析 ssh_url 数组（如果存在）
    if let Some(ssh_urls) = extract_array(content, "ssh_url") {
        for url in ssh_urls {
            let url = url.trim().trim_matches('\'').trim_matches('"').to_string();
            if !url.is_empty() && url.starts_with("ssh") {
                proxies.push(ProxyInfo {
                    proxy_id: None,
                    proxy_name: extract_proxy_name(&url),
                    proxy_type: ProxyType::Ssh,
                    url,
                    is_active: true,
                });
            }
        }
    }

    Ok(proxies)
}

/// 从 JavaScript 内容中提取数组
fn extract_array(content: &str, array_name: &str) -> Option<Vec<String>> {
    use regex::Regex;

    // 匹配 JavaScript 数组赋值语法
    let pattern = format!(r"(?s){}\s*=\s*\[(.*?)\];", regex::escape(array_name));
    let re = Regex::new(&pattern).ok()?;
    let captures = re.captures(content)?;
    let array_content = captures.get(1)?.as_str();

    // 解析数组内容
    let mut items = Vec::new();
    let bytes = array_content.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // 跳过空白字符
        if bytes[i] == b' ' || bytes[i] == b'\n' || bytes[i] == b'\r' || bytes[i] == b'\t' {
            i += 1;
            continue;
        }

        // 跳过注释
        if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // 找到字符串的开始
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let quote = bytes[i];
            let start = i + 1;
            i += 1;

            // 找到字符串的结束
            while i < len && bytes[i] != quote {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2; // 跳过转义字符
                } else {
                    i += 1;
                }
            }

            if i < len {
                let item = String::from_utf8_lossy(&bytes[start..i]).to_string();
                items.push(item);
                i += 1; // 跳过结束引号
            }
        } else {
            i += 1;
        }
    }

    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}
