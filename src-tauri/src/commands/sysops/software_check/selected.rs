//! selected.rs — 选中包上游版本检查（缓存校验→并行检查→批量写库）
use chrono::Utc;
use log::{error, info};
use std::collections::HashMap;
use tauri::State;

use super::super::proxy_utils::build_client;
use super::super::software_sync::batch::batch_check_upstream;
use super::super::software_sync::batch_cache::write_github_tag_cache;
use super::super::software_sync::batch_engine::PackageTask;
use super::super::software_sync::cache::check_github_cache;
use super::super::software_sync::cache_fill::{collect_missing_repos, fetch_repo_tags, write_back};
use super::super::software_sync::utils::{
    build_checker_settings, read_http_settings, UpstreamCheckResult,
};
use crate::errors::AppResult;
use crate::models::*;
use crate::versions;
use crate::AppState;

/// 写入单个软件包的上游检查结果
pub fn apply_upstream_check_result(
    conn: &rusqlite::Connection,
    software_id: i64,
    cleaned_version: &str,
    is_outdated: bool,
    upstream_license_id: Option<String>,
    language_ids: &[i64],
    fill_languages: bool,
) -> AppResult<()> {
    crate::db::Database::update_software_outdated_conn(conn, software_id, is_outdated)?;
    if fill_languages && !language_ids.is_empty() {
        crate::db::Database::update_software_languages_conn(conn, software_id, language_ids)?;
    }
    let upstream_info = UpstreamInfo {
        software_id,
        upstream_version: Some(cleaned_version.to_string()),
        upstream_license_id,
        last_checked: Some(Utc::now().timestamp()),
        upstream_url_status: None,
        last_check_error: None,
    };
    crate::db::Database::upsert_upstream_info_conn(conn, &upstream_info)?;
    Ok(())
}

