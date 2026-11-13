//! github_tag_cache_extend.rs - GitHub tags 缓存增量校验（check_and_extend_cache）
use crate::db::github_tag_cache::{recompute_version_from_cache, CacheCheckResult};
use crate::db::Database;
use crate::errors::AppResult;
use log::{info, warn};

impl Database {
    /// 增量校验：缓存有效→Hit；过期+版本一致→Extend；过期+版本变→Recalculate；不可访问→Deleted；无缓存→Miss
    /// 同步函数。内部阻塞 HTTP 请求已用 std::thread::spawn 隔离到独立 OS 线程执行，
    /// 因此可直接在 tokio worker 线程（异步命令上下文）内同步调用，无需调用方包裹 spawn_blocking。
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

        // 缓存过期：调 releases/latest 验证版本。
        // 注意：本函数由异步命令（check_selected_upstream / check_all_upstream）在
        // tokio worker 线程内同步调用，而 reqwest::blocking 会在调用线程创建并销毁一个
        // 内部 tokio runtime；在已有 runtime 上下文里这么做会触发
        // "Cannot drop a runtime in a context where blocking is not allowed" panic。
        // 因此用 std::thread::spawn 把阻塞请求放到独立 OS 线程执行——该线程没有 runtime
        // 上下文，blocking client 可正常创建与销毁其 runtime。
        let tags_url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        );
        let (status_ok, github_version) =
            match std::thread::spawn(move || match reqwest::blocking::get(&tags_url) {
                Ok(resp) => {
                    let status_ok = resp.status().is_success();
                    let version = if status_ok {
                        resp.json::<serde_json::Value>().ok().and_then(|v| {
                            v.get("tag_name").and_then(|x| x.as_str()).map(String::from)
                        })
                    } else {
                        None
                    };
                    (status_ok, version)
                }
                Err(_) => (false, None),
            })
            .join()
            {
                Ok(tuple) => tuple,
                Err(_) => {
                    warn!(
                        "[GitHub Cache] {}:{} 请求线程异常（可能 panic）",
                        owner, repo
                    );
                    (false, None)
                }
            };
        if !status_ok {
            // 仓库不可访问或请求线程异常：清除缓存
            let _ = self.delete_cache(owner, repo);
            info!("[GitHub Cache] {}:{} 仓库不可访问，已清除缓存", owner, repo);
            return Ok(CacheCheckResult::Deleted);
        }
        let github_version = match github_version {
            Some(v) => v,
            None => {
                // 仓库无 release 信息：清除缓存
                let _ = self.delete_cache(owner, repo);
                info!(
                    "[GitHub Cache] {}:{} 无 release 信息，已清除缓存",
                    owner, repo
                );
                return Ok(CacheCheckResult::Deleted);
            }
        };

        let cached_ver = row.cached_version.as_deref().unwrap_or("");
        if cached_ver == github_version {
            // 版本一致：顺延 TTL
            let new_synced_at = now + ttl;
            self.conn.execute(
                "UPDATE github_tag_cache SET last_synced_at = ?1 WHERE owner = ?2 AND repo = ?3",
                rusqlite::params![new_synced_at, owner, repo],
            )?;
            info!(
                "[GitHub Cache] {}:{} 版本一致({})，顺延 TTL +{}s",
                owner, repo, github_version, ttl
            );
            Ok(CacheCheckResult::Extend)
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
                    return Ok(CacheCheckResult::Recalculate {
                        new_version: new_ver,
                    });
                }
            }
            // 无缓存 tags：返回需要全量重拉
            info!(
                "[GitHub Cache] {}:{} 版本变化 {} -> {}，需要全量重拉",
                owner, repo, cached_ver, github_version
            );
            Ok(CacheCheckResult::Recalculate {
                new_version: github_version,
            })
        }
    }
}
