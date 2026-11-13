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
use std::sync::OnceLock;

use super::redirect_parse::*;
use super::trait_def::{CheckOptions, CheckResult, VersionChecker};
use super::utils::{extract_version_from_url, extract_version_with_regex};

/// 最大重定向次数，防止无限循环
const MAX_REDIRECTS: usize = 5;

/// 重定向检查专用客户端（不自动跟随重定向，带浏览器 UA），进程级复用以避免重复构建连接池
static REDIRECT_CLIENT: OnceLock<Client> = OnceLock::new();

/// 获取重定向检查专用客户端单例
///
/// 注意：代理不在此处全局配置。RedirectChecker 检查的是任意 URL（包括
/// 非 GitHub 域名），不应强制走代理。如需代理，仅在具体请求中按需注入。
fn redirect_client() -> &'static Client {
    REDIRECT_CLIENT.get_or_init(|| {
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .expect("初始化重定向检查客户端失败")
    })
}

/// 不校验证书的兜底客户端单例。
///
/// 仅当上游 TLS 证书过期/不可信导致安全请求失败时启用（如 wiz.cn 的
/// `*.wiz.cn` 证书于 2026-11-05 过期，浏览器可手动跳过但 reqwest 默认拒绝）。
/// 与 `redirect_client` 共享「不自动跟随重定向 + 浏览器 UA」策略，仅关闭 TLS 校验。
/// 该客户端只用于拉取公开版本信息（无鉴权），且每次启用都会打 WARN 日志以便审计。
static REDIRECT_CLIENT_INSECURE: OnceLock<Client> = OnceLock::new();

