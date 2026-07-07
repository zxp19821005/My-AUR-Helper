/**
 * software_check.rs - 软件包上游版本检查相关命令
 */
use log::{debug, error, info};
use tauri::State;

use crate::checkers;
use crate::errors::{AppError, AppResult};
use crate::models::*;
use crate::versions;
use crate::AppState;

/// 检查单个软件包的上游版本
#[tauri::command]
pub async fn check_upstream_version(
    state: State<'_, AppState>,
    pkgname: String,
) -> AppResult<String> {
    info!("正在检查上游版本: {}", pkgname);
    let client = reqwest::Client::new();
    let sw = {
        let db = state.db.lock()?;
        db.get_software_by_name(&pkgname)?
            .ok_or_else(|| AppError::PackageNotFound(pkgname.clone()))?
    };
    let checker = checkers::get_checker(&sw.checker_type_id);
    let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
    let version_extract_regex = sw.version_extract_regex.as_deref();
    debug!("使用检查器: {} 检查 {}", checker.name(), pkgname);
    match checker.check(&client, upstream_url, &sw.pkgname, version_extract_regex).await {
        Ok(Some(version)) => {
            let db = state.db.lock()?;
            let aur_ver = db
                .get_aur_info(sw.software_id.unwrap_or(0))?
                .map(|a| a.aur_version.unwrap_or_default());
            
            debug!("[版本比较] 准备比较版本:");
            debug!("[版本比较]   AUR版本: {}", aur_ver.as_deref().unwrap_or("(未获取)"));
            debug!("[版本比较]   上游版本: {}", version);
            
            let is_outdated = match aur_ver.as_deref() {
                Some(aur) => {
                    let comparison = versions::compare_versions(aur, &version);
                    debug!("[版本比较]   比较结果: {:?}", comparison);
                    debug!("[版本比较]   算法: 使用 vercmp 算法，先标准化版本再比较");
                    comparison == versions::VersionComparison::LessThan
                }
                None => {
                    debug!("[版本比较]   未获取到 AUR 版本，标记为需更新");
                    true
                }
            };
            
            info!("[版本检查结果] 软件包: {}", pkgname);
            info!("[版本检查结果]   AUR版本: {}", aur_ver.as_deref().unwrap_or("(未获取)"));
            info!("[版本检查结果]   上游版本: {}", version);
            info!("[版本检查结果]   是否需要更新: {}", if is_outdated { "是" } else { "否" });
            
            db.update_software_outdated(sw.software_id.unwrap_or(0), is_outdated)?;
            let upstream_info = UpstreamInfo {
                software_id: sw.software_id.unwrap_or(0),
                upstream_url: sw.upstream_url.clone(),
                upstream_version: Some(version.clone()),
                upstream_license: None,
                last_checked: None,
            };
            db.upsert_upstream_info(&upstream_info)?;
            Ok(version)
        }
        Ok(None) => {
            error!("无法确定 {} 的上游版本", pkgname);
            Err(AppError::VersionCheckError(format!(
                "无法确定 {} 的上游版本",
                pkgname
            )))
        }
        Err(e) => {
            error!("版本检查失败 {}: {}", pkgname, e);
            Err(AppError::VersionCheckError(format!("检查失败: {}", e)))
        }
    }
}

/// 检查所有软件包的上游版本
#[tauri::command]
pub async fn check_all_upstream(state: State<'_, AppState>) -> AppResult<Vec<(String, String)>> {
    info!("正在检查所有软件包的上游版本");
    let client = reqwest::Client::new();
    let packages = {
        let db = state.db.lock()?;
        db.get_all_software()?
    };
    let mut results = Vec::new();
    for sw in &packages {
        let checker = checkers::get_checker(&sw.checker_type_id);
        let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
        let version_extract_regex = sw.version_extract_regex.as_deref();
        match checker.check(&client, upstream_url, &sw.pkgname, version_extract_regex).await {
            Ok(Some(version)) => {
                let db = state.db.lock()?;
                let aur_ver = db
                    .get_aur_info(sw.software_id.unwrap_or(0))?
                    .map(|a| a.aur_version.unwrap_or_default());
                
                debug!("[版本比较] 准备比较版本:");
                debug!("[版本比较]   AUR版本: {}", aur_ver.as_deref().unwrap_or("(未获取)"));
                debug!("[版本比较]   上游版本: {}", version);
                
                let is_outdated = match aur_ver.as_deref() {
                    Some(aur) => {
                        let comparison = versions::compare_versions(aur, &version);
                        debug!("[版本比较]   比较结果: {:?}", comparison);
                        debug!("[版本比较]   算法: 使用 vercmp 算法，先标准化版本再比较");
                        comparison == versions::VersionComparison::LessThan
                    }
                    None => {
                        debug!("[版本比较]   未获取到 AUR 版本，标记为需更新");
                        true
                    }
                };
                
                info!("[版本检查结果] 软件包: {}", sw.pkgname);
                info!("[版本检查结果]   AUR版本: {}", aur_ver.as_deref().unwrap_or("(未获取)"));
                info!("[版本检查结果]   上游版本: {}", version);
                info!("[版本检查结果]   是否需要更新: {}", if is_outdated { "是" } else { "否" });
                
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
            _ => {
                let db = state.db.lock()?;
                let _ = db.update_software_outdated(sw.software_id.unwrap_or(0), false);
            }
        }
    }
    info!("已完成 {} 个软件包的上游版本检查", results.len());
    Ok(results)
}

/// 检查指定软件包的上游版本（选中或全部）
#[tauri::command]
pub async fn check_selected_upstream(
    state: State<'_, AppState>,
    pkgname_list: Vec<String>,
) -> AppResult<Vec<(String, String)>> {
    info!("正在检查 {} 个软件包的上游版本", pkgname_list.len());
    let client = reqwest::Client::new();
    let mut results = Vec::new();
    for pkgname in &pkgname_list {
        let sw = {
            let db = state.db.lock()?;
            db.get_software_by_name(pkgname)?
                .ok_or_else(|| AppError::PackageNotFound(pkgname.clone()))?
        };
        let checker = checkers::get_checker(&sw.checker_type_id);
        let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
        let version_extract_regex = sw.version_extract_regex.as_deref();
        match checker.check(&client, upstream_url, &sw.pkgname, version_extract_regex).await {
            Ok(Some(version)) => {
                let db = state.db.lock()?;
                let aur_ver = db
                    .get_aur_info(sw.software_id.unwrap_or(0))?
                    .map(|a| a.aur_version.unwrap_or_default());
                
                debug!("[版本比较] 准备比较版本:");
                debug!("[版本比较]   AUR版本: {}", aur_ver.as_deref().unwrap_or("(未获取)"));
                debug!("[版本比较]   上游版本: {}", version);
                
                let is_outdated = match aur_ver.as_deref() {
                    Some(aur) => {
                        let comparison = versions::compare_versions(aur, &version);
                        debug!("[版本比较]   比较结果: {:?}", comparison);
                        debug!("[版本比较]   算法: 使用 vercmp 算法，先标准化版本再比较");
                        comparison == versions::VersionComparison::LessThan
                    }
                    None => {
                        debug!("[版本比较]   未获取到 AUR 版本，标记为需更新");
                        true
                    }
                };
                
                info!("[版本检查结果] 软件包: {}", sw.pkgname);
                info!("[版本检查结果]   AUR版本: {}", aur_ver.as_deref().unwrap_or("(未获取)"));
                info!("[版本检查结果]   上游版本: {}", version);
                info!("[版本检查结果]   是否需要更新: {}", if is_outdated { "是" } else { "否" });
                
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
