/**
 * software.rs - 软件包管理命令
 *
 * 提供软件包的增删改查和上游版本检查功能
 */
use log::{info, debug, error};  // 日志记录
use tauri::State;               // Tauri 状态管理

use crate::checkers;            // 版本检查器模块
use crate::models::*;           // 数据模型
use crate::AppState;            // 应用状态

/// 获取所有软件包列表
/// @param state - Tauri 应用状态
/// @returns 所有软件包信息列表
#[tauri::command]
pub async fn list_software(state: State<'_, AppState>) -> Result<Vec<SoftwareInfo>, String> {
    debug!("Listing all software");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.get_all_software().map_err(|e| e.to_string())?;
    info!("Listed {} software entries", result.len());
    Ok(result)
}

/// 根据包名获取单个软件包信息
/// @param state - Tauri 应用状态
/// @param pkgname - 包名
/// @returns 可选的软件包信息
#[tauri::command]
pub async fn get_software(state: State<'_, AppState>, pkgname: String) -> Result<Option<SoftwareInfo>, String> {
    debug!("Getting software: {}", pkgname);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_software_by_name(&pkgname).map_err(|e| e.to_string())
}

/// 搜索软件包（按包名或上游 URL 模糊匹配）
/// @param state - Tauri 应用状态
/// @param keyword - 搜索关键词
/// @returns 匹配的软件包列表
#[tauri::command]
pub async fn search_software(state: State<'_, AppState>, keyword: String) -> Result<Vec<SoftwareInfo>, String> {
    debug!("Searching software: {}", keyword);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.search_software(&keyword).map_err(|e| e.to_string())?;
    info!("Search for '{}' found {} results", keyword, result.len());
    Ok(result)
}

/// 添加新的软件包
/// @param state - Tauri 应用状态
/// @param pkgname - 包名
/// @param upstream_url - 可选的上游项目地址
/// @param package_type - 包类型 ID
/// @param checker_type - 检查器类型 ID
/// @param check_test_versions - 是否检查测试版本
/// @param check_binary_files - 是否检查二进制文件
/// @param auto_check_enabled - 是否启用自动检查
/// @param license_id - 可选的 License ID
/// @param language_id - 可选的编程语言 ID
/// @returns 新插入的软件包 ID
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

/// 设置软件包的 License
/// @param state - Tauri 应用状态
/// @param software_id - 软件包 ID
/// @param license_id - 可选的 License ID（设为 None 可清除）
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
/// @param state - Tauri 应用状态
/// @param software_id - 软件包 ID
/// @param language_id - 可选的编程语言 ID（设为 None 可清除）
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

/// 删除软件包
/// @param state - Tauri 应用状态
/// @param software_id - 要删除的软件包 ID
#[tauri::command]
pub async fn delete_software(state: State<'_, AppState>, software_id: i64) -> Result<(), String> {
    info!("Deleting software with id {}", software_id);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_software(software_id).map_err(|e| e.to_string())
}

/// 检查单个软件包的上游版本
/// 根据包配置的检查器类型，查询上游版本号并与本地记录比较
/// @param state - Tauri 应用状态
/// @param pkgname - 包名
/// @returns 检测到的上游版本号
#[tauri::command]
pub async fn check_upstream_version(
    state: State<'_, AppState>,
    pkgname: String,
) -> Result<String, String> {
    info!("Checking upstream version for: {}", pkgname);
    let client = reqwest::Client::new();
    // 从数据库获取包信息
    let sw = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_software_by_name(&pkgname)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Package not found".to_string())?
    };
    // 获取对应的版本检查器
    let checker = checkers::get_checker(&sw.checker_type_id);
    let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
    debug!("Using checker: {} for {}", checker.name(), pkgname);
    // 执行版本检查
    match checker.check(&client, upstream_url, &sw.pkgname).await {
        Ok(Some(version)) => {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            // 获取当前记录的上游版本
            let current_ver = db.get_upstream_info(sw.software_id.unwrap_or(0))
                .map_err(|e| e.to_string())?
                .map(|u| u.upstream_version.unwrap_or_default());
            // 比较版本号判断是否过期
            let is_outdated = current_ver.as_deref() != Some(&version);
            // 更新过期状态
            db.update_software_outdated(sw.software_id.unwrap_or(0), is_outdated)
                .map_err(|e| e.to_string())?;
            // 更新上游版本信息
            let upstream_info = UpstreamInfo {
                software_id: sw.software_id.unwrap_or(0),
                upstream_url: sw.upstream_url.clone(),
                upstream_version: Some(version.clone()),
                upstream_license: None,
                last_checked: None,
            };
            db.upsert_upstream_info(&upstream_info).map_err(|e| e.to_string())?;
            info!("Checked {}: {} -> outdated={}", pkgname, version, is_outdated);
            Ok(version)
        }
        Ok(None) => {
            error!("Could not determine upstream version for {}", pkgname);
            Err("Could not determine upstream version".to_string())
        }
        Err(e) => {
            error!("Version check failed for {}: {}", pkgname, e);
            Err(format!("Check failed: {}", e))
        }
    }
}

/// 检查所有软件包的上游版本
/// 遍历所有软件包，逐一检查上游版本并更新状态
/// @param state - Tauri 应用状态
/// @returns 已检查的包名和新版本号列表
#[tauri::command]
pub async fn check_all_upstream(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    info!("Checking upstream versions for all software");
    let client = reqwest::Client::new();
    // 获取所有软件包列表
    let packages = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_all_software().map_err(|e| e.to_string())?
    };
    let mut results = Vec::new();
    // 逐个检查每个包的上游版本
    for sw in &packages {
        let checker = checkers::get_checker(&sw.checker_type_id);
        let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
        match checker.check(&client, upstream_url, &sw.pkgname).await {
            Ok(Some(version)) => {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                let current_ver = db.get_upstream_info(sw.software_id.unwrap_or(0))
                    .map_err(|e| e.to_string())?
                    .map(|u| u.upstream_version.unwrap_or_default());
                let is_outdated = current_ver.as_deref() != Some(&version);
                let _ = db.update_software_outdated(sw.software_id.unwrap_or(0), is_outdated);
                let upstream_info = UpstreamInfo {
                    software_id: sw.software_id.unwrap_or(0),
                    upstream_url: sw.upstream_url.clone(),
                    upstream_version: Some(version.clone()),
                    upstream_license: None,
                    last_checked: None,
                };
                let _ = db.upsert_upstream_info(&upstream_info);
                debug!("Checked {}: {} -> outdated={}", sw.pkgname, version, is_outdated);
                results.push((sw.pkgname.clone(), version));
            }
            _ => {
                // 检查失败时，标记为未过期（保守处理）
                let db = state.db.lock().map_err(|e| e.to_string())?;
                let _ = db.update_software_outdated(sw.software_id.unwrap_or(0), false);
            }
        }
    }
    info!("Completed upstream check for {} packages", results.len());
    Ok(results)
}
