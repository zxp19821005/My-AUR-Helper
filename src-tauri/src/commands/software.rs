/**
 * software.rs - 软件包管理命令
 */
use log::{info, debug, error};
use tauri::State;

use crate::aur;
use crate::checkers;
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

/// 从 AUR 同步软件包（按用户名）
#[tauri::command]
pub async fn sync_from_aur(state: State<'_, AppState>) -> Result<i64, String> {
    info!("Syncing packages from AUR");
    let username = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_setting("aur_username")
            .map_err(|e| e.to_string())?
            .map(|s| s.value)
            .unwrap_or_default()
    };
    if username.is_empty() {
        return Err("AUR username not configured".to_string());
    }
    let client = reqwest::Client::new();
    let packages = aur::fetch_packages_by_user(&client, &username)
        .await
        .map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    for pkg in &packages {
        let sw = db
            .get_software_by_name(&pkg.pkgname)
            .map_err(|e| e.to_string())?;
        let software_id = if let Some(existing) = sw {
            existing.software_id.unwrap_or(0)
        } else {
            let new_sw = SoftwareInfo {
                software_id: None,
                pkgname: pkg.pkgname.clone(),
                upstream_url: pkg.url.clone(),
                package_type_id: PackageType::Compiled,
                checker_type_id: CheckerType::Manual,
                is_outdated: false,
                check_test_versions: false,
                check_binary_files: false,
                auto_check_enabled: true,
                license_id: None,
                language_id: None,
            };
            db.insert_software(&new_sw).map_err(|e| e.to_string())?
        };
        let aur_info = AurInfo {
            software_id,
            pkgdesc: pkg.pkgdesc.clone(),
            aur_version: pkg.version.clone(),
            license_id: None,
            last_updated: None,
            depends: pkg
                .depends
                .as_ref()
                .map(|d| serde_json::to_string(d).unwrap_or_default()),
            makedepends: pkg
                .makedepends
                .as_ref()
                .map(|d| serde_json::to_string(d).unwrap_or_default()),
            optdepends: pkg
                .optdepends
                .as_ref()
                .map(|d| serde_json::to_string(d).unwrap_or_default()),
            out_of_date: pkg.out_of_date,
        };
        let _ = db.upsert_aur_info(&aur_info);
    }
    info!("Synced {} packages from AUR", packages.len());
    Ok(packages.len() as i64)
}

/// 从 PKGBUILD 文件同步软件包
#[tauri::command]
pub async fn sync_from_pkgbuild(state: State<'_, AppState>) -> Result<i64, String> {
    info!("Syncing packages from PKGBUILD files");
    let aur_dir = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_setting("aur_files_dir")
            .map_err(|e| e.to_string())?
            .map(|s| s.value)
            .unwrap_or_default()
    };
    if aur_dir.is_empty() {
        return Err("AUR files directory not configured".to_string());
    }
    let path = std::path::Path::new(&aur_dir);
    let packages = aur::sync_from_local_files(path)
        .await
        .map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    for sw in &packages {
        let _ = db.upsert_software(sw);
    }
    info!("Synced {} packages from PKGBUILD files", packages.len());
    Ok(packages.len() as i64)
}

/// 更新指定软件包的 AUR 信息
#[tauri::command]
pub async fn update_aur_info(
    state: State<'_, AppState>,
    pkgname_list: Option<Vec<String>>,
) -> Result<i64, String> {
    info!("Updating AUR info for packages");
    let pkgnames: Vec<String> = if let Some(list) = pkgname_list {
        list
    } else {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_all_software()
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|s| s.pkgname)
            .collect()
    };
    let client = reqwest::Client::new();
    let mut count = 0i64;
    for pkgname in &pkgnames {
        if let Ok(Some(data)) = aur::get_package_info(&client, pkgname).await {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let sw = db.get_software_by_name(pkgname).map_err(|e| e.to_string())?;
            if let Some(existing) = sw {
                if let Some(sid) = existing.software_id {
                    let info = AurInfo {
                        software_id: sid,
                        pkgdesc: data["Description"].as_str().map(|s| s.to_string()),
                        aur_version: data["Version"].as_str().map(|s| s.to_string()),
                        license_id: None,
                        last_updated: data["LastModified"].as_i64(),
                        depends: data["Depends"]
                            .as_array()
                            .map(|a| serde_json::to_string(a).unwrap_or_default()),
                        makedepends: data["MakeDepends"]
                            .as_array()
                            .map(|a| serde_json::to_string(a).unwrap_or_default()),
                        optdepends: data["OptDepends"]
                            .as_array()
                            .map(|a| serde_json::to_string(a).unwrap_or_default()),
                        out_of_date: data["OutOfDate"].as_i64().map(|v| v != 0),
                    };
                    let _ = db.upsert_aur_info(&info);
                    count += 1;
                }
            }
        }
    }
    info!("Updated AUR info for {} packages", count);
    Ok(count)
}

/// 检查指定软件包的上游版本（选中或全部）
#[tauri::command]
pub async fn check_selected_upstream(
    state: State<'_, AppState>,
    pkgname_list: Vec<String>,
) -> Result<Vec<(String, String)>, String> {
    info!("Checking upstream versions for {} packages", pkgname_list.len());
    let client = reqwest::Client::new();
    let mut results = Vec::new();
    for pkgname in &pkgname_list {
        let sw = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            db.get_software_by_name(pkgname)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Package not found: {}", pkgname))?
        };
        let checker = checkers::get_checker(&sw.checker_type_id);
        let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
        match checker.check(&client, upstream_url, &sw.pkgname).await {
            Ok(Some(version)) => {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                let current_ver = db
                    .get_upstream_info(sw.software_id.unwrap_or(0))
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
                results.push((sw.pkgname.clone(), version));
            }
            _ => {}
        }
    }
    Ok(results)
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

/// 检查单个软件包的上游版本
#[tauri::command]
pub async fn check_upstream_version(
    state: State<'_, AppState>,
    pkgname: String,
) -> Result<String, String> {
    info!("Checking upstream version for: {}", pkgname);
    let client = reqwest::Client::new();
    let sw = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_software_by_name(&pkgname)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Package not found".to_string())?
    };
    let checker = checkers::get_checker(&sw.checker_type_id);
    let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
    debug!("Using checker: {} for {}", checker.name(), pkgname);
    match checker.check(&client, upstream_url, &sw.pkgname).await {
        Ok(Some(version)) => {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let current_ver = db
                .get_upstream_info(sw.software_id.unwrap_or(0))
                .map_err(|e| e.to_string())?
                .map(|u| u.upstream_version.unwrap_or_default());
            let is_outdated = current_ver.as_deref() != Some(&version);
            db.update_software_outdated(sw.software_id.unwrap_or(0), is_outdated)
                .map_err(|e| e.to_string())?;
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
#[tauri::command]
pub async fn check_all_upstream(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    info!("Checking upstream versions for all software");
    let client = reqwest::Client::new();
    let packages = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_all_software().map_err(|e| e.to_string())?
    };
    let mut results = Vec::new();
    for sw in &packages {
        let checker = checkers::get_checker(&sw.checker_type_id);
        let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
        match checker.check(&client, upstream_url, &sw.pkgname).await {
            Ok(Some(version)) => {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                let current_ver = db
                    .get_upstream_info(sw.software_id.unwrap_or(0))
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
                let db = state.db.lock().map_err(|e| e.to_string())?;
                let _ = db.update_software_outdated(sw.software_id.unwrap_or(0), false);
            }
        }
    }
    info!("Completed upstream check for {} packages", results.len());
    Ok(results)
}
