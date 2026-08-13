/**
 * release_history.rs - GitHub Releases 历史遍历扫描
 *
 * 功能：分页遍历仓库的全部 releases，提取并比较版本号。
 * 主要用于：测试版本（prerelease）检查、资产过滤、以及 latest release
 * 无匹配二进制文件时回退查找历史版本。
 *
 * 设计要点：
 * - releases 列表按发布时间倒序返回，首个通过校验的 release 即「最新且含
 *   匹配二进制」的 release，命中后立即结束扫描（避免大响应超时）。
 * - 每页数量限制为 30（release 多的仓库单页 JSON 可达数 MB，慢速/代理网络
 *   下极易在读取响应体时超时，即 "error decoding response body"）。
 */
use log::{debug, warn};
use reqwest::Client;

use crate::checkers::github::binary_check::{extract_version_from_assets, has_linux_binary};
use crate::checkers::github::release::build_github_request;
use crate::checkers::utils::clean_version;
use crate::errors::AppResult;
use crate::versions;

/// 遍历 releases，提取并比较版本号（支持分页）
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `token`: GitHub API Token（可选）
/// - `version_extract_regex`: 版本提取正则表达式（可选）
/// - `check_test_versions`: 是否包含测试版本（prerelease）
/// - `check_binary_files`: 是否检查二进制文件
/// - `pkgname`: 软件包名称（用于日志）
///
/// # 返回
/// - `Ok(Some(version))`: 找到的最新版本
/// - `Ok(None)`: 未找到任何有效 release
/// - `Err(e)`: 请求失败
pub async fn check_github_releases(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    version_extract_regex: Option<&str>,
    check_test_versions: bool,
    check_binary_files: bool,
    pkgname: &str,
) -> AppResult<Option<String>> {
    let mut best_version: Option<String> = None;
    let mut page = 1;
    // 降低每页数量以减小单次响应体积：releases 列表接口在 release 较多的仓库下
    // 单页 JSON 可达数 MB，慢速/代理网络下极易在读取响应体时超时
    // （"error decoding response body"）。30 条/页足够覆盖绝大多数"最新含二进制"场景。
    let per_page = 30;
    let max_pages = 5;

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
    'scan: loop {
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

        let resp = build_github_request(client, &api_url, token).send().await?;

        if resp.status().as_u16() == 403 {
            warn!("[二进制检查] {}: 触发 GitHub API 限流，停止搜索", pkgname);
            break;
        }

        if !resp.status().is_success() {
            return Ok(None);
        }

        let releases: Vec<serde_json::Value> = resp.json().await?;

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
                    break 'scan;
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
