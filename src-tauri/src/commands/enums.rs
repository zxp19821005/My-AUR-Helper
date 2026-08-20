/**
 * enums.rs - 枚举值管理命令
 *
 * 提供 License 和编程语言的增删改查功能，支持从 SPDX 官方源同步 License 数据。
 * 读取走内存缓存（Licenses / Languages），写库后使对应缓存失效（下次读自动回源）。
 * 锁序约定：先 memory_cache 锁，后 db 锁，防止与缓存命令死锁。
 */
use log::{debug, info};
use tauri::State;

use crate::cache::CacheDomain;
use crate::commands::sysops::proxy_utils::build_client;
use crate::commands::sysops::software_sync::utils::read_http_timeout;
use crate::errors::{AppError, AppResult};
use crate::models::*;
use crate::AppState;

/// 获取所有 License 列表（走 Licenses 缓存，miss 时回源 DB 全量加载并填充缓存）
#[tauri::command]
pub async fn get_licenses(state: State<'_, AppState>) -> AppResult<Vec<EnumLicense>> {
    debug!("正在获取所有 License");
    let mut cache = state.memory_cache.lock()?;
    let result: Vec<EnumLicense> = cache.get_or_load(CacheDomain::Licenses, || {
        let db = state.db.lock()?;
        db.get_all_licenses()
    })?;
    info!("已获取 {} 个 License", result.len());
    Ok(result)
}

/// 从 SPDX 同步 License 数据（不删除自定义 License），写库后使 Licenses 缓存失效
#[tauri::command]
pub async fn sync_licenses_from_spdx(state: State<'_, AppState>) -> AppResult<usize> {
    info!("正在从 SPDX 同步 License 数据");
    let timeout = {
        let db = state.db.lock()?;
        read_http_timeout(&db)
    };
    let client = build_client(timeout, true);
    let req = client
        .get("https://raw.githubusercontent.com/spdx/license-list-data/main/json/licenses.json");
    let resp = req.send().await?;
    let data: serde_json::Value = resp.json().await?;
    let licenses = data["licenses"]
        .as_array()
        .ok_or_else(|| AppError::ParseError("SPDX 数据格式错误".into()))?;

    let mut cache = state.memory_cache.lock()?;
    let db = state.db.lock()?;
    let mut count = 0usize;
    for lic in licenses {
        let spdx_id = lic["licenseId"].as_str().unwrap_or("");
        let full_name = lic["name"].as_str().unwrap_or("");
        if !spdx_id.is_empty() && !full_name.is_empty() {
            let _ = db.upsert_license(&EnumLicense {
                id: None,
                spdx_id: spdx_id.to_string(),
                full_name: full_name.to_string(),
            });
            count += 1;
        }
    }
    cache.invalidate(CacheDomain::Licenses);
    info!("已从 SPDX 同步 {} 个 License", count);
    Ok(count)
}

/// 添加新的 License（写库后使 Licenses 缓存失效）
#[tauri::command]
pub async fn add_license(
    state: State<'_, AppState>,
    spdx_id: String,
    full_name: String,
) -> AppResult<i64> {
    info!("正在添加 License: {} ({})", spdx_id, full_name);
    let mut cache = state.memory_cache.lock()?;
    let db = state.db.lock()?;
    let id = db.upsert_license(&EnumLicense {
        id: None,
        spdx_id,
        full_name,
    })?;
    cache.invalidate(CacheDomain::Licenses);
    Ok(id)
}

/// 获取所有编程语言列表（走 Languages 缓存，miss 时回源 DB 全量加载并填充缓存）
#[tauri::command]
pub async fn get_languages(state: State<'_, AppState>) -> AppResult<Vec<EnumProgrammingLanguage>> {
    debug!("正在获取所有编程语言");
    let mut cache = state.memory_cache.lock()?;
    let result: Vec<EnumProgrammingLanguage> = cache.get_or_load(CacheDomain::Languages, || {
        let db = state.db.lock()?;
        db.get_all_languages()
    })?;
    info!("已获取 {} 种编程语言", result.len());
    Ok(result)
}

/// 添加或更新编程语言（写库后使 Languages 缓存失效）
#[tauri::command]
pub async fn upsert_language(
    state: State<'_, AppState>,
    language: EnumProgrammingLanguage,
) -> AppResult<i64> {
    info!("正在添加/更新编程语言: {}", language.name);
    let mut cache = state.memory_cache.lock()?;
    let db = state.db.lock()?;
    let id = db.upsert_language(&language)?;
    cache.invalidate(CacheDomain::Languages);
    Ok(id)
}

/// 获取单个 License（从 Licenses 缓存整域查找，未命中时自动重建缓存）
#[tauri::command]
pub async fn get_license(state: State<'_, AppState>, id: i64) -> AppResult<Option<EnumLicense>> {
    debug!("正在获取 License: {}", id);
    let mut cache = state.memory_cache.lock()?;
    let all: Vec<EnumLicense> = cache.get_or_load(CacheDomain::Licenses, || {
        let db = state.db.lock()?;
        db.get_all_licenses()
    })?;
    Ok(all.into_iter().find(|l| l.id == Some(id)))
}

/// 更新 License（写库后使 Licenses 缓存失效）
#[tauri::command]
pub async fn update_license(
    state: State<'_, AppState>,
    id: i64,
    spdx_id: String,
    full_name: String,
) -> AppResult<()> {
    info!("正在更新 License {}: {} ({})", id, spdx_id, full_name);
    let mut cache = state.memory_cache.lock()?;
    let db = state.db.lock()?;
    db.update_license(&EnumLicense {
        id: Some(id),
        spdx_id,
        full_name,
    })?;
    cache.invalidate(CacheDomain::Licenses);
    Ok(())
}

/// 删除 License（写库后使 Licenses 缓存失效）
#[tauri::command]
pub async fn delete_license(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    info!("正在删除 License: {}", id);
    let mut cache = state.memory_cache.lock()?;
    let db = state.db.lock()?;
    db.delete_license_by_id(id)?;
    cache.invalidate(CacheDomain::Licenses);
    Ok(())
}

/// 删除编程语言（写库后使 Languages 缓存失效）
#[tauri::command]
pub async fn delete_language(state: State<'_, AppState>, name: String) -> AppResult<()> {
    info!("正在删除编程语言: {}", name);
    let mut cache = state.memory_cache.lock()?;
    let db = state.db.lock()?;
    db.delete_language(&name)?;
    cache.invalidate(CacheDomain::Languages);
    Ok(())
}
