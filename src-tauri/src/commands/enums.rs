/**
 * enums.rs - 枚举值管理命令
 *
 * 提供 License 和编程语言的增删改查功能
 * 支持从 SPDX 官方源同步 License 数据
 */
use log::{info, debug};     // 日志记录
use tauri::State;           // Tauri 状态管理

use crate::models::*;       // 数据模型
use crate::AppState;        // 应用状态

/// 获取所有 License 列表
/// @param state - Tauri 应用状态（包含数据库连接）
/// @returns License 列表
#[tauri::command]
pub async fn get_licenses(state: State<'_, AppState>) -> Result<Vec<EnumLicense>, String> {
    debug!("Getting all licenses");
    let db = state.db.lock().map_err(|e| e.to_string())?; // 获取数据库锁
    let result = db.get_all_licenses().map_err(|e| e.to_string())?;
    info!("Got {} licenses", result.len());
    Ok(result)
}

/// 从 SPDX 同步 License 数据
/// 从 GitHub 上的 SPDX License List Data 仓库获取最新的许可证列表并存入数据库
/// @param state - Tauri 应用状态
/// @returns 同步的 License 数量
#[tauri::command]
pub async fn sync_licenses_from_spdx(state: State<'_, AppState>) -> Result<usize, String> {
    info!("Syncing licenses from SPDX");
    let client = reqwest::Client::new();
    // 从 GitHub 获取 SPDX License 列表 JSON 数据
    let resp = client
        .get("https://raw.githubusercontent.com/spdx/license-list-data/main/json/licenses.json")
        .send()
        .await
        .map_err(|e| format!("SPDX fetch failed: {}", e))?;
    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let licenses = data["licenses"]
        .as_array()
        .ok_or_else(|| "Invalid SPDX data".to_string())?;

    // 解析 SPDX JSON 数据为 EnumLicense 结构体
    let mut enum_licenses = Vec::new();
    for lic in licenses {
        let spdx_id = lic["licenseId"].as_str().unwrap_or("");
        let full_name = lic["name"].as_str().unwrap_or("");
        let url = lic["seeAlso"].as_array()
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

    // 清空旧数据并插入新数据
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_all_licenses().map_err(|e| e.to_string())?;
    let count = enum_licenses.len();
    for lic in &enum_licenses {
        let _ = db.upsert_license(lic); // 逐条插入，忽略错误
    }
    info!("Synced {} licenses from SPDX", count);
    Ok(count)
}

/// 添加新的 License
/// @param state - Tauri 应用状态
/// @param spdx_id - SPDX 标准 ID
/// @param full_name - License 完整名称
/// @param url - 可选的 License URL
/// @param description - 可选的描述
/// @param category - 可选的分类
/// @returns 新插入记录的 ID
#[tauri::command]
pub async fn add_license(
    state: State<'_, AppState>,
    spdx_id: String,
    full_name: String,
    url: Option<String>,
    description: Option<String>,
    category: Option<String>,
) -> Result<i64, String> {
    info!("Adding license: {} ({})", spdx_id, full_name);
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
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.upsert_license(&lic).map_err(|e| e.to_string())
}

/// 获取所有编程语言列表
/// @param state - Tauri 应用状态
/// @returns 编程语言列表
#[tauri::command]
pub async fn get_languages(state: State<'_, AppState>) -> Result<Vec<EnumProgrammingLanguage>, String> {
    debug!("Getting all languages");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.get_all_languages().map_err(|e| e.to_string())?;
    info!("Got {} languages", result.len());
    Ok(result)
}

/// 添加或更新编程语言
/// @param state - Tauri 应用状态
/// @param language - 编程语言信息
/// @returns 新插入或更新的记录 ID
#[tauri::command]
pub async fn upsert_language(state: State<'_, AppState>, language: EnumProgrammingLanguage) -> Result<i64, String> {
    info!("Upserting language: {}", language.name);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.upsert_language(&language).map_err(|e| e.to_string())
}

/// 删除编程语言
/// @param state - Tauri 应用状态
/// @param name - 要删除的编程语言名称
#[tauri::command]
pub async fn delete_language(state: State<'_, AppState>, name: String) -> Result<(), String> {
    info!("Deleting language: {}", name);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_language(&name).map_err(|e| e.to_string())
}
