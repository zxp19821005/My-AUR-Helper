/**
 * github_tag_cache.rs - GitHub tags 缓存读写
 *
 * 功能：
 * - 按 (owner, repo) 存储 GraphQL + REST 拉取的 tags/releases 快照
 * - TTL 机制：超过 ttl_seconds（默认 86400s = 24h）视为过期
 * - 批量检查前查缓存，命中则跳过网络请求
 * - 手动清理命令：清除全部 / 清除过期 / 按仓库清除
 *
 * 设计要点：
 * - PRIMARY KEY (owner, repo) 保证每仓库只有一条记录
 * - data_json 存 JSON：{"tags":[...], "releases":[...]}
 * - etag 字段预留 If-None-Match 优化（当前未使用，后续可扩展）
 */
use crate::errors::AppResult;
use log::{debug, info, warn};

use super::Database;

/// 默认缓存失效时间（秒）
pub const DEFAULT_CACHE_TTL_SECONDS: i64 = 86_400;

/// 查询结果行
#[derive(Debug, Clone)]
pub struct GithubTagCacheRow {
    pub owner: String,
    pub repo: String,
    /// 最后同步时间（Unix 时间戳，秒）
    pub last_synced_at: i64,
    /// 缓存的 tag 数量
    pub tag_count: i32,
    /// JSON 数组：["v44.1.1", "v44.1.0", ...]（按 pushedAt DESC）
    pub data_json: String,
    /// 上次检查到的上游版本（用于增量校验）
    pub cached_version: Option<String>,
}

