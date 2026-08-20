/**
 * manager.rs - 内存缓存管理器
 *
 * 缓存主体：HashMap<CacheDomain, CacheEntry>，提供读取回源、失效、写盘、清空、统计。
 * 生命周期：启动时 load_from_disk 重建未过期条目；运行中 get_or_load 回源填充 /
 * invalidate 失效；定时或退出时 flush 写盘。
 *
 * 锁序约定（防死锁）：所有命令先取 memory_cache 锁，再取 db 锁。
 */
use std::collections::HashMap;

use serde::{de::DeserializeOwned, Serialize};

use crate::errors::AppResult;
use crate::models::{CacheDomainStats, MemoryCacheStats};

use super::config::CacheConfig;
use super::domain::CacheDomain;
use super::persistence;

/// 内存缓存条目
struct CacheEntry {
    /// 缓存数据体（序列化为具体领域类型）
    data: serde_json::Value,
    /// 创建时间（Unix 秒）
    created_at: i64,
    /// 过期时间（Unix 秒），0 表示永不过期
    expires_at: i64,
    /// 是否有未写盘的修改
    dirty: bool,
    /// 最近访问时间（LRU 淘汰依据）
    last_accessed: i64,
}

/// 内存缓存管理器
pub struct CacheManager {
    config: CacheConfig,
    entries: HashMap<CacheDomain, CacheEntry>,
}

impl CacheManager {
    /// 创建缓存管理器（配置从 settings 表读取）
    /// @param config - 缓存配置
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: HashMap::new(),
        }
    }

    /// 读取缓存配置（供 lib.rs 定时任务使用）
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// 启动时从磁盘加载未过期的缓存条目
    /// @returns 加载结果（目录不存在等错误向上传递，调用方记录日志即可）
    pub fn load_from_disk(&mut self) -> AppResult<()> {
        if !self.config.enabled {
            return Ok(());
        }
        let now = CacheConfig::now_ts();
        for domain in CacheDomain::all() {
            if !domain.persistent() || self.entries.contains_key(&domain) {
                continue;
            }
            let path = self.config.dir.join(format!("{}.json", domain.file_name()));
            let Some(pc) = persistence::load_file(&path)? else {
                continue;
            };
            // 过期条目丢弃（文件保留，由下次写覆盖）
            if pc.meta.expires_at != 0 && pc.meta.expires_at <= now {
                continue;
            }
            self.entries.insert(
                domain,
                CacheEntry {
                    data: pc.data,
                    created_at: pc.meta.created_at,
                    expires_at: pc.meta.expires_at,
                    dirty: false,
                    last_accessed: now,
                },
            );
        }
        Ok(())
    }

    /// 读取缓存：命中且未过期直接返回；miss 时执行 loader 回源 DB 并填充缓存
    /// @param domain - 缓存域
    /// @param loader - 回源加载闭包（在缓存锁内调用，须遵守 cache→db 锁序）
    /// @returns 领域数据
    pub fn get_or_load<T>(
        &mut self,
        domain: CacheDomain,
        loader: impl FnOnce() -> AppResult<T>,
    ) -> AppResult<T>
    where
        T: Serialize + DeserializeOwned,
    {
        if !self.config.enabled {
            return loader();
        }
        let now = CacheConfig::now_ts();
        // 命中且未过期
        if let Some(entry) = self.entries.get_mut(&domain) {
            if entry.expires_at == 0 || entry.expires_at > now {
                entry.last_accessed = now;
                return Ok(serde_json::from_value(entry.data.clone())?);
            }
            // 过期：移除后回源重建
            self.entries.remove(&domain);
        }
        // miss：回源加载并填充
        let data = loader()?;
        let value = serde_json::to_value(&data)?;
        self.entries.insert(
            domain,
            CacheEntry {
                data: value,
                created_at: now,
                expires_at: if self.config.ttl_secs > 0 {
                    now + self.config.ttl_secs
                } else {
                    0
                },
                dirty: true,
                last_accessed: now,
            },
        );
        self.evict_if_needed();
        Ok(data)
    }

    /// 使指定缓存域失效（写库成功后调用，下次读自动回源重建）
    /// @param domain - 缓存域
    pub fn invalidate(&mut self, domain: CacheDomain) {
        self.entries.remove(&domain);
    }

    /// 将脏缓存条目写盘（仅持久化域）
    /// @returns 实际写入的缓存域数量
    pub fn flush(&mut self) -> AppResult<usize> {
        if !self.config.enabled {
            return Ok(0);
        }
        let mut written = 0usize;
        for (domain, entry) in self.entries.iter_mut() {
            if !domain.persistent() || !entry.dirty {
                continue;
            }
            let path = self.config.dir.join(format!("{}.json", domain.file_name()));
            persistence::save_file(&path, *domain, &entry.data, entry.expires_at)?;
            entry.dirty = false;
            written += 1;
        }
        Ok(written)
    }

    /// 清空内存缓存与磁盘缓存文件
    pub fn clear(&mut self) -> AppResult<()> {
        self.entries.clear();
        for domain in CacheDomain::all() {
            if domain.persistent() {
                let path = self.config.dir.join(format!("{}.json", domain.file_name()));
                persistence::remove_file(&path)?;
            }
        }
        Ok(())
    }

    /// 生成缓存统计信息（供前端设置页展示）
    pub fn stats(&self) -> MemoryCacheStats {
        let mut domains = Vec::new();
        for domain in CacheDomain::all() {
            let entry = self.entries.get(&domain);
            domains.push(CacheDomainStats {
                domain: domain.file_name().to_string(),
                label: domain.label().to_string(),
                loaded: entry.is_some(),
                size: entry.map(|e| data_size(&e.data)).unwrap_or(0),
                created_at: entry.map(|e| e.created_at),
                expires_at: entry.map(|e| e.expires_at),
                persistent: domain.persistent(),
                file_size: self.disk_file_size(domain),
            });
        }
        let total_entries = self
            .entries
            .values()
            .map(|e| data_size(&e.data))
            .sum::<usize>();
        MemoryCacheStats {
            enabled: self.config.enabled,
            max_entries: self.config.max_entries,
            ttl_secs: self.config.ttl_secs,
            write_interval_secs: self.config.write_interval_secs,
            cache_dir: self.config.dir.display().to_string(),
            domains,
            total_entries,
        }
    }

    /// LRU 淘汰：条目数超过上限时移除最久未访问的条目
    fn evict_if_needed(&mut self) {
        if self.entries.len() <= self.config.max_entries {
            return;
        }
        let victim = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.last_accessed)
            .map(|(d, _)| *d);
        if let Some(domain) = victim {
            self.entries.remove(&domain);
            log::debug!("内存缓存 LRU 淘汰: {:?}", domain);
        }
    }

    /// 磁盘缓存文件大小（字节）
    fn disk_file_size(&self, domain: CacheDomain) -> u64 {
        std::fs::metadata(self.config.dir.join(format!("{}.json", domain.file_name())))
            .map(|m| m.len())
            .unwrap_or(0)
    }
}

/// 估算数据条目数：JSON 数组取长度，其余为 1
fn data_size(data: &serde_json::Value) -> usize {
    match data {
        serde_json::Value::Array(arr) => arr.len(),
        _ => 1,
    }
}
