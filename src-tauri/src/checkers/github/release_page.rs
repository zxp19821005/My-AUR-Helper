//! release_page.rs - GitHub releases 单页请求与内部重试
use crate::checkers::github::release::build_github_request;
use crate::errors::{AppError, AppResult};
use log::{debug, warn};
use reqwest::Client;

/// 单页 releases 请求的返回结果
pub enum PageOutcome {
    /// 成功获取 releases 列表（可能为空数组）
    Releases(Vec<serde_json::Value>),
    /// 触发 GitHub API 限流（HTTP 403），应停止整段扫描
    RateLimited,
    /// 非成功状态码（非 403），视为无有效结果，直接返回 None
    Abort,
}

/// 单页 releases 请求（内部最多重试 5 次，失败不重置分页进度）
///
/// 用于 [`check_github_releases`] 的分页循环：单页请求或解析失败时做内部重试，
/// 5 次全败才向上返回错误，避免外层批量重试将 `page` 重置为 1 导致无限循环。
pub async fn fetch_release_page(
    client: &Client,
    api_url: &str,
    token: Option<&str>,
    pkgname: &str,
    page: u32,
) -> AppResult<PageOutcome> {
    let mut page_resp: Option<Vec<serde_json::Value>> = None;
    let mut page_error: Option<AppError> = None;
    // 记录是否出现过超时类错误（body 流阶段超时，本质是可重试的网络故障），
    // 用于最终错误分类：超时→NetworkTimeout(可重试)，否则→ParseError(不可重试)。
    let mut saw_timeout = false;
    for retry in 0..5 {
        let resp = match build_github_request(client, api_url, token).send().await {
            Ok(r) => r,
            Err(e) => {
                let app_err: AppError = e.into();
                if matches!(app_err, AppError::NetworkTimeout(_)) {
                    saw_timeout = true;
                }
                if retry < 4 {
                    debug!(
                        "[二进制检查] {}: 第 {} 页请求失败 (尝试 {}/5): {}",
                        pkgname,
                        page,
                        retry + 1,
                        app_err
                    );
                    page_error = Some(app_err);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
                // 5 次全败：超时类错误包装为 NetworkTimeout（可重试），
                // 让外层 check_with_retry 在代理抖动等瞬时故障时重试；
                // 真解析错误保持 ParseError（不可重试），避免无限循环。
                return Err(if saw_timeout {
                    AppError::NetworkTimeout(format!(
                        "[二进制检查] {}: 第 {} 页请求 5 次均失败(超时): {}",
                        pkgname, page, app_err
                    ))
                } else {
                    AppError::ParseError(format!(
                        "[二进制检查] {}: 第 {} 页请求 5 次均失败: {}",
                        pkgname, page, app_err
                    ))
                });
            }
        };

        if resp.status().as_u16() == 403 {
            warn!("[二进制检查] {}: 触发 GitHub API 限流，停止搜索", pkgname);
            return Ok(PageOutcome::RateLimited);
        }

        if !resp.status().is_success() {
            return Ok(PageOutcome::Abort);
        }

        match resp.json::<Vec<serde_json::Value>>().await {
            Ok(releases) => {
                page_resp = Some(releases);
                break;
            }
            Err(e) => {
                let app_err: AppError = e.into();
                if matches!(app_err, AppError::NetworkTimeout(_)) {
                    saw_timeout = true;
                }
                if retry < 4 {
                    debug!(
                        "[二进制检查] {}: 第 {} 页响应解析失败 (尝试 {}/5): {}",
                        pkgname,
                        page,
                        retry + 1,
                        app_err
                    );
                    page_error = Some(app_err);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
                // 5 次全败：超时类错误包装为 NetworkTimeout（可重试），
                // 让外层 check_with_retry 在代理抖动等瞬时故障时重试；
                // 真解析错误保持 ParseError（不可重试），避免无限循环。
                return Err(if saw_timeout {
                    AppError::NetworkTimeout(format!(
                        "[二进制检查] {}: 第 {} 页响应解析 5 次均失败(超时): {}",
                        pkgname, page, app_err
                    ))
                } else {
                    AppError::ParseError(format!(
                        "[二进制检查] {}: 第 {} 页响应解析 5 次均失败: {}",
                        pkgname, page, app_err
                    ))
                });
            }
        }
    }

    match page_resp {
        Some(r) => Ok(PageOutcome::Releases(r)),
        None => Err(page_error.unwrap_or_else(|| {
            AppError::ParseError(format!(
                "[二进制检查] {}: 第 {} 页请求最终失败",
                pkgname, page
            ))
        })),
    }
}
