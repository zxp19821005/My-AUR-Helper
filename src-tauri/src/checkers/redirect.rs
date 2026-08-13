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
 * 支持版本格式：/v1.2.3、-v1.2.3 等常见版本 URL 格式。
 * URL 解析 / JS 重定向 / 脚本扫描等纯函数见同目录 redirect_parse.rs。
 */
use crate::errors::AppResult;
use async_trait::async_trait;
use log::{debug, info, warn};
use reqwest::Client;
use std::collections::HashSet;

use super::redirect_parse::*;
use super::trait_def::{CheckOptions, CheckResult, VersionChecker};
use super::utils::{extract_version_from_url, extract_version_with_regex};

/// 最大重定向次数，防止无限循环
const MAX_REDIRECTS: usize = 5;

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
