/**
 * software.rs - 软件包 CRUD 和设置命令
 */
use log::{debug, info};
use tauri::State;

use crate::models::*;
use crate::AppState;

/// 获取所有软件包列表
#[tauri::command]
pub async fn list_software(state: State<'_, AppState>) -> Result<Vec<SoftwareInfo>, String> {
    debug!("Listing all software");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.get_all_software().map_err(|e| e.to_string())?;
    info!("Listed {} software entries", result.len());
    Ok(result)
}

/// 获取软件包列表展示数据（含 AUR + Upstream 信息）
#[tauri::command]
pub async fn list_software_view(state: State<'_, AppState>) -> Result<Vec<SoftwareListEntry>, String> {
    debug!("Listing software view");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.get_software_list_entries().map_err(|e| e.to_string())?;
    info!("Listed {} software view entries", result.len());
    Ok(result)
}

/// 根据包名获取单个软件包信息
#[tauri::command]
pub async fn get_software(state: State<'_, AppState>, pkgname: String) -> Result<Option<SoftwareInfo>, String> {
    debug!("Getting software: {}", pkgname);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_software_by_name(&pkgname).map_err(|e| e.to_string())
}

/// 搜索软件包
#[tauri::command]
pub async fn search_software(state: State<'_, AppState>, keyword: String) -> Result<Vec<SoftwareInfo>, String> {
    debug!("Searching software: {}", keyword);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.search_software(&keyword).map_err(|e| e.to_string())?;
    info!("Search for '{}' found {} results", keyword, result.len());
    Ok(result)
}

/// 添加新的软件包
#[tauri::command]
pub async fn add_software(
    state: State<'_, AppState>,
    pkgname: String,
    upstream_url: Option<String>,
    package_type: i32,
    checker_type: i32,
    check_test_versions: bool,
    check_binary_files: bool,
    auto_check_enabled: bool,
    license_id: Option<i64>,
    language_id: Option<i64>,
) -> Result<i64, String> {
    info!("Adding software: {}", pkgname);
    let sw = SoftwareInfo {
        software_id: None,
        pkgname: pkgname.clone(),
        upstream_url,
        package_type_id: PackageType::from_id(package_type),
        checker_type_id: CheckerType::from_id(checker_type),
        is_outdated: false,
        check_test_versions,
        check_binary_files,
        auto_check_enabled,
        license_id,
        language_id,
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = db.insert_software(&sw).map_err(|e| e.to_string())?;
    info!("Added software '{}' with id {}", pkgname, id);
    Ok(id)
}

/// 更新软件包信息（编辑）
#[tauri::command]
pub async fn update_software(
    state: State<'_, AppState>,
    software_id: i64,
    pkgname: String,
    upstream_url: Option<String>,
    package_type: i32,
    checker_type: i32,
    is_outdated: bool,
    check_test_versions: bool,
    check_binary_files: bool,
    auto_check_enabled: bool,
    license_id: Option<i64>,
    language_id: Option<i64>,
) -> Result<(), String> {
    info!("Updating software {}: {}", software_id, pkgname);
    let sw = SoftwareInfo {
        software_id: Some(software_id),
        pkgname: pkgname.clone(),
        upstream_url,
        package_type_id: PackageType::from_id(package_type),
        checker_type_id: CheckerType::from_id(checker_type),
        is_outdated,
        check_test_versions,
        check_binary_files,
        auto_check_enabled,
        license_id,
        language_id,
    };
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.upsert_software(&sw).map_err(|e| e.to_string())
}

/// 设置软件包的 License
#[tauri::command]
pub async fn set_software_license(
    state: State<'_, AppState>,
    software_id: i64,
    license_id: Option<i64>,
) -> Result<(), String> {
    info!("Setting license for software {} to {:?}", software_id, license_id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_software_license(software_id, license_id)
        .map_err(|e| e.to_string())
}

/// 设置软件包的编程语言
#[tauri::command]
pub async fn set_software_language(
    state: State<'_, AppState>,
    software_id: i64,
    language_id: Option<i64>,
) -> Result<(), String> {
    info!("Setting language for software {} to {:?}", software_id, language_id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_software_language(software_id, language_id)
        .map_err(|e| e.to_string())
}

/// 删除单个软件包
#[tauri::command]
pub async fn delete_software(state: State<'_, AppState>, software_id: i64) -> Result<(), String> {
    info!("Deleting software with id {}", software_id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_software(software_id).map_err(|e| e.to_string())
}

/// 批量删除软件包
#[tauri::command]
pub async fn batch_delete_software(state: State<'_, AppState>, ids: Vec<i64>) -> Result<i64, String> {
    info!("Batch deleting {} software packages", ids.len());
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut count = 0i64;
    for id in &ids {
        if db.delete_software(*id).is_ok() {
            count += 1;
        }
    }
    info!("Deleted {} software packages", count);
    Ok(count)
}
