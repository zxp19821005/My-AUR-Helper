/**
 * release_tags.rs - latest release 无匹配资产时的 tags 回退解析
 *
 * 功能：当 `releases/latest` 的附件里找不到目标平台包时，从 tags 列表定位
 * 候选版本，再逐个校验其 release 资产，选出「最新的、确实带目标平台包的版本」。
 *
 * # 为什么不能翻页扫全部 releases
 * electron/electron 这类巨型仓库（~3000 release）的 `/releases?per_page=100`
 * 分页响应体达数十 MB，在 30s 超时 + 代理下解码响应体超时，5 次内重试全败后被
 * 误判为不可重试错误，导致检查永久失败。tags 接口（`/tags`）每页仅 tag 名
 * （无资产），体积小、可早停，不会触发超时；二进制校验只拉命中 tag 对应的
 * 单个 release（`/releases/tags/{tag}`，体积小），不翻页。
 *
 * # 关键设计：asset 正则 vs tag 正则（2026-10-08 修复）
 * `version_extract_regex` 在 DB 中是单一字段，但有两种互斥语义：
 * - **tag 正则**（`v10\.\d+\.\d+`）→ 用于在 tag 列表里筛选目标 major 线
 * - **asset 正则**（`_(\d+\.\d+\.\d+)_amd64\.deb`）→ 用于在 release 资产名里
 *   挑平台包并顺带提取版本
 *
 * asset 正则拿去匹配 tag 名**永远为 false**（tag 形如 `v1.3.1`，不含
 * `_1.3.1_amd64.deb`）。原实现直接用同一个正则做两件事，于是错误地判定
 * 「tags 中未找到匹配正则的版本」而提前返回 None —— 这正是
 * ai-assistant-studio-bin（v2.1.0/v2.0.0 只发 Windows exe，v1.3.1 才有 deb）
 * 返回空版本的根因。
 *
 * 修复后策略：先用「版本形状」筛出候选 tag（倒序），再逐个用 asset 正则
 * 校验其 release 资产，首个命中即最新可用版本。
 */
use log::{debug, info, warn};
use reqwest::Client;

use crate::checkers::github::binary_check::{extract_version_from_assets, has_linux_binary};
use crate::checkers::github::graphql_batch_helpers::has_file_extension;
use crate::checkers::github::graphql_batch_tags::fill_tags;
use crate::checkers::github::release::build_github_request;
use crate::checkers::github::tags::tag_looks_like_version;
use crate::checkers::utils::clean_version;
use crate::errors::AppResult;

/// 候选 tag 校验上限，避免极端仓库产生过多单 release 请求
const MAX_CANDIDATE_VERIFY: usize = 30;

/// tags 翻页上限，与 cache_fill 一致（覆盖 electron 的 v2–v46 全部 major 线）
const MAX_TAGS_PAGES: u32 = 60;

