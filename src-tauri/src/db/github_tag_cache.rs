//! github_tag_cache.rs - GitHub tags 缓存读写（按 owner+repo 存储快照，TTL 86400s）
use crate::errors::AppResult;
use log::debug;

use super::Database;
/// 默认缓存失效时间（秒）
pub const DEFAULT_CACHE_TTL_SECONDS: i64 = 86_400;

/// 缓存查询结果行
#[derive(Debug, Clone)]
pub struct GithubTagCacheRow {
    pub owner: String,
    pub repo: String,
    pub last_synced_at: i64, // Unix 时间戳（秒）
    pub tag_count: i32,
    pub data_json: String, // {"tags":[...],"releases":[...]}（按 pushedAt DESC）
    pub cached_version: Option<String>, // 增量校验用
}

/// 从缓存 JSON 构建 tags 列表。支持对象 `{"tags":[...]}` 和扁平数组 `[...]` 两种格式
pub fn decode_tags(data_json: &str) -> Vec<String> {
    let value: serde_json::Value = match serde_json::from_str(data_json) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    if let Some(tags) = value.get("tags").and_then(|v| v.as_array()) {
        return tags
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }
    if let Some(arr) = value.as_array() {
        return arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }
    Vec::new()
}

/// 从 tags 列表按正则提取版本（tags 按 pushedAt DESC，首个匹配即该 major 线最新）
/// 无正则或无匹配时取第一个 tag（仓库最新版本）。
pub fn recompute_version_from_tags(tags: &[String], regex: Option<&str>) -> Option<String> {
    if tags.is_empty() {
        return None;
    }
    if let Some(re_str) = regex {
        match regex::Regex::new(re_str) {
            Ok(re) => {
                for tag in tags {
                    if re.is_match(tag) {
                        return Some(tag.clone());
                    }
                }
            }
            Err(e) => debug!("[GitHub Cache] 正则编译失败: {}", e),
        }
    }
    tags.first().cloned()
}

/// 增量校验结果
#[derive(Debug, Clone)]
pub enum CacheCheckResult {
    /// 缓存有效，无需更新
    Hit,
    /// 版本一致，已顺延 TTL
    Extend,
    /// 版本不一致，已用缓存 tags 重新计算版本并更新
    Recalculate { new_version: String },
    /// 仓库无法访问或无版本信息，已清除缓存
    Deleted,
    /// 无缓存记录
    Miss,
}

/// 从缓存的 tags JSON 中重新计算最新版本（便捷封装）
pub fn recompute_version_from_cache(data_json: &str, regex: Option<&str>) -> Option<String> {
    let tags = decode_tags(data_json);
    recompute_version_from_tags(&tags, regex)
}

impl Database {
    /// 获取当前生效的缓存 TTL（秒），读 settings.github_cache_ttl，默认为 86400
    pub fn get_cache_ttl_seconds(&self) -> AppResult<i64> {
        match self.conn.query_row(
            "SELECT value FROM settings WHERE key = 'github_cache_ttl'",
            [],
            |row| row.get::<_, String>(0),
        ) {
            Ok(v) => Ok(v.parse::<i64>().unwrap_or(DEFAULT_CACHE_TTL_SECONDS)),
            Err(_) => Ok(DEFAULT_CACHE_TTL_SECONDS),
        }
    }

    /// 查询单条缓存（不检查 TTL）。找不到行时返回 Err(QueryReturnedNoRows)
    pub fn get_cache(&self, owner: &str, repo: &str) -> AppResult<Option<GithubTagCacheRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT owner, repo, last_synced_at, tag_count, data_json, cached_version
             FROM github_tag_cache
             WHERE owner = ?1 AND repo = ?2",
        )?;
        let row = stmt.query_row([owner, repo], |row| {
            Ok(GithubTagCacheRow {
                owner: row.get(0)?,
                repo: row.get(1)?,
                last_synced_at: row.get(2)?,
                tag_count: row.get(3)?,
                data_json: row.get(4)?,
                cached_version: row.get(5).ok(),
            })
        })?;
        // query_row 在找不到行时会返回 Error::QueryReturnedNoRows
        Ok(Some(row))
    }

    /// 批量查询所有缓存（用于统计/管理界面展示）
    pub fn list_all_caches(&self) -> AppResult<Vec<GithubTagCacheRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT owner, repo, last_synced_at, tag_count, data_json, cached_version
             FROM github_tag_cache
             ORDER BY last_synced_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(GithubTagCacheRow {
                owner: row.get(0)?,
                repo: row.get(1)?,
                last_synced_at: row.get(2)?,
                tag_count: row.get(3)?,
                data_json: row.get(4)?,
                cached_version: row.get(5).ok(),
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 写入或更新缓存（UPSERT）
    pub fn upsert_cache(
        &self,
        owner: &str,
        repo: &str,
        tag_count: i32,
        data_json: &str,
        cached_version: Option<&str>,
    ) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO github_tag_cache (owner, repo, last_synced_at, tag_count, data_json, cached_version)
             VALUES (?1, ?2, unixepoch(), ?3, ?4, ?5)
             ON CONFLICT(owner, repo) DO UPDATE SET
                 last_synced_at   = excluded.last_synced_at,
                 tag_count        = excluded.tag_count,
                 data_json        = excluded.data_json,
                 cached_version   = excluded.cached_version",
            rusqlite::params![owner, repo, tag_count, data_json, cached_version],
        )?;
        Ok(())
    }

    /// 删除单条缓存
    pub fn delete_cache(&self, owner: &str, repo: &str) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM github_tag_cache WHERE owner = ?1 AND repo = ?2",
            rusqlite::params![owner, repo],
        )?;
        Ok(())
    }

    /// 清除所有过期缓存（last_synced_at < now - ttl_seconds）
    pub fn clear_expired_caches(&self) -> AppResult<i64> {
        let ttl = self.get_cache_ttl_seconds()?;
        let now = chrono::Utc::now().timestamp();
        let cutoff = now - ttl;
        let rows = self.conn.execute(
            "DELETE FROM github_tag_cache WHERE last_synced_at < ?1",
            rusqlite::params![cutoff],
        )?;
        Ok(rows as i64)
    }

    /// 清除全部缓存
    pub fn clear_all_caches(&self) -> AppResult<i64> {
        let rows = self.conn.execute("DELETE FROM github_tag_cache", [])?;
        Ok(rows as i64)
    }

    /// 统计缓存条目总数
    pub fn cache_count(&self) -> AppResult<i64> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM github_tag_cache", [], |row| {
                    row.get(0)
                })?;
        Ok(count)
    }
}
