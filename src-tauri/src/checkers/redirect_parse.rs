/**
 * redirect_parse.rs - HTTP 重定向版本检查器的纯解析辅助函数
 *
 * 功能：提供重定向链中所需的 URL 解析与版本提取工具：
 * - resolve_url: 相对/绝对/协议相对重定向目标解析为完整 URL
 * - extract_meta_refresh: 解析 <meta http-equiv="refresh">
 * - extract_js_redirect: 解析内联 JS 重定向（仅字面 URL）
 * - extract_version_from_scripts: 抓取页面引用的 JS 打包产物并套用版本正则
 *
 * 这些函数无副作用、不依赖检查器状态，集中于此便于复用与测试。
 */
use log::debug;
use regex::Regex;
use reqwest::Client;
use std::collections::HashSet;
use url::Url;

/// 兜底扫描脚本时的数量上限：覆盖绝大多数 SPA 的全部 chunk
/// （如 flomo 需要抓到第 8 个 index chunk 才能拿到 VUE_APP_VERSION）
const MAX_SCRIPT_FETCH: usize = 12;
/// 兜底扫描脚本的总字节预算，防止失控（约 16MB）
const MAX_SCRIPT_BYTES: usize = 16 * 1024 * 1024;

/// 将相对/绝对重定向目标解析为完整 URL（处理 // 协议相对与相对路径）
pub(crate) fn resolve_url(base: &str, target: &str) -> String {
    let target = target.trim();
    if target.starts_with("http://") || target.starts_with("https://") {
        return target.to_string();
    }
    if target.starts_with("//") {
        return format!("https:{}", target);
    }
    match Url::parse(base) {
        Ok(b) => b
            .join(target)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| target.to_string()),
        Err(_) => target.to_string(),
    }
}

/// 解析 <meta http-equiv="refresh" content="N; url=...">
pub(crate) fn extract_meta_refresh(html: &str) -> Option<String> {
    let re = Regex::new(
        r#"(?is)<meta[^>]*http-equiv[^>]*refresh[^>]*content\s*=\s*["']\s*\d+\s*;\s*url\s*=\s*['"]?\s*([^'">\s]+)"#,
    )
    .ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// 解析内联 JS 重定向（仅匹配字面 URL，避免误抓 window.location.replace(变量)）
pub(crate) fn extract_js_redirect(html: &str) -> Option<String> {
    let patterns = [
        r#"(?i)(?:window\.)?location(?:\.href|\.replace|\.assign)?\s*=\s*["']([^"']+)["']"#,
        r#"(?i)window\.location\.replace\(\s*["']([^"']+)["']\s*\)"#,
    ];
    for p in patterns {
        if let Ok(re) = Regex::new(p) {
            if let Some(cap) = re.captures(html) {
                if let Some(m) = cap.get(1) {
                    let url = m.as_str().trim();
                    if url.starts_with("http://") || url.starts_with("https://") {
                        return Some(url.to_string());
                    }
                }
            }
        }
    }
    None
}

/// 抓取页面引用的 JS 打包产物，对合并后的文本套用版本正则。
/// 用于 SPA 把下载地址拼在 JS 里、且无任何 HTTP/HTML 重定向的情况（如 flomo）。
/// 每抓完一个脚本就立即尝试匹配，命中即返回，避免无谓下载后续脚本。
pub(crate) async fn extract_version_from_scripts(
    client: &Client,
    page_url: &str,
    html: &str,
    version_extract_regex: Option<&str>,
) -> Option<String> {
    let regex = version_extract_regex?;
    let re = Regex::new(regex).ok()?;
    let script_re = Regex::new(r#"<script[^>]*\ssrc\s*=\s*["']([^"']+)["']"#).ok()?;

    let mut combined = String::new();
    let mut fetched: HashSet<String> = HashSet::new();
    let mut total_bytes: usize = 0;
    let mut count = 0;
    for cap in script_re.captures_iter(html) {
        if count >= MAX_SCRIPT_FETCH || total_bytes >= MAX_SCRIPT_BYTES {
            break;
        }
        let src = match cap.get(1) {
            Some(m) => m.as_str(),
            None => continue,
        };
        let abs = if src.starts_with("http://") || src.starts_with("https://") {
            src.to_string()
        } else {
            resolve_url(page_url, src)
        };
        if !fetched.insert(abs.clone()) {
            continue;
        }
        count += 1;
        debug!("[HTTP 重定向] 抓取脚本以提取版本: {}", abs);
        let resp = match client.get(&abs).send().await {
            Ok(r) => r,
            Err(e) => {
                debug!("[HTTP 重定向] 抓取脚本失败 {}: {}", abs, e);
                continue;
            }
        };
        let text = match resp.text().await {
            Ok(t) => t,
            Err(e) => {
                debug!("[HTTP 重定向] 读取脚本失败 {}: {}", abs, e);
                continue;
            }
        };
        combined.push_str(&text);
        combined.push('\n');
        total_bytes += text.len();
        // 每抓完一个就尝试匹配，命中即返回（省带宽）
        if let Some(m) = re.captures(&combined).and_then(|c| c.get(1)) {
            debug!("[HTTP 重定向] 在 JS 文本上命中版本正则: {}", regex);
            return Some(m.as_str().to_string());
        }
    }

    debug!(
        "[HTTP 重定向] 在全部脚本 JS 文本上套用版本正则未命中: {}",
        regex
    );
    None
}