/// 从 tags 解析候选版本并验证其 release 资产含目标平台包
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner` / `repo`: 仓库所有者与名称
/// - `token`: GitHub API Token（可选）
/// - `version_extract_regex`: 版本提取正则（自动判定 tag / asset 语义）
/// - `pkgname`: 软件包名（用于日志）
///
/// # 返回
/// - `Ok(Some(version))`：最新一个「其 release 含目标平台包」的版本
/// - `Ok(None)`：tags 为空、无候选、或所有候选均无匹配资产
pub async fn resolve_via_tags(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    version_extract_regex: Option<&str>,
    pkgname: &str,
) -> AppResult<Option<String>> {
    let asset_regex = version_extract_regex.filter(|r| has_file_extension(r));
    let tag_regex = version_extract_regex.filter(|r| !has_file_extension(r));

    // 1) 拉取全量 tags。asset 正则场景下不用它做早停（tag 名匹配不到），
    //    按 pushed_at 倒序扫全量；tag 正则场景可命中即早停省请求。
    let all_tags = fill_tags(client, owner, repo, token, &[], tag_regex, MAX_TAGS_PAGES).await;
    if all_tags.is_empty() {
        debug!("[二进制检查] {}: tags 列表为空", pkgname);
        return Ok(None);
    }

    // 2) 选候选 tag（已按 pushed_at DESC，最新在前）
    let candidates = select_candidates(&all_tags, tag_regex);
    if candidates.is_empty() {
        debug!(
            "[二进制检查] {}: tags 中未找到匹配 {:?} 的版本",
            pkgname, tag_regex
        );
        return Ok(None);
    }

    // 3) 倒序逐个校验候选 tag 的 release 资产，首个命中即返回
    for tag in candidates.iter().take(MAX_CANDIDATE_VERIFY) {
        if let Some(version) =
            verify_tag_assets(client, owner, repo, token, tag, asset_regex, pkgname).await
        {
            return Ok(Some(version));
        }
    }

    info!(
        "[二进制检查] {}: 前 {} 个候选 tag 均无匹配 Linux 资产，放弃",
        pkgname,
        candidates.len().min(MAX_CANDIDATE_VERIFY)
    );
    Ok(None)
}

/// 从 tags 列表挑选候选 tag
///
/// - `tag_regex` 为 `Some`（tag 语义）：只保留正则命中的 tag（如 electron v10 线）；
///   无命中时返回空 vec，调用方据此判定"该 major 线在本仓库不存在"
/// - `tag_regex` 为 `None`（asset 语义或无正则）：退化为「形状像版本号」的 tag，
///   最终版本交由资产校验决定
fn select_candidates(all_tags: &[String], tag_regex: Option<&str>) -> Vec<String> {
    match tag_regex {
        Some(re) => match regex::Regex::new(re) {
            Ok(compiled) => all_tags
                .iter()
                .filter(|t| compiled.is_match(t))
                .cloned()
                .collect(),
            Err(e) => {
                debug!("[二进制检查] tag 正则编译失败 {}，无候选", e);
                Vec::new()
            }
        },
        None => all_tags
            .iter()
            .filter(|t| tag_looks_like_version(t))
            .cloned()
            .collect(),
    }
}

/// 校验单个 tag 对应 release 的资产是否满足要求
///
/// # 返回
/// - `Some(version)`：该 release 含目标平台包，version 为从资产名提取的版本
///   （或 tag 去 v 前缀）
/// - `None`：请求失败 / 无附件 / 资产不匹配
async fn verify_tag_assets(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    tag: &str,
    asset_regex: Option<&str>,
    pkgname: &str,
) -> Option<String> {
    let release_url = format!(
        "https://api.github.com/repos/{}/{}/releases/tags/{}",
        owner, repo, tag
    );
    let data = match build_github_request(client, &release_url, token)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(d) => d,
            Err(e) => {
                warn!(
                    "[二进制检查] {}: tag '{}' release 解析失败: {}",
                    pkgname, tag, e
                );
                return None;
            }
        },
        Ok(resp) => {
            debug!(
                "[二进制检查] {}: tag '{}' 的 release 请求 HTTP {}",
                pkgname,
                tag,
                resp.status()
            );
            return None;
        }
        Err(e) => {
            debug!(
                "[二进制检查] {}: tag '{}' release 请求失败: {}",
                pkgname, tag, e
            );
            return None;
        }
    };

    let Some(assets) = data["assets"].as_array().filter(|a| !a.is_empty()) else {
        warn!(
            "[二进制检查] {}: tag '{}' 的 release 无附件，跳过",
            pkgname, tag
        );
        return None;
    };

    // 有 asset 正则 → 必须命中该正则（精确的平台包）；
    // 无 asset 正则 → 退化为「存在任意非 Windows/macOS 资产」
    match asset_regex {
        Some(filter) if !has_linux_binary(assets, Some(filter)) => {
            debug!(
                "[二进制检查] {}: tag '{}' 无匹配资产 {:?}，继续下一个候选",
                pkgname,
                tag,
                assets
                    .iter()
                    .filter_map(|a| a["name"].as_str())
                    .collect::<Vec<_>>()
            );
            return None;
        }
        None if !has_linux_binary(assets, None) => {
            debug!(
                "[二进制检查] {}: tag '{}' 无 Linux 平台资产，继续下一个候选",
                pkgname, tag
            );
            return None;
        }
        _ => {}
    }

    // 优先从资产名提取版本（tag 可能是 `latest` 而真实版本在文件名里），
    // 否则用 tag 去掉 v 前缀
    let version = asset_regex
        .and_then(|filter| extract_version_from_assets(assets, filter))
        .unwrap_or_else(|| clean_version(tag));
    info!(
        "[二进制检查] {}: 命中 tag '{}' 的匹配资产，上游版本={}",
        pkgname, tag, version
    );
    Some(version)
}
