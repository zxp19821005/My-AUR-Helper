/**
 * enums.rs - 枚举值管理命令
 *
 * 提供 License 和编程语言的增删改查功能
 * 支持从 SPDX 官方源同步 License 数据
 */
use log::{debug, info};
use tauri::State;

use crate::errors::{AppError, AppResult};
use crate::models::*;
use crate::AppState;

/// 获取所有 License 列表
#[tauri::command]
pub async fn get_licenses(state: State<'_, AppState>) -> AppResult<Vec<EnumLicense>> {
    debug!("正在获取所有 License");
    let db = state.db.lock()?;
    let result = db.get_all_licenses()?;
    info!("已获取 {} 个 License", result.len());
    Ok(result)
}

/// 从 SPDX 同步 License 数据
#[tauri::command]
pub async fn sync_licenses_from_spdx(state: State<'_, AppState>) -> AppResult<usize> {
    info!("正在从 SPDX 同步 License 数据");
    let client = reqwest::Client::new();
    let resp = client
        .get("https://raw.githubusercontent.com/spdx/license-list-data/main/json/licenses.json")
        .send()
        .await?;
    let data: serde_json::Value = resp.json().await?;
    let licenses = data["licenses"]
        .as_array()
        .ok_or_else(|| AppError::ParseError("SPDX 数据格式错误".into()))?;

    let mut enum_licenses = Vec::new();
    for lic in licenses {
        let spdx_id = lic["licenseId"].as_str().unwrap_or("");
        let full_name = lic["name"].as_str().unwrap_or("");
        let url = lic["seeAlso"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let is_deprecated = lic["isDeprecatedLicenseId"].as_bool().unwrap_or(false);
        let is_osi_approved = lic["isOsiApproved"].as_bool().unwrap_or(false);
        let category = lic["licenseCategory"].as_str().map(|s| s.to_string());
        enum_licenses.push(EnumLicense {
            id: None,
            spdx_id: spdx_id.to_string(),
            full_name: full_name.to_string(),
            url,
            is_deprecated,
            is_osi_approved,
            description: None,
            category,
            created_at: None,
        });
    }

    let db = state.db.lock()?;
    db.delete_all_licenses()?;
    let count = enum_licenses.len();
    for lic in &enum_licenses {
        let _ = db.upsert_license(lic);
    }
    info!("已从 SPDX 同步 {} 个 License", count);
    Ok(count)
}

/// 添加新的 License
#[tauri::command]
pub async fn add_license(
    state: State<'_, AppState>,
    spdx_id: String,
    full_name: String,
    url: Option<String>,
    description: Option<String>,
    category: Option<String>,
) -> AppResult<i64> {
    info!("正在添加 License: {} ({})", spdx_id, full_name);
    let lic = EnumLicense {
        id: None,
        spdx_id,
        full_name,
        url,
        is_deprecated: false,
        is_osi_approved: false,
        description,
        category,
        created_at: None,
    };
    let db = state.db.lock()?;
    db.upsert_license(&lic)
}

/// 获取所有编程语言列表
#[tauri::command]
pub async fn get_languages(state: State<'_, AppState>) -> AppResult<Vec<EnumProgrammingLanguage>> {
    debug!("正在获取所有编程语言");
    let db = state.db.lock()?;
    let result = db.get_all_languages()?;
    info!("已获取 {} 种编程语言", result.len());
    Ok(result)
}

/// 添加或更新编程语言
#[tauri::command]
pub async fn upsert_language(
    state: State<'_, AppState>,
    language: EnumProgrammingLanguage,
) -> AppResult<i64> {
    info!("正在添加/更新编程语言: {}", language.name);
    let db = state.db.lock()?;
    db.upsert_language(&language)
}

/// 获取单个 License
#[tauri::command]
pub async fn get_license(state: State<'_, AppState>, id: i64) -> AppResult<Option<EnumLicense>> {
    debug!("正在获取 License: {}", id);
    let db = state.db.lock()?;
    db.get_license_by_id(id)
}

/// 更新 License
#[tauri::command]
pub async fn update_license(
    state: State<'_, AppState>,
    id: i64,
    spdx_id: String,
    full_name: String,
    url: Option<String>,
    description: Option<String>,
    category: Option<String>,
) -> AppResult<()> {
    info!("正在更新 License {}: {} ({})", id, spdx_id, full_name);
    let lic = EnumLicense {
        id: Some(id),
        spdx_id,
        full_name,
        url,
        is_deprecated: false,
        is_osi_approved: false,
        description,
        category,
        created_at: None,
    };
    let db = state.db.lock()?;
    db.update_license(&lic)
}

/// 删除 License
#[tauri::command]
pub async fn delete_license(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    info!("正在删除 License: {}", id);
    let db = state.db.lock()?;
    db.delete_license_by_id(id)
}

/// 删除编程语言
#[tauri::command]
pub async fn delete_language(state: State<'_, AppState>, name: String) -> AppResult<()> {
    info!("正在删除编程语言: {}", name);
    let db = state.db.lock()?;
    db.delete_language(&name)
}
