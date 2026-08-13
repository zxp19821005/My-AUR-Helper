/**
 * redirect.rs - HTTP 重定向版本检查器
 *
 * 功能：通过跟踪 HTTP 重定向获取最终 URL，并从中提取版本号。
 *
 * 工作流程：
 * 1. 发送 GET 请求到上游 URL（禁用自动重定向）
 * 2. 按以下优先级跟踪"重定向"直到拿到最终地址：
 *    a. HTTP 3xx 的 Location 头
 *    b. HTML <meta http-equiv="refresh" content="N; url=...">
 *    c. 内联 JS 重定向（window.location* = "url" / window.location.replace("url")）
 * 3. 每次跳转后用版本正则/URL 提取版本
 * 4. 兜底：若最终响应是 SPA（下载地址由 JS 在打包产物里拼接，无 Location/无 meta-refresh），
 *    则抓取页面引用的 <script src> 并对 JS 文本套用版本正则（如 flomo 把版本写死在 bundle 里）
 *
 * 支持的版本格式：
 * - /v1.2.3 或 /1.2.3
 * - -v1.2.3 或 -1.2.3
 * - 其他常见的版本 URL 格式
 */
use crate::errors::AppResult;
use async_trait::async_trait;
use log::{debug, info, warn};
use regex::Regex;
use reqwest::Client;
use std::collections::HashSet;
use url::Url;

use super::trait_def::{CheckOptions, CheckResult, VersionChecker};
use super::utils::{extract_version_from_url, extract_version_with_regex};

/// 最大重定向次数，防止无限循环
const MAX_REDIRECTS: usize = 5;
/// 兜底扫描脚本时的数量上限：覆盖绝大多数 SPA 的全部 chunk
/// （如 flomo 需要抓到第 8 个 index chunk 才能拿到 VUE_APP_VERSION）
const MAX_SCRIPT_FETCH: usize = 12;
/// 兜底扫描脚本的总字节预算，防止失控（约 16MB）
const MAX_SCRIPT_BYTES: usize = 16 * 1024 * 1024;

pub struct RedirectChecker;

#[async_trait]
impl VersionChecker for RedirectChecker {
    fn name(&self) -> &'static str {
        "redirect"
    }

    async fn check(
        &self,
        _client: &Client,
        upstream_url: &str,
        pkgname: &str,
        version_extract_regex: Option<&str>,
        _options: &CheckOptions,
    ) -> AppResult<CheckResult> {
        info!(
            "[版本检查] 开始检查软件包: {} (检查器: {})",
            pkgname,
            self.name()
        );
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);

        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(CheckResult::default());
        }

        // 创建不自动跟随重定向的客户端（直连）
        // 注意：绝不使用配置中的 GitHub 镜像代理（ProxyType::Download）作为正向代理去请求任意主机。
        // 若系统环境变量 http_proxy/https_proxy 已设置，reqwest 会自动采用真实系统代理。
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()?;

        let mut current_url = upstream_url.to_string();
        let mut version = None;
        let mut visited: HashSet<String> = HashSet::new();

        'redirects: for _ in 0..MAX_REDIRECTS {
            if !visited.insert(current_url.clone()) {
                debug!("[HTTP 重定向] 检测到重定向环，停止跟踪");
                break;
            }

            debug!("[HTTP 重定向] 第 {} 次请求: {}", visited.len(), current_url);
            let resp = client.get(&current_url).send().await?;

            // 调试：打印响应状态码和所有 headers
            debug!("[HTTP 重定向] 响应状态码: {}", resp.status());
            debug!("[HTTP 重定向] 响应 Headers: {:?}", resp.headers());

            // 在消费响应体之前先克隆重定向相关响应头
            let location_hdr = resp
                .headers()
                .get("location")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());
            let cd_hdr = resp
                .headers()
                .get("content-disposition")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());
            let body = resp.text().await?;

            // 1) HTTP 3xx Location 重定向
            if let Some(loc) = location_hdr {
                debug!("[HTTP 重定向] Location: {}", loc);
                current_url = resolve_url(&current_url, &loc);
                if version.is_none() {
                    version = self.extract_version(&current_url, version_extract_regex);
                }
                continue;
            }

            // 2) HTML <meta http-equiv="refresh"> 重定向
            if let Some(url) = extract_meta_refresh(&body) {
                debug!("[HTTP 重定向] 发现 meta-refresh 重定向: {}", url);
                current_url = resolve_url(&current_url, &url);
                if version.is_none() {
                    version = self.extract_version(&current_url, version_extract_regex);
                }
                continue;
            }

            // 3) 内联 JS 重定向（字面 URL）
            if let Some(url) = extract_js_redirect(&body) {
                debug!("[HTTP 重定向] 发现 JS 重定向: {}", url);
                current_url = resolve_url(&current_url, &url);
                if version.is_none() {
                    version = self.extract_version(&current_url, version_extract_regex);
                }
                continue;
            }

            // 4) 无更多重定向：尝试从 Content-Disposition / 最终 URL 提取版本
            if version.is_none() {
                if let Some(disp) = &cd_hdr {
                    debug!("[HTTP 重定向] Content-Disposition: {}", disp);
                    version = self.extract_version(disp, version_extract_regex);
                }
            }
            if version.is_none() {
                version = self.extract_version(&current_url, version_extract_regex);
            }

            // 5) 兜底：SPA 把下载地址拼在 JS 打包产物里（无 Location / 无 meta-refresh），
            //    抓取 <script src> 并对 JS 文本套用版本正则（如 flomo）
            if version.is_none() {
                if let Some(v) = extract_version_from_scripts(
                    &client,
                    &current_url,
                    &body,
                    version_extract_regex,
                )
                .await
                {
                    version = Some(v);
                }
            }

            break 'redirects;
        }

        if let Some(v) = &version {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v);
        } else {
            warn!("[版本检查] 未能从重定向 URL 提取版本: {}", upstream_url);
        }

        Ok(CheckResult {
            version,
            ..Default::default()
        })
    }
}

impl RedirectChecker {
    /// 从文本提取版本号：优先用用户配置的正则，失败回退到 URL 提取
    fn extract_version(&self, text: &str, version_extract_regex: Option<&str>) -> Option<String> {
        if let Some(regex) = version_extract_regex {
            match extract_version_with_regex(text, regex) {
                Some(ver) => Some(ver),
                None => extract_version_from_url(text),
            }
        } else {
            extract_version_from_url(text)
        }
    }
}

/// 将相对/绝对重定向目标解析为完整 URL（处理 // 协议相对与相对路径）
fn resolve_url(base: &str, target: &str) -> String {
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
fn extract_meta_refresh(html: &str) -> Option<String> {
    let re = Regex::new(
        r#"(?is)<meta[^>]*http-equiv[^>]*refresh[^>]*content\s*=\s*["']\s*\d+\s*;\s*url\s*=\s*['"]?\s*([^'">\s]+)"#,
    )
    .ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// 解析内联 JS 重定向（仅匹配字面 URL，避免误抓 window.location.replace(变量)）
fn extract_js_redirect(html: &str) -> Option<String> {
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
async fn extract_version_from_scripts(
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
