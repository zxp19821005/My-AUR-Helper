/**
 * browser.rs - 浏览器（JS 渲染）版本检查器
 *
 * 功能：
 * - 适用于上游页面由 JavaScript 动态渲染、静态抓取只能拿到 SPA 空壳的场景
 *   （如百度的 landingPage）
 * - 通过调用本机已安装的 Chromium / Chrome 的 `--headless --dump-dom` 执行 JS，
 *   拿到渲染后的 DOM，去除标签得到纯文本，再复用既有正则/HTML 版本提取逻辑
 *
 * 依赖：本机需安装 Chromium 或 Google Chrome，否则 check() 返回明确的错误提示
 */

use crate::errors::{AppError, AppResult};
use async_trait::async_trait;
use log::{debug, info, warn};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;

use super::trait_def::{CheckOptions, CheckResult, VersionChecker};
use super::utils::{extract_version_from_html, extract_version_with_regex};

/// 浏览器（JS 渲染）检查器
///
/// 适用于上游页面由 JavaScript 动态渲染、静态抓取只能拿到 SPA 空壳的场景
/// （如百度的 landingPage）。通过调用本机已安装的 Chromium / Chrome 的
/// `--headless --dump-dom` 执行 JS，拿到渲染后的 DOM，再去除标签得到纯文本，
/// 最后复用既有正则/HTML 版本提取逻辑。
pub struct BrowserChecker;

impl BrowserChecker {
    /// 在本机查找浏览器可执行文件：先查常见绝对路径，再在 PATH 中查找
    fn find_browser() -> Option<String> {
        const ABSOLUTE: &[&str] = &[
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chrome",
            "/snap/bin/chromium",
        ];
        for p in ABSOLUTE {
            if Path::new(p).exists() {
                return Some(p.to_string());
            }
        }
        let path_env = std::env::var("PATH").unwrap_or_default();
        const NAMES: &[&str] = &[
            "chromium",
            "chromium-browser",
            "google-chrome",
            "google-chrome-stable",
            "chrome",
            "brave",
            "microsoft-edge",
        ];
        for dir in path_env.split(':') {
            for name in NAMES {
                let full = Path::new(dir).join(name);
                if full.exists() {
                    return Some(full.to_string_lossy().to_string());
                }
            }
        }
        None
    }

    /// 将渲染后的 HTML 转为纯文本：去掉 script/style 块与标签
    ///
    /// 关键点：内联标签（如 `<span>`）直接去除、不插入空格，避免把
    /// `V<span>3.4.2</span>` 变成 `V 3.4.2` 导致用户正则 `V(\d+\.\d+\.\d+)`
    /// 匹配失败；仅在块级标签结束处插入换行，避免跨块文字粘连影响提取。
    fn html_to_text(html: &str) -> String {
        let re_script = regex::Regex::new(r"(?is)<script\b[^>]*>.*?</script>").unwrap();
        let re_style = regex::Regex::new(r"(?is)<style\b[^>]*>.*?</style>").unwrap();
        let re_br = regex::Regex::new(r"(?i)<br\s*/?>").unwrap();
        let re_block =
            regex::Regex::new(r"(?i)</(p|div|li|tr|th|td|h[1-6]|section|article|header|footer)>")
                .unwrap();
        let re_tag = regex::Regex::new(r"<[^>]*>").unwrap();

        let s = re_script.replace_all(html, " ");
        let s = re_style.replace_all(&s, " ");
        let s = re_br.replace_all(&s, "\n");
        let s = re_block.replace_all(&s, "\n");
        re_tag.replace_all(&s, "").to_string()
    }
}

#[async_trait]
impl VersionChecker for BrowserChecker {
    fn name(&self) -> &'static str {
        "browser"
    }

    async fn check(
        &self,
        _client: &reqwest::Client,
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

        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(CheckResult::default());
        }

        let browser = match Self::find_browser() {
            Some(b) => b,
            None => {
                return Err(AppError::VersionCheckError(
                    "未找到本机浏览器（chromium / chrome）。浏览器(JS渲染)检查器需要本机已安装 Chromium 或 Google Chrome，请先安装后再使用。".to_string(),
                ));
            }
        };
        debug!("[版本检查] 使用浏览器: {}", browser);

        let output = tokio::time::timeout(
            Duration::from_secs(60),
            Command::new(&browser)
                .args([
                    "--headless",
                    "--no-sandbox",
                    "--disable-gpu",
                    "--disable-dev-shm-usage",
                    "--dump-dom",
                    upstream_url,
                ])
                .output(),
        )
        .await
        .map_err(|_| AppError::VersionCheckError(format!("浏览器检查 {} 超时（60 秒）", pkgname)))?
        .map_err(|e| AppError::VersionCheckError(format!("启动浏览器失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(
                "[版本检查] 浏览器返回非零退出码: {} | stderr: {}",
                output.status, stderr
            );
            return Ok(CheckResult::default());
        }

        let dom = String::from_utf8_lossy(&output.stdout);
        let text = Self::html_to_text(&dom);
        debug!(
            "[版本检查] 渲染后文本长度: {} (前 200 字符: {})",
            text.len(),
            &text.chars().take(200).collect::<String>()
        );

        let version = match version_extract_regex {
            Some(regex) => extract_version_with_regex(&text, regex)
                .or_else(|| extract_version_from_html(&text)),
            None => extract_version_from_html(&text),
        };

        if let Some(v) = &version {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        Ok(CheckResult {
            version,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_version_not_split_by_tags() {
        // 模拟百度 landingPage 渲染后的 DOM：版本号被拆到多个 <span> 中，
        // 且页面带 <script> 干扰内容
        let html = r#"<div class="ver">最新版本 <span>V</span><span>3.4.2</span></div><script>var x="V9.9.9";</script>"#;
        let text = BrowserChecker::html_to_text(html);
        assert!(text.contains("V3.4.2"), "转换后文本: {}", text);

        // 用户配置的正则应能从纯文本中匹配到 3.4.2（不被 script 中的 V9.9.9 干扰，
        // 因为正则要求 V 后紧跟数字）
        let v = extract_version_with_regex(&text, r"V(\d+\.\d+\.\d+)");
        assert_eq!(v.as_deref(), Some("3.4.2"));
    }

    #[test]
    fn block_boundary_separated() {
        let html = r#"<div>版本:</div><div>1.2.3</div>"#;
        let text = BrowserChecker::html_to_text(html);
        assert!(text.contains("版本:"));
        assert!(text.contains("1.2.3"));
        // 块级分隔后，关键词提取仍能命中
        let v = extract_version_from_html(&text);
        assert_eq!(v.as_deref(), Some("1.2.3"));
    }
}
