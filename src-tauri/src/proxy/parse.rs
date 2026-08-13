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

use self::parse_array::extract_array;
use super::download::get_proxy_file_path;

mod parse_array; // 代理 JS 数组提取子模块
#[cfg(test)]
mod parse_tests; // 代理解析单元测试子模块

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

/// 规整代理 URL 为干净的源站基址（scheme://host[:port]），并返回代理的「目标协议头约定」。
///
/// 用户脚本条目可能已把下载地址后缀拼在代理后面，约定隐含在条目里：
/// - 若后缀含协议头（如 `...com/https://github.com`、`...cn/?https://github.com`）
///   → 该代理约定【保留】目标协议头（返回值第二个元素 = false）
/// - 若后缀为裸主机（如 `...cc/github.com`）→ 该代理约定【去除】目标协议头（= true）
///
/// 无论哪种，源站基址只保留 scheme://host[:port]，杜绝双重拼接（如 github.com/https://github.com）。
pub fn normalize_proxy_url(url: &str) -> (String, bool) {
    let url = url.trim();
    let after_scheme = match url.find("://") {
        Some(p) => p + 3,
        None => {
            return (
                url.trim_end_matches(|c| c == '/' || c == '?').to_string(),
                false,
            )
        }
    };
    let rest = &url[after_scheme..];
    // 源站之后第一个路径/查询分隔符，分隔符之后即为已拼接的下载地址后缀
    match rest.find('/').or_else(|| rest.find('?')) {
        None => (url.to_string(), false), // 干净源站、无分隔符 → 默认保留目标协议头
        Some(r) => {
            let suffix = &rest[r + 1..];
            if suffix.is_empty() {
                // 仅末尾多余 / 或 ?（如 https://host/），视为干净源站
                (url[..after_scheme + r].to_string(), false)
            } else {
                // 后缀是否携带协议头，决定目标协议头约定
                let strip_target_protocol =
                    !(suffix.contains("https://") || suffix.contains("http://"));
                (url[..after_scheme + r].to_string(), strip_target_protocol)
            }
        }
    }
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

/// 从单条原始脚本条目构造代理信息
/// 负责去引号/空白、规整为源站基址、推断「目标协议头约定」、按类型过滤无效 URL。
/// @param raw - 原始条目字符串（如 'https://cors.isteed.cc/github.com'）
/// @param proxy_type - 代理类型
/// @returns 规整后的代理信息，无效则返回 None
fn make_proxy(raw: &str, proxy_type: ProxyType) -> Option<ProxyInfo> {
    let raw = raw.trim().trim_matches('\'').trim_matches('"');
    if raw.is_empty() {
        return None;
    }
    let (url, strip_target_protocol) = normalize_proxy_url(raw);
    // 按类型校验协议头前缀：SSH 必须 ssh://，其余必须 http(s)://
    let expected = match proxy_type {
        ProxyType::Ssh => "ssh",
        _ => "http",
    };
    if !url.starts_with(expected) {
        return None;
    }
    Some(ProxyInfo {
        proxy_id: None,
        proxy_name: extract_proxy_name(&url),
        proxy_type,
        url,
        is_active: true,
        success_count: 0,
        fail_count: 0,
        avg_latency: None,
        last_test_status: None,
        strip_target_protocol,
    })
}

/// 解析 JavaScript 内容中的代理信息
fn parse_js_content(content: &str) -> AppResult<Vec<ProxyInfo>> {
    let mut proxies = Vec::new();

    // 解析 download_url_us 数组（回退到 download_url）
    for array_name in &["download_url_us", "download_url"] {
        if let Some(download_urls) = extract_array(content, array_name) {
            for url in download_urls {
                if let Some(p) = make_proxy(&url, ProxyType::Download) {
                    proxies.push(p);
                }
            }
            break; // 匹配到 download_url_us 就不再尝试 download_url
        }
    }

    // 解析 clone_url 数组
    if let Some(clone_urls) = extract_array(content, "clone_url") {
        for url in clone_urls {
            if let Some(p) = make_proxy(&url, ProxyType::Clone) {
                proxies.push(p);
            }
        }
    }

    // 解析 raw_url 数组
    if let Some(raw_urls) = extract_array(content, "raw_url") {
        for url in raw_urls {
            if let Some(p) = make_proxy(&url, ProxyType::Raw) {
                proxies.push(p);
            }
        }
    }

    // 解析 clone_ssh_url 数组（并尝试兼容旧名称 ssh_url）
    for array_name in &["clone_ssh_url", "ssh_url"] {
        if let Some(ssh_urls) = extract_array(content, array_name) {
            for url in ssh_urls {
                if let Some(p) = make_proxy(&url, ProxyType::Ssh) {
                    proxies.push(p);
                }
            }
            break; // 匹配到 clone_ssh_url 就不再尝试 ssh_url
        }
    }

    Ok(proxies)
}