/// 检查选中的软件包上游版本
#[tauri::command]
pub async fn check_selected_upstream(
    state: State<'_, AppState>,
    pkgname_list: Vec<String>,
) -> AppResult<Vec<(String, String)>> {
    info!("正在检查 {} 个软件包的上游版本", pkgname_list.len());

    let (packages, settings, timeout, retry) = {
        let db = state.db.lock()?;
        let mut packages = Vec::new();
        for pkgname in &pkgname_list {
            if let Some(sw) = db.get_software_by_name(pkgname)? {
                packages.push(sw);
            }
        }
        let settings = build_checker_settings(&db);
        let (timeout, retry) = read_http_settings(&db);
        (packages, settings, timeout, retry)
    };

    let lang_by_id: HashMap<i64, Vec<i64>> = packages
        .iter()
        .map(|p| (p.software_id.unwrap_or(0), p.language_ids.clone()))
        .collect();

    let aur_map: HashMap<i64, String> = {
        let db = state.db.lock()?;
        db.get_aur_versions_map()?
    };

    let mut tasks = Vec::new();
    for sw in &packages {
        let has_aur = aur_map
            .get(&sw.software_id.unwrap_or(0))
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        if has_aur {
            tasks.push(PackageTask {
                pkgname: sw.pkgname.clone(),
                software_id: sw.software_id.unwrap_or(0),
                upstream_url: sw.upstream_url.clone().unwrap_or_default(),
                version_extract_regex: sw.version_extract_regex.clone(),
                check_test_versions: sw.check_test_versions,
                check_binary_files: sw.check_binary_files,
                checker_type: sw.checker_type_id.clone(),
                package_type: sw.package_type_id.clone(),
            });
        }
    }

    if tasks.is_empty() {
        info!("[版本检查] 所有选中包均无 AUR 版本，跳过检查");
        return Ok(Vec::new());
    }

    let client = build_client(timeout, false);
    let github_client = build_client(timeout, true);

    // ---- GitHub tags 缓存：查有效缓存，跳过命中仓库的包 ----
    use crate::checkers::utils::extract_owner_repo;
    use crate::models::{CheckerType, PackageType};

    let github_repos: Vec<(String, String)> = tasks
        .iter()
        .filter(|t| {
            matches!(
                t.checker_type,
                CheckerType::GitHubTags | CheckerType::GitHubAPI
            ) && t.package_type != PackageType::Git
                && extract_owner_repo(&t.upstream_url).is_some()
        })
        .map(|t| {
            let (o, r) = extract_owner_repo(&t.upstream_url).unwrap();
            (o, r)
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let cache_summary = {
        let db = state.db.lock().unwrap();
        check_github_cache(&db, &tasks, &github_repos)
    };
    let skip_keys = cache_summary.skip_keys;
    let cache_hit_results = cache_summary.cache_hit_results;
    // 保留完整任务列表 / token / client：三者稍后都会被 move 进 batch_check_upstream
    let all_tasks = tasks.clone();
    let github_token = settings.github_token.clone();
    let fill_client = github_client.clone();
    let tasks: Vec<PackageTask> = tasks
        .into_iter()
        .filter(|t| {
            if !matches!(
                t.checker_type,
                CheckerType::GitHubTags | CheckerType::GitHubAPI
            ) {
                return true;
            }
            if let Some((owner, repo)) = extract_owner_repo(&t.upstream_url) {
                if skip_keys.contains(&(owner, repo)) {
                    return false;
                }
            }
            true
        })
        .collect();

    let outcome = batch_check_upstream(
        tasks,
        client,
        github_client,
        settings,
        retry,
        |_owner, _repo, _tag_count, _json_str| {},
    )
    .await;

    if !outcome.github_cache_map.is_empty() {
        let db = state.db.lock().unwrap();
        if let Err(e) = write_github_tag_cache(&db, &outcome.github_cache_map) {
            error!("[GitHub Cache] 写盘失败: {}", e);
        }
    }

    let mut check_results: Vec<UpstreamCheckResult> = Vec::new();
    for r in cache_hit_results {
        check_results.push(r);
    }
    for r in outcome.checked {
        if r.upstream_version.is_empty() {
            check_results.push(r);
            continue;
        }
        let aur_ver = aur_map
            .get(&r.software_id)
            .filter(|v| !v.is_empty())
            .map(|s| s.as_str());
        let is_outdated = match aur_ver {
            Some(aur) => {
                versions::compare_versions(aur, &r.upstream_version)
                    == versions::VersionComparison::LessThan
            }
            None => true,
        };
        check_results.push(UpstreamCheckResult {
            pkgname: r.pkgname,
            software_id: r.software_id,
            upstream_version: r.upstream_version,
            is_outdated,
            license_spdx_id: r.license_spdx_id,
            language_names: r.language_names,
            _all_tags: None,
            _owner: String::new(),
            _repo: String::new(),
        });
    }

    // 补写 REST 回落场景缺失的缓存：
    // GraphQL 失败时 batch 不产生快照，仓库不会进入 github_cache_map，
    // 若不在此补齐，这些仓库每次检查都会重复请求 GitHub API。
    {
        let missing = {
            let db = state.db.lock()?;
            collect_missing_repos(&db, &all_tasks, &check_results, &outcome.github_cache_map)
        };
        if !missing.is_empty() {
            let tags_map = fetch_repo_tags(&fill_client, missing, github_token.as_deref()).await;
            let db = state.db.lock()?;
            write_back(&db, tags_map)?;
        }
    }

    let lang_ids_by_sw: HashMap<i64, Vec<i64>> = {
        let db = state.db.lock()?;
        let mut map = HashMap::new();
        for result in &check_results {
            if !result.upstream_version.is_empty() && !result.language_names.is_empty() {
                map.insert(
                    result.software_id,
                    db.resolve_language_ids(&result.language_names)?,
                );
            }
        }
        map
    };

    let mut db = state.db.lock()?;
    let tx = db.conn.transaction()?;
    let mut results = Vec::new();
    for result in &check_results {
        if !result.upstream_version.is_empty() {
            let cleaned_version = result
                .upstream_version
                .strip_prefix('v')
                .unwrap_or(&result.upstream_version);
            let upstream_license_id = result.license_spdx_id.clone();
            let language_ids = lang_ids_by_sw
                .get(&result.software_id)
                .cloned()
                .unwrap_or_default();
            let fill_languages = lang_by_id
                .get(&result.software_id)
                .map(|l| l.is_empty())
                .unwrap_or(false)
                && !language_ids.is_empty();

            apply_upstream_check_result(
                &tx,
                result.software_id,
                cleaned_version,
                result.is_outdated,
                upstream_license_id,
                &language_ids,
                fill_languages,
            )?;

            info!(
                "[版本检查结果] {}: AUR={:?} 上游={} 需更新={}",
                result.pkgname,
                aur_map.get(&result.software_id),
                result.upstream_version,
                result.is_outdated
            );

            results.push((result.pkgname.clone(), result.upstream_version.clone()));
        } else {
            crate::db::Database::update_software_outdated_conn(&tx, result.software_id, false)?;
        }
    }
    tx.commit()?;

    for pkgname in outcome.manual {
        results.push((pkgname, "manual".to_string()));
    }

    info!("已完成 {} 个软件包的上游版本检查", results.len());
    Ok(results)
}
