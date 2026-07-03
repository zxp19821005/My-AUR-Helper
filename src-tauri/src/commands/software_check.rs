/**
 * software_check.rs - 软件包上游版本检查相关命令
 */
use log::{debug, error, info};
use tauri::State;

use crate::checkers;
use crate::models::*;
use crate::AppState;

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
