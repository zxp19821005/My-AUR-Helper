//! release_history.rs - GitHub Releases 历史遍历扫描
//!
//! 功能：分页遍历仓库的全部 releases，提取并比较版本号。
//! 主要用于：测试版本（prerelease）检查、资产过滤、以及 latest release
//! 无匹配二进制文件时回退查找历史版本。
//!
//! 设计要点：
//! - releases 列表按发布时间倒序返回，首个通过校验的 release 即「最新且含
//!   匹配二进制」的 release，命中后立即结束扫描（避免大响应超时）。
//! - 每页数量限制为 30（release 多的仓库单页 JSON 可达数 MB，慢速/代理网络
//!   下极易在读取响应体时超时，即 "error decoding response body"）。
//! - 单页请求超时/decode 错误会触发内部重试，**不重置分页进度**，
//!   避免外层批量重试将 page 重置为 1 导致的无限循环。
use log::{debug, warn};
use reqwest::Client;

use crate::checkers::github::binary_check::{extract_version_from_assets, has_linux_binary};
use crate::checkers::github::release::build_github_request;
use crate::checkers::utils::clean_version;
use crate::errors::{AppError, AppResult};
use crate::versions;

/// GitHub Releases 历史扫描参数（打包以避免函数参数过多）
///
/// 持有调用方数据的引用，无所有权转移。
#[derive(Clone, Copy)]
pub struct ReleaseScanParams<'a> {
    /// GitHub 仓库所有者
    pub owner: &'a str,
    /// GitHub 仓库名称
    pub repo: &'a str,
    /// GitHub API Token（可选）
    pub token: Option<&'a str>,
    /// 版本提取正则表达式（可选）
    pub version_extract_regex: Option<&'a str>,
    /// 是否包含测试版本（prerelease）
    pub check_test_versions: bool,
    /// 是否检查二进制文件
    pub check_binary_files: bool,
    /// 软件包名称（用于日志）
    pub pkgname: &'a str,
    /// 起始页码（用于重试时断点续查，默认从第 1 页开始）
    pub start_page: u32,
}

impl<'a> Default for ReleaseScanParams<'a> {
    fn default() -> Self {
        Self {
            owner: "",
            repo: "",
            token: None,
            version_extract_regex: None,
            check_test_versions: false,
            check_binary_files: false,
            pkgname: "",
            start_page: 0,
        }
    }
}