/// 从缓存行构建 tags 列表（反序列化 JSON 数组）
pub fn decode_tags(data_json: &str) -> Vec<String> {
    match serde_json::from_str::<Vec<String>>(data_json) {
        Ok(tags) => tags,
        Err(_) => Vec::new(),
    }
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

/// 从缓存的 tags JSON 中重新计算最新版本
/// 使用第一个包的正则（ callers 传入 regex ）从已有 tags 中挑选版本
pub fn recompute_version_from_cache(data_json: &str, regex: Option<&str>) -> Option<String> {
    let tags = decode_tags(data_json);
    if tags.is_empty() {
        return None;
    }
    // tags 按 pushedAt DESC 排列，第一个匹配正则的就是最新版本
    if let Some(re) = regex.and_then(|r| match regex::Regex::new(r) {
        Ok(rx) => Some(rx),
        Err(e) => {
            debug!("[GitHub Cache] 正则编译失败: {}", e);
            None
        }
    }) {
        for tag in &tags {
            if re.is_match(tag) {
                return Some(tag.clone());
            }
        }
    }
    // 无正则时取第一个 tag（即最新版本）
    tags.first().cloned()
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

    /// 查询单条缓存（不检查 TTL，调用方自行判断是否过期）
    /// @returns None 表示缓存不存在
    pub fn get_cache(&self, owner: &str, repo: &str) -> AppResult<Option<GithubTagCacheRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT owner, repo, last_synced_at, tag_count, data_json, cached_version
             FROM github_tag_cache
             WHERE owner = ?1 AND repo = ?2",
        )?;
        let row = stmt.query_row([owner, repo], |row| Ok(GithubTagCacheRow {
            owner: row.get(0)?,
            repo: row.get(1)?,
            last_synced_at: row.get(2)?,
            tag_count: row.get(3)?,
            data_json: row.get(4)?,
            cached_version: row.get(5).ok(),
        }))?;
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
        let rows = stmt.query_map([], |row| Ok(GithubTagCacheRow {
            owner: row.get(0)?,
            repo: row.get(1)?,
            last_synced_at: row.get(2)?,
            tag_count: row.get(3)?,
            data_json: row.get(4)?,
            cached_version: row.get(5).ok(),
        }))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 写入或更新缓存（UPSERT）
    /// @param owner       仓库所有者
    /// @param repo        仓库名
    /// @param tag_count   本次缓存的 tag 数量
    /// @param data_json   JSON 快照字符串
    /// @param cached_version 本次检查到的上游版本（可为 None）
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
    /// @returns 删除的行数
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
    /// @returns 删除的行数
    pub fn clear_all_caches(&self) -> AppResult<i64> {
        let rows = self.conn.execute("DELETE FROM github_tag_cache", [])?;
        Ok(rows as i64)
    }

    /// 统计缓存条目总数
    pub fn cache_count(&self) -> AppResult<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM github_tag_cache",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// 增量校验：检查缓存是否过期，如过期则调 GitHub API 验证版本
    ///
    /// - 缓存有效 → CacheCheckResult::Hit
    /// - 缓存过期 + 版本一致 → CacheCheckResult::Extend（顺延 TTL）
    /// - 缓存过期 + 版本不一致 + 有 tags → CacheCheckResult::Recalculate（用缓存 tags 重算）
    /// - 缓存过期 + 仓库不可访问 → CacheCheckResult::Deleted
    /// - 无缓存 → CacheCheckResult::Miss
    ///
    /// @param owner    仓库所有者
    /// @param repo     仓库名
    /// @param regex    版本提取正则（可选，用于从缓存 tags 重算版本）
    ///
    /// 注意：此方法为同步函数，内部使用 reqwest::blocking 进行 HTTP 请求，
    /// 调用方应通过 spawn_blocking 包裹以避免阻塞异步运行时。
    pub fn check_and_extend_cache(
        &self,
        owner: &str,
        repo: &str,
        regex: Option<&str>,
    ) -> AppResult<CacheCheckResult> {
        let ttl = self.get_cache_ttl_seconds()?;
        let now = chrono::Utc::now().timestamp();

        // 查缓存
        let row = match self.get_cache(owner, repo)? {
            Some(r) => r,
            None => return Ok(CacheCheckResult::Miss),
        };

        // 缓存有效
        if row.last_synced_at >= now - ttl {
            return Ok(CacheCheckResult::Hit);
        }

        // 缓存过期：调 releases/latest 验证版本
        let tags_url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        );
        let resp = match reqwest::blocking::get(&tags_url) {
            Ok(r) => r,
            Err(e) => {
                warn!(
                    "[GitHub Cache] {}:{}/releases/latest 请求失败: {}",
                    owner, repo, e
                );
                return Ok(CacheCheckResult::Deleted);
            }
        };
        if !resp.status().is_success() {
            // 仓库不存在或无权限：清除缓存
            let _ = self.delete_cache(owner, repo);
            info!(
                "[GitHub Cache] {}:{} 仓库不可访问，已清除缓存",
                owner, repo
            );
            return Ok(CacheCheckResult::Deleted);
        }

        let release: serde_json::Value = match resp.json() {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    "[GitHub Cache] {}:{}/releases/latest 解析失败: {}",
                    owner, repo, e
                );
                return Ok(CacheCheckResult::Deleted);
            }
        };
        let github_version = release
            .get("tag_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        match github_version {
            Some(github_ver) => {
                let cached_ver = row.cached_version.as_deref().unwrap_or("");
                if cached_ver == github_ver {
                    // 版本一致：顺延 TTL
                    let new_synced_at = now + ttl;
                    self.conn.execute(
                        "UPDATE github_tag_cache SET last_synced_at = ?1 WHERE owner = ?2 AND repo = ?3",
                        rusqlite::params![new_synced_at, owner, repo],
                    )?;
                    info!(
                        "[GitHub Cache] {}:{} 版本一致({})，顺延 TTL +{}s",
                        owner, repo, github_ver, ttl
                    );
                    return Ok(CacheCheckResult::Extend);
                } else {
                    // 版本不一致：尝试从缓存 tags 重算
                    if !row.data_json.is_empty() {
                        if let Some(new_ver) = recompute_version_from_cache(&row.data_json, regex) {
                            // 更新版本号 + 重置 TTL
                            self.conn.execute(
                                "UPDATE github_tag_cache SET last_synced_at = ?1, cached_version = ?2 WHERE owner = ?3 AND repo = ?4",
                                rusqlite::params![now, new_ver, owner, repo],
                            )?;
                            info!(
                                "[GitHub Cache] {}:{} 版本变化: {} -> {}，从缓存 tags 重算",
                                owner, repo, cached_ver, new_ver
                            );
                            return Ok(CacheCheckResult::Recalculate { new_version: new_ver });
                        }
                    }
                    // 无缓存 tags：返回需要全量重拉
                    info!(
                        "[GitHub Cache] {}:{} 版本变化 {} -> {}，需要全量重拉",
                        owner, repo, cached_ver, github_ver
                    );
                    return Ok(CacheCheckResult::Recalculate {
                        new_version: github_ver,
                    });
                }
            }
            None => {
                // 仓库无 release：清除缓存
                let _ = self.delete_cache(owner, repo);
                return Ok(CacheCheckResult::Deleted);
            }
        }
    }
}
