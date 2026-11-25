//! cache_preload.rs - GitHub tags 缓存预置命令
//!
//! 背景：electron/electron 这类巨型仓库的检查走 `github_api` 路径，该路径从不回填
//! tags 缓存，导致 `github_tag_cache` 表长期为空，每次检查都实时翻 releases（易超时）。
//! 正常的自动回填（`cache_fill`）只在「本次检查成功」后触发，而该仓库检查常因超时失败，
//! 形成死锁——缓存永不写入。
//!
//! 本命令提供手动入口：主动为指定仓库（或库内全部 GitHub 仓库）拉取 tags 并写入缓存，
//! 使后续检查直接命中缓存、零网络、瞬时完成。等价于「提前跑一次成功的检查」来播种缓存。

use log::info;
use std::collections::HashSet;
use tauri::State;

use super::cache_fill::{fetch_repo_tags, write_back};
use super::utils::{build_checker_settings, read_http_timeout};
use crate::checkers::utils::extract_owner_repo;
use crate::errors::AppResult;
use crate::models::{CheckerType, PackageType};
use crate::AppState;

/// 预置 GitHub tags 缓存
///
/// 为指定仓库（传入 owner+repo）或库内全部 GitHub 上游仓库拉取 tags 并写入 `github_tag_cache`。
/// 写入后，后续上游检查对这些仓库将直接命中缓存，不再发起网络请求。
///
/// # 参数
/// - `owner` / `repo`：可选。都提供时只预置该仓库；任一缺失时预置库内全部 GitHub 仓库。
///
/// # 返回
/// - `Ok(n)`：成功写入缓存的仓库数量
#[tauri::command]
#[specta::specta]
pub async fn preload_github_tag_cache(
    state: State<'_, AppState>,
    owner: Option<String>,
    repo: Option<String>,
) -> AppResult<u32> {
    // 在锁内收集目标仓库与配置，避免跨 await 持有 db 引用
    let (timeout, github_token, repos) = {
        let db = state.db.lock()?;
        let timeout = read_http_timeout(&db);
        let github_token = build_checker_settings(&db).github_token;
        let repos: Vec<(String, String)> = if let (Some(o), Some(r)) = (owner, repo) {
            vec![(o, r)]
        } else {
            let packages = db.get_all_software()?;
            packages
                .into_iter()
                .filter_map(|sw| {
                    if !matches!(
                        sw.checker_type_id,
                        CheckerType::GitHubTags | CheckerType::GitHubAPI
                    ) || sw.package_type_id == PackageType::Git
                    {
                        return None;
                    }
                    extract_owner_repo(&sw.upstream_url.unwrap_or_default())
                })
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        };
        (timeout, github_token, repos)
    };

    if repos.is_empty() {
        info!("[GitHub Cache] 预置缓存：无目标仓库");
        return Ok(0);
    }

    let client = super::super::proxy_utils::build_client(timeout, true);
    let tags_map = fetch_repo_tags(&client, repos, github_token.as_deref()).await;
    let count = tags_map.len() as u32;
    write_back(&*state.db.lock()?, tags_map)?;
    info!("[GitHub Cache] 预置缓存完成，写入 {} 个仓库", count);
    Ok(count)
}
