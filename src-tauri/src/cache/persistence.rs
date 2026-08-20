/**
 * persistence.rs - 缓存磁盘读写
 *
 * 将内存缓存条目序列化为 JSON 写入缓存目录（原子写：临时文件 + rename），
 * 启动时读取未过期的缓存文件重建内存缓存。
 * 文件名由 CacheDomain 枚举白名单映射，不拼接用户输入，无路径遍历风险。
 */
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::errors::AppResult;

use super::domain::CacheDomain;
use super::config::CacheConfig;

/// 磁盘缓存文件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMeta {
    /// 缓存域名称（对应 CacheDomain.file_name()）
    pub domain: String,
    /// 创建时间（Unix 秒）
    pub created_at: i64,
    /// 过期时间（Unix 秒），0 表示永不过期
    pub expires_at: i64,
    /// 数据条目数（JSON 数组长度；非数组为 1）
    pub size: usize,
}

/// 磁盘缓存文件内容（meta + data）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedCache {
    pub meta: CacheMeta,
    /// 缓存数据体（序列化为具体领域类型）
    pub data: serde_json::Value,
}

/// 读取缓存文件
/// @param path - 缓存文件完整路径
/// @returns 文件不存在时返回 None
pub fn load_file(path: &Path) -> AppResult<Option<PersistedCache>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path)?;
    let pc: PersistedCache = serde_json::from_str(&content)?;
    Ok(Some(pc))
}

/// 原子写入缓存文件（先写 .tmp 再 rename，崩溃不残留半截文件）
/// @param path - 目标文件路径
/// @param domain - 缓存域（用于元数据）
/// @param data - 缓存数据体
/// @param expires_at - 过期时间（Unix 秒），0 表示永不过期
pub fn save_file(
    path: &Path,
    domain: CacheDomain,
    data: &serde_json::Value,
    expires_at: i64,
) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let meta = CacheMeta {
        domain: domain.file_name().to_string(),
        created_at: CacheConfig::now_ts(),
        expires_at,
        size: json_size(data),
    };
    let pc = PersistedCache {
        meta,
        data: data.clone(),
    };
    let json = serde_json::to_vec_pretty(&pc)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// 删除缓存文件（不存在时静默成功）
/// @param path - 缓存文件完整路径
pub fn remove_file(path: &Path) -> AppResult<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// 估算数据条目数：JSON 数组取长度，其余为 1
fn json_size(data: &serde_json::Value) -> usize {
    match data {
        serde_json::Value::Array(arr) => arr.len(),
        _ => 1,
    }
}
