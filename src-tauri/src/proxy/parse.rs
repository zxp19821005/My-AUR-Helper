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

/// 从 URL 提取代理名称（公开供其他模块复用）
pub fn extract_proxy_name(url: &str) -> String {
    // 尝试从 URL 中提取有意义的名称
    if let Some(domain) = url.split("://").nth(1) {
        if let Some(first_part) = domain.split('/').next() {
            return first_part.to_string();
        }
    }
    url.to_string()
}

/// 规整代理 URL 为干净的代理基址
/// 部分用户脚本条目会把下载地址（如 `/https://github.com` 或 `?https://github.com`）
/// 直接拼在代理后面。干净的代理基址只应包含一个协议头；若第一个 `://` 之后
/// 再次出现 `://`，则说明那是已拼接的下载地址后缀，从其起点截断即可，
/// 从而覆盖 `/https://`、`?https://` 等各种后缀形式，避免测试时产生重复地址。
pub fn normalize_proxy_url(url: &str) -> String {
    let url = url.trim();
    let after_scheme = match url.find("://") {
        Some(p) => p + 3,
        None => return url.trim_end_matches('/').to_string(),
    };
    let cut = url[after_scheme..]
        .find("https://")
        .or_else(|| url[after_scheme..].find("http://"))
        .map(|rel| after_scheme + rel)
        .unwrap_or(url.len());
    let base = &url[..cut];
    base.trim_end_matches(|c| c == '/' || c == '?')
        .to_string()
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

    // 解析 download_url_us 数组
    for array_name in &["download_url_us", "download_url"] {
        if let Some(download_urls) = extract_array(content, array_name) {
            for url in download_urls {
                let url = normalize_proxy_url(&url.trim().trim_matches('\'').trim_matches('"').to_string());
                if !url.is_empty() && url.starts_with("http") {
                    proxies.push(ProxyInfo {
                        proxy_id: None,
                        proxy_name: extract_proxy_name(&url),
                        proxy_type: ProxyType::Download,
                        url,
                        is_active: true,
                success_count: 0,
                fail_count: 0,
                avg_latency: None,
                last_test_status: None,
            });
                }
            }
            break; // 匹配到 download_url_us 就不再尝试 download_url
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
                    success_count: 0,
                    fail_count: 0,
                    avg_latency: None,
                    last_test_status: None,
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
                    success_count: 0,
                    fail_count: 0,
                    avg_latency: None,
                    last_test_status: None,
                });
            }
        }
    }

    // 解析 clone_ssh_url 数组（并尝试兼容旧名称 ssh_url）
    for array_name in &["clone_ssh_url", "ssh_url"] {
        if let Some(ssh_urls) = extract_array(content, array_name) {
            for url in ssh_urls {
                let url = normalize_proxy_url(&url.trim().trim_matches('\'').trim_matches('"').to_string());
                if !url.is_empty() && url.starts_with("ssh") {
                    proxies.push(ProxyInfo {
                        proxy_id: None,
                        proxy_name: extract_proxy_name(&url),
                        proxy_type: ProxyType::Ssh,
                        url,
                        is_active: true,
                success_count: 0,
                fail_count: 0,
                avg_latency: None,
                last_test_status: None,
            });
                }
            }
            break; // 匹配到 clone_ssh_url 就不再尝试 ssh_url
        }
    }

    Ok(proxies)
}

