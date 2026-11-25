/**
 * http.rs - HTTP 页面解析版本检查器（带重试）
 *
 * 通过请求上游 HTML 页面，使用版本提取正则从页面文本中提取版本号，
 * 适用于上游以网页形式发布版本信息的场景。
 * 网络错误会自动重试（指数退避）。
 */
use crate::checkers::helpers::{finish_check, MissingVersionLog};
use crate::errors::AppResult;
use crate::network::retry::{retry_with_backoff, DEFAULT_MAX_RETRIES};
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use super::trait_def::{CheckOptions, CheckResult, VersionChecker};
use super::utils::{
    extract_version_from_html, extract_version_from_url, extract_version_with_regex,
    normalize_extracted_version,
};

pub struct HttpChecker;

#[async_trait]
impl VersionChecker for HttpChecker {
    fn name(&self) -> &'static str {
        "http"
    }

    async fn check(
        &self,
        client: &Client,
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

        // 带重试的请求逻辑
        let result = retry_with_backoff(DEFAULT_MAX_RETRIES, || {
            let client = client.clone();
            let url = upstream_url.to_string();
            async move {
                let resp = client
                    .get(&url)
                    .header("User-Agent", "my-aur-helper/0.1")
                    .send()
                    .await?;
                if !resp.status().is_success() {
                    // HTTP 非2xx 不可重试，直接返回错误
                    return Err(crate::errors::AppError::NetworkError(format!(
                        "HTTP {} for {}",
                        resp.status(),
                        url
                    )));
                }
                let body = resp.text().await?;
                Ok(body)
            }
        })
        .await;

        let body: String = match result {
            Ok(v) => v,
            Err(_) => {
                // 重试耗尽，返回空结果
                return Ok(CheckResult::default());
            }
        };
        // 注意：resp 已消费，通过 text() 获取 body
        let final_url = upstream_url.to_string();

        let raw = if let Some(regex) = version_extract_regex {
            match extract_version_with_regex(&body, regex) {
                Some(ver) => Some(ver),
                None => extract_version_from_html(&body),
            }
        } else {
            extract_version_from_html(&body)
        };

        // 兜底：版本可能写在重定向/文件下载 URL 里
        let raw = raw.or_else(|| extract_version_from_url(&final_url));

        // 归一化：剥离文件扩展名/架构后缀/应用名前缀，保证无捕获组的正则
        // （如 `helio-\d\.\d+-x64.deb`）也能得到干净版本号 `3.18`，而非整段匹配
        // `helio-3.18-x64.deb` 泄漏进上游版本造成误报。
        let version = raw.map(|v| normalize_extracted_version(&v));

        Ok(finish_check(
            pkgname,
            version,
            None,
            vec![],
            MissingVersionLog::Debug,
        ))
    }
}