/// 遍历 releases，提取并比较版本号（支持分页）
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `params`: 扫描参数（仓库、token、正则、检查标志、包名）
///
/// # 返回
/// - `Ok(Some(version))`: 找到的最新版本
/// - `Ok(None)`: 未找到任何有效 release
/// - `Err(e)`: 请求失败
pub async fn check_github_releases(
    client: &Client,
    params: &ReleaseScanParams<'_>,
) -> AppResult<Option<String>> {
    let ReleaseScanParams {
        owner,
        repo,
        token,
        version_extract_regex,
        check_test_versions,
        check_binary_files,
        pkgname,
        ..
    } = *params;
    let mut best_version: Option<String> = None;
    let mut page = params.start_page.max(1); // 支持从指定页码开始（重试断点续查）
                                             // GitHub REST API 限制 per_page 最大为 100。
                                             // 之前设为 30 是为了减小单次响应体（避免慢速/代理网络下 "error decoding response body"），
                                             // 但现已有单页内部重试机制（最多 3 次），可以安全使用最大值 100，
                                             // 大幅减少翻页次数（electron/electron 约 3000+ release，30/页需 100+ 页，100/页只需 30+ 页）。
    let per_page = 100;
    // D5 修复：max_pages 从 5（150 条）改为 30（900 条），后改为 167（对应 30 条/页时约 5000 条）。
    // 2026-09-02：per_page 改为 100 后，167 页实际覆盖 16700 条，远超 electron/electron 的 ~3000 条，
    // 作为上限保护完全足够；实际命中即早停，仅正则范围极窄的包才可能触达上限。
    let max_pages = 167;

    let tag_filter = if let Some(regex) = version_extract_regex {
        // 如果正则包含明显的文件扩展名，说明是用于匹配 asset 文件名的，
        // 不应用于过滤 release tags
        let has_file_extension = regex.contains(".rpm")
            || regex.contains(".deb")
            || regex.contains(".zip")
            || regex.contains(".tar")
            || regex.contains(".pkg")
            || regex.contains(".dmg")
            || regex.contains(".exe")
            || regex.contains(".AppImage");

        if has_file_extension {
            debug!(
                "[二进制检查] {}: 正则 '{}' 包含文件扩展名，跳过 tag 过滤",
                pkgname, regex
            );
            None
        } else {
            regex::Regex::new(regex).ok()
        }
    } else {
        None
    };

    // releases 列表接口按发布时间倒序返回，因此首个通过校验的 release
    // 即为"最新且含匹配二进制"的 release。命中后立即结束整段扫描，
    // 避免为罕见的"最新 release 无二进制、需翻多页历史"场景付出无谓的大响应请求。
    //
    // 内层 'page_loop'：对单页请求做内部重试（最多 3 次），
    // 失败时递增 page 继续，**不重置为 1**，避免外层批量重试导致无限循环。
    'page_loop: loop {
        if page > max_pages {
            debug!(
                "[二进制检查] {}: 已达到最大页数限制 ({} 页，{} 个 releases)，停止搜索",
                pkgname,
                max_pages,
                max_pages * per_page
            );
            break;
        }

        let api_url = format!(
            "https://api.github.com/repos/{}/{}/releases?per_page={}&page={}",
            owner, repo, per_page, page
        );

        // 单页请求内部重试：最多 5 次，防止偶发 decode 超时导致整段扫描重来。
        // 内部重试耗尽后返回 ParseError（非 retryable），阻止外层 check_with_retry
        // 将整个函数从 page=1 重新调用，避免无限循环。
        let mut page_resp: Option<Vec<serde_json::Value>> = None;
        let mut page_error: Option<AppError> = None;
        for retry in 0..5 {
            let resp = match build_github_request(client, &api_url, token).send().await {
                Ok(r) => r,
                Err(e) => {
                    let app_err: AppError = e.into();
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
                    // 5 次全败：包装为 ParseError（非 retryable），阻止外层重试
                    return Err(AppError::ParseError(format!(
                        "[二进制检查] {}: 第 {} 页请求 5 次均失败: {}",
                        pkgname, page, app_err
                    )));
                }
            };

            if resp.status().as_u16() == 403 {
                warn!("[二进制检查] {}: 触发 GitHub API 限流，停止搜索", pkgname);
                break 'page_loop;
            }

            if !resp.status().is_success() {
                return Ok(None);
            }

            match resp.json::<Vec<serde_json::Value>>().await {
                Ok(releases) => {
                    page_resp = Some(releases);
                    break;
                }
                Err(e) => {
                    let app_err: AppError = e.into();
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
                    // 5 次全败：包装为 ParseError（非 retryable），阻止外层重试
                    return Err(AppError::ParseError(format!(
                        "[二进制检查] {}: 第 {} 页响应解析 5 次均失败: {}",
                        pkgname, page, app_err
                    )));
                }
            }
        }

        let releases = match page_resp {
            Some(r) => r,
            None => {
                // 理论上不会到达这里（上方已 return），兜底返回
                return Err(page_error.unwrap_or_else(|| {
                    AppError::ParseError(format!(
                        "[二进制检查] {}: 第 {} 页请求最终失败",
                        pkgname, page
                    ))
                }));
            }
        };

        if releases.is_empty() {
            debug!(
                "[二进制检查] {}: 第 {} 页无更多 releases，停止搜索",
                pkgname, page
            );
            break;
        }

        debug!(
            "[二进制检查] {}: 正在检查第 {} 页 ({} 个 releases)",
            pkgname,
            page,
            releases.len()
        );

        for release in &releases {
            if let Some(tag) = release["tag_name"].as_str() {
                if !check_test_versions && release["prerelease"].as_bool().unwrap_or(false) {
                    debug!(
                        "[二进制检查] {}: Release {} 是 prerelease，跳过",
                        pkgname, tag
                    );
                    continue;
                }

                if let Some(ref re) = tag_filter {
                    let release_name = release["name"].as_str().unwrap_or(tag);
                    if !re.is_match(tag) && !re.is_match(release_name) {
                        debug!(
                            "[二进制检查] {}: Release {} ({}) 不匹配正则 {}，跳过",
                            pkgname,
                            tag,
                            release_name,
                            version_extract_regex.unwrap_or("")
                        );
                        continue;
                    }
                }

                if check_binary_files {
                    if let Some(assets) = release["assets"].as_array() {
                        if !has_linux_binary(assets, version_extract_regex) {
                            debug!(
                                "[二进制检查] {}: Release {} 无匹配的资产文件，跳过",
                                pkgname, tag
                            );
                            continue;
                        }
                    }
                    // 二进制检查命中：当前 release 即"最新含匹配二进制"，
                    // 采用其版本并立即结束整段扫描（倒序首个 = 最新）
                    let version = if let Some(assets) = release["assets"].as_array() {
                        extract_version_from_assets(assets, version_extract_regex.unwrap())
                            .unwrap_or_else(|| clean_version(tag))
                    } else {
                        clean_version(tag)
                    };
                    best_version = Some(version);
                    break 'page_loop;
                }

                // 非二进制：取所有匹配 tag 中的最大版本
                let version = clean_version(tag);
                best_version = match best_version.take() {
                    Some(current)
                        if versions::compare_versions(&current, &version)
                            == versions::VersionComparison::LessThan =>
                    {
                        Some(version)
                    }
                    Some(current) => Some(current),
                    None => Some(version),
                };
            }
        }

        page += 1;
    }

    Ok(best_version)
}