/// 从 JavaScript 内容中提取指定数组的所有字符串元素
/// 使用状态机精确处理字符串和注释中的括号
/// @param content - JavaScript 文件内容
/// @param array_name - 要提取的数组变量名（如 "download_url_us"）
/// @returns 提取到的字符串列表
fn extract_array(content: &str, array_name: &str) -> Option<Vec<String>> {
    use regex::Regex;

    // 1. 找到数组声明起始位置：array_name = [
    let pattern = format!(r"{}\s*=\s*\[", regex::escape(array_name));
    let re = Regex::new(&pattern).ok()?;
    let m = re.find(content)?;

    // 2. 状态机：从 '[' 开始精确跟踪括号匹配
    //    跟踪状态：字符串内、注释内、括号深度
    let bytes = content.as_bytes();
    let start = m.start() + m.as_str().len() - 1; // '[' 的位置
    let mut i = start + 1;
    let mut depth = 1;
    let mut in_string = false;
    let mut string_quote: u8 = 0;

    while i < bytes.len() && depth > 0 {
        let ch = bytes[i];

        // 处理注释（// 到行尾，仅当不在字符串中）
        if !in_string && ch == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // 处理字符串
        if ch == b'\'' || ch == b'"' {
            if in_string && ch == string_quote {
                in_string = false;
            } else if !in_string {
                in_string = true;
                string_quote = ch;
            }
            i += 1;
            continue;
        }

        // 处理转义字符（在字符串中）
        if in_string && ch == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }

        // 统计括号（仅在字符串和注释之外）
        if !in_string {
            if ch == b'[' {
                depth += 1;
            } else if ch == b']' {
                depth -= 1;
            }
        }

        if depth > 0 {
            i += 1;
        }
    }

    if depth != 0 {
        return None; // 括号不匹配
    }

    let array_content = &content[start + 1..i];

    // 3. 解析数组内容中的字符串字面量
    let mut items = Vec::new();
    let bytes = array_content.as_bytes();
    let len = bytes.len();
    let mut pos = 0;

    while pos < len {
        // 跳过空白字符
        if bytes[pos] == b' ' || bytes[pos] == b'\n' || bytes[pos] == b'\r' || bytes[pos] == b'\t' {
            pos += 1;
            continue;
        }

        // 跳过注释行
        if bytes[pos] == b'/' && pos + 1 < len && bytes[pos + 1] == b'/' {
            while pos < len && bytes[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }

        // 找到字符串的开始
        if bytes[pos] == b'\'' || bytes[pos] == b'"' {
            let quote = bytes[pos];
            let s = pos + 1;
            pos += 1;

            // 找到字符串的结束（处理转义）
            while pos < len && bytes[pos] != quote {
                if bytes[pos] == b'\\' && pos + 1 < len {
                    pos += 2;
                } else {
                    pos += 1;
                }
            }

            if pos < len {
                let item = String::from_utf8_lossy(&bytes[s..pos]).to_string();
                items.push(item);
                pos += 1;
            }
        } else {
            pos += 1;
        }
    }

    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proxy_counts_from_file() {
        // 读取实际的代理规则文件
        let content = std::fs::read_to_string(
            "/home/zxp-archlinux/.config/com.zxp19821005.aur-helper/tmp/proxy_rules.js"
        ).expect("无法读取代理规则文件");

        let proxies = parse_js_content(&content).expect("解析失败");

        // 按类型统计
        let download: Vec<_> = proxies.iter().filter(|p| p.proxy_type == ProxyType::Download).collect();
        let clone: Vec<_> = proxies.iter().filter(|p| p.proxy_type == ProxyType::Clone).collect();
        let raw: Vec<_> = proxies.iter().filter(|p| p.proxy_type == ProxyType::Raw).collect();
        let ssh: Vec<_> = proxies.iter().filter(|p| p.proxy_type == ProxyType::Ssh).collect();

        println!("总代理数: {}", proxies.len());
        println!("  下载代理: {} (download)", download.len());
        println!("  克隆代理: {} (clone)", clone.len());
        println!("  RAW代理: {} (raw)", raw.len());
        println!("  SSH代理: {} (ssh)", ssh.len());

        // 验证预期数量
        assert_eq!(download.len(), 30, "下载代理应为 30 个");
        assert_eq!(clone.len(), 5, "克隆代理应为 5 个");
        assert_eq!(raw.len(), 8, "RAW 代理应为 8 个");
        assert_eq!(ssh.len(), 1, "SSH 代理应为 1 个");
        assert_eq!(proxies.len(), 44, "总代理数应为 44");
    }

    #[test]
    fn test_comment_entries_are_not_parsed() {
        let content = std::fs::read_to_string(
            "/home/zxp-archlinux/.config/com.zxp19821005.aur-helper/tmp/proxy_rules.js"
        ).expect("无法读取代理规则文件");

        let proxies = parse_js_content(&content).expect("解析失败");

        // 验证被注释掉的条目确实没有被导入
        let commented_urls = vec![
            "https://gh.api.99988866.xyz/https://github.com",
            "https://hub.glowp.xyz/https://github.com",
            "https://gitdl.cn/https://github.com",
            "https://gitproxy.click/https://github.com",
            "https://cdn.moran233.xyz/https://github.com",
        ];

        for commented_url in commented_urls {
            assert!(
                !proxies.iter().any(|p| p.url == commented_url),
                "被注释掉的 URL 不应被导入: {}",
                commented_url
            );
        }
    }
}