fn insecure_redirect_client() -> &'static Client {
    REDIRECT_CLIENT_INSECURE.get_or_init(|| {
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(true)
            .build()
            .expect("初始化重定向检查兜底客户端失败")
    })
}

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

        // 复用进程级重定向检查客户端（不自动跟随重定向，带浏览器 UA）
        // 注意：绝不使用配置中的 GitHub 镜像代理（ProxyType::Download）作为正向代理去请求任意主机。
        // 若系统环境变量 http_proxy/https_proxy 已设置，reqwest 会自动采用真实系统代理。
        let client = redirect_client();

        let mut current_url = upstream_url.to_string();
        let mut version = None;
        let mut visited: HashSet<String> = HashSet::new();

        'redirects: for _ in 0..MAX_REDIRECTS {
            if !visited.insert(current_url.clone()) {
                debug!("[HTTP 重定向] 检测到重定向环，停止跟踪");
                break;
            }

            debug!("[HTTP 重定向] 第 {} 次请求: {}", visited.len(), current_url);
            // 添加 Referer 头：部分服务器（如 nfschina）需要 Referer 才能返回 200，
            // 否则返回 403。使用当前 URL 作为 Referer，模拟浏览器跳转行为。
            // 安全请求失败时，若错误特征指向 TLS 证书问题（过期/不可信），
            // 退化为不校验证书的兜底请求一次（仅拉取公开版本信息，并打 WARN 审计）。
            let resp = match client
                .get(&current_url)
                .header("Referer", &current_url)
                .send()
                .await
            {
                Ok(r) => r,
                // 安全请求失败：`send()` 仅在「传输层」出错时返回 Err（TLS 握手失败、
                // 证书过期、DNS 解析失败、超时、连接被拒等），HTTP 4xx/5xx 属于 Ok 响应、
                // 由后续状态码分支处理。这里对传输层失败降级为「不校验证书」兜底请求一次，
                // 仅用于拉取公开版本信息，并打 WARN 日志以便审计。
                Err(e) => {
                    if e.is_connect() {
                        warn!(
                            "[HTTP 重定向] 安全请求失败（{}），启用不校验证书的兜底请求: {}",
                            e, current_url
                        );
                        insecure_redirect_client()
                            .get(&current_url)
                            .header("Referer", &current_url)
                            .send()
                            .await?
                    } else {
                        return Err(e.into());
                    }
                }
            };

            // 调试：打印响应状态码和所有 headers
            debug!("[HTTP 重定向] 响应状态码: {}", resp.status());
            debug!("[HTTP 重定向] 响应 Headers: {:?}", resp.headers());

            // 在消费响应体之前先克隆重定向相关响应头（这些无需下载 body 即可读取）
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

            // 1) HTTP 3xx Location 重定向（纯响应头，无需下载 body）
            if let Some(loc) = location_hdr {
                debug!("[HTTP 重定向] Location: {}", loc);
                current_url = resolve_url(&current_url, &loc);
                if version.is_none() {
                    version = self.extract_version(&current_url, version_extract_regex);
                }
                continue;
            }

            // 2) 头级别版本提取（Content-Disposition 文件名 / 当前 URL），命中即结束。
            //    避免下载完整响应体：二进制落点（AppImage/deb…可达上百 MB）版本号就在文件名/CD 中。
            let cd_version = cd_hdr
                .as_ref()
                .and_then(|d| self.extract_version(d, version_extract_regex));
            let url_version = self.extract_version(&current_url, version_extract_regex);
            let from_cd = cd_version.is_some();
            if version.is_none() {
                version = cd_version.or(url_version);
            }

            // 是否还需下载 body：完全没提取到→必须（meta-refresh/JS/SPA）；仅 URL 命中且像 HTML→下载；CD 或二进制落点→跳过。
            let need_body = version.is_none() || (!from_cd && !response_is_binary(&resp));

            if !need_body {
                debug!("[HTTP 重定向] 已从 URL/Content-Disposition 提取可信版本，跳过 body 下载");
                break 'redirects;
            }

            // 仅当头级别未拿到可信版本时才下载 body（典型为 HTML 页面）
            debug!("[HTTP 重定向] 头级别未提取到可信版本，下载 body 解析内联重定向/脚本");
            let body = resp.text().await?;

            // 3) HTML <meta http-equiv="refresh"> 重定向
            if let Some(url) = extract_meta_refresh(&body) {
                debug!("[HTTP 重定向] 发现 meta-refresh 重定向: {}", url);
                current_url = resolve_url(&current_url, &url);
                if version.is_none() {
                    version = self.extract_version(&current_url, version_extract_regex);
                }
                continue;
            }

            // 4) 内联 JS 重定向（字面 URL）
            if let Some(url) = extract_js_redirect(&body) {
                debug!("[HTTP 重定向] 发现 JS 重定向: {}", url);
                current_url = resolve_url(&current_url, &url);
                if version.is_none() {
                    version = self.extract_version(&current_url, version_extract_regex);
                }
                continue;
            }

            // 5) 兜底：SPA 把下载地址拼在 JS 打包产物里（无 Location / 无 meta-refresh），
            //    抓取 <script src> 并对 JS 文本套用版本正则（如 flomo）
            if version.is_none() {
                if let Some(v) =
                    extract_version_from_scripts(client, &current_url, &body, version_extract_regex)
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

/// 判断响应是否为二进制下载落点（版本号已含于 URL/文件名，无需下载 body 解析）。
/// 命中时头级别的 CD/URL 提取结果即可信，可跳过响应体下载。
fn response_is_binary(resp: &reqwest::Response) -> bool {
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    // 二进制/归档类下载落点的 Content-Type 白名单：命中时版本号已含于 URL/文件名，无需下载 body
    const BIN_CT: &str = "\
application/octet-stream application/x-debian-package application/x-redhat-package-manager \
application/vnd.appimage application/zip application/x-zip-compressed application/gzip \
application/x-gzip application/x-xz application/x-tar application/x-7z-compressed \
application/x-rar-compressed application/x-msdownload application/x-apple-diskimage \
application/vnd.microsoft.portable-executable application/x-executable application/x-rpm \
application/pdf image/ video/ audio/";
    BIN_CT.split_whitespace().any(|p| ct.starts_with(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 确保安全客户端与「不校验证书」兜底客户端均能成功构建，
    /// 防止 reqwest TLS API（danger_accept_invalid_certs 等）漂移导致编译/运行时失败。
    #[test]
    fn test_redirect_clients_build() {
        // 进程级单例，重复调用应返回同一实例且不 panic
        let a = redirect_client();
        let b = redirect_client();
        assert!(std::ptr::eq(a, b));

        let c = insecure_redirect_client();
        let d = insecure_redirect_client();
        assert!(std::ptr::eq(c, d));
    }
}
