/**
 * batch_cache.rs - GitHub tags 缓存写盘辅助函数
 *
 * 功能：
 * - query_valid_github_caches: 按 (owner, repo) 批量查询有效缓存
 * - write_github_tag_cache: 将批量检查结果中的仓库 tags 写入 SQLite 缓存表
 *
 * 设计要点：
 * - 本模块仅负责缓存表的读写，不含检查逻辑
 * - write_github_tag_cache 由调用方在 batch_check_upstream 完成后按需调用
 */
use std::collections::{HashMap, HashSet};

use log::info;

use crate::db::github_tag_cache::GithubTagCacheRow;
use crate::db::Database;
use crate::errors::AppResult;

/// GitHub tags 缓存辅助函数：按 (owner,repo) 批量查询有效缓存
///
/// 有效缓存定义：记录存在且 last_synced_at >= now - ttl_seconds
/// 返回 (命中缓存的仓库集合, 缓存行列表)，调用方可据此跳过对应包的网络请求
pub fn query_valid_github_caches(
    db: &Database,
    owners_repos: &[(String, String)],
) -> (HashSet<(String, String)>, Vec<GithubTagCacheRow>) {
    let ttl = db.get_cache_ttl_seconds().unwrap_or(86400);
    let now = chrono::Utc::now().timestamp();
    let cutoff = now - ttl;

    let mut hit_keys: HashSet<(String, String)> = HashSet::new();
    let mut valid_rows: Vec<GithubTagCacheRow> = Vec::new();

    for (owner, repo) in owners_repos {
        match db.get_cache(owner, repo) {
            Ok(Some(row)) if row.last_synced_at >= cutoff => {
                hit_keys.insert((owner.clone(), repo.clone()));
                valid_rows.push(row);
            }
            _ => {}
        }
    }
    (hit_keys, valid_rows)
}

/// 将批量检查结果中各仓库的 tags 写入缓存
///
/// 仅当所有包都已得到版本结果（无 fallback 回填）时调用；
/// 若存在 REST tags 回填，回填完成后会再次调用本函数覆盖旧缓存。
pub fn write_github_tag_cache(
    db: &Database,
    cache_map: &HashMap<(String, String), crate::checkers::github::graphql_batch::RepoCache>,
) -> AppResult<()> {
    for ((owner, repo), cache) in cache_map {
        let tags = &cache.snapshot.tags;
        let releases: Vec<serde_json::Value> = cache
            .snapshot
            .releases
            .iter()
            .map(|r| {
                serde_json::json!({
                    "tag_name": &r.tag_name,
                    "name": &r.name,
                    "is_prerelease": r.is_prerelease,
                    "is_draft": r.is_draft,
                    "created_at": &r.created_at,
                })
            })
            .collect();
        let data = serde_json::json!({"tags": tags, "releases": releases});
        let json_str = data.to_string();
        let tag_count = tags.len() as i32;
        let cached_ver = cache.snapshot.releases.first().map(|r| r.tag_name.clone());
        db.upsert_cache(owner, repo, tag_count, &json_str, cached_ver.as_deref())?;
        info!(
            "[GitHub Cache] 写入 {}:{}/{} tags, version={:?}",
            owner, repo, tag_count, cached_ver
        );
    }
    Ok(())
}
