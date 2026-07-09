use chrono::Utc;
use log::{debug, error, info};
use std::time::Duration;
use tauri::State;

use crate::checkers::{self, CheckerSettings};
use crate::errors::{AppError, AppResult};
use crate::models::*;
use crate::versions;
use crate::AppState;

fn get_setting_opt(db: &crate::db::Database, key: &str) -> Option<String> {
    db.get_setting(key)
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
}

fn build_checker_settings(db: &crate::db::Database) -> CheckerSettings {
    CheckerSettings {
        github_token: get_setting_opt(db, "github_token"),
        gitee_token: get_setting_opt(db, "gitee_token"),
        gitlab_token: get_setting_opt(db, "gitlab_token"),
    }
}

fn build_client(timeout_secs: u64) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .unwrap_or_default()
}

async fn check_with_retry(
    checker: &dyn checkers::VersionChecker,
    client: &reqwest::Client,
    upstream_url: &str,
    pkgname: &str,
    version_extract_regex: Option<&str>,
    retry_count: u32,
) -> AppResult<Option<String>> {
    let mut last_error = None;
    for attempt in 0..=retry_count {
        if attempt > 0 {
            info!("[重试] 第 {} 次重试 {}", attempt, pkgname);
        }
        match checker
            .check(client, upstream_url, pkgname, version_extract_regex)
            .await
        {
            Ok(result) => return Ok(result),
            Err(e) => {
                error!("检查 {} 失败 (尝试 {}/{}): {}", pkgname, attempt + 1, retry_count + 1, e);
                last_error = Some(e);
            }
        }
    }
    Err(last_error.unwrap_or(AppError::VersionCheckError("检查失败".to_string())))
}

fn parse_u64(val: &str, default: u64) -> u64 {
    val.parse().unwrap_or(default)
}

fn parse_u32(val: &str, default: u32) -> u32 {
    val.parse().unwrap_or(default)
}

#[tauri::command]
pub async fn check_upstream_version(
    state: State<'_, AppState>,
    pkgname: String,
) -> AppResult<String> {
    info!("正在检查上游版本: {}", pkgname);
    let (sw, settings, timeout, retry) = {
        let db = state.db.lock()?;
        let sw = db.get_software_by_name(&pkgname)?
            .ok_or_else(|| AppError::PackageNotFound(pkgname.clone()))?;
        let settings = build_checker_settings(&db);
        let timeout = parse_u64(&get_setting_opt(&db, "http_timeout").unwrap_or_default(), 30);
        let retry = parse_u32(&get_setting_opt(&db, "http_retry_count").unwrap_or_default(), 2);
        (sw, settings, timeout, retry)
    };
    let client = build_client(timeout);
    let checker = checkers::get_checker(&sw.checker_type_id, settings);
    let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
    let version_extract_regex = sw.version_extract_regex.as_deref();
    debug!("使用检查器: {} 检查 {}", checker.name(), pkgname);
    match check_with_retry(&*checker, &client, upstream_url, &sw.pkgname, version_extract_regex, retry).await {
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
                last_checked: Some(Utc::now().naive_utc()),
            };
            db.upsert_upstream_info(&upstream_info)?;
            Ok(version)
        }
        Ok(None) => {
            error!("无法确定 {} 的上游版本", pkgname);
            Err(AppError::VersionCheckError(format!("无法确定 {} 的上游版本", pkgname)))
        }
        Err(e) => {
            error!("版本检查失败 {}: {}", pkgname, e);
            Err(AppError::VersionCheckError(format!("检查失败: {}", e)))
        }
    }
}

#[tauri::command]
pub async fn check_all_upstream(state: State<'_, AppState>) -> AppResult<Vec<(String, String)>> {
    info!("正在检查所有软件包的上游版本");
    let (packages, settings, timeout, retry) = {
        let db = state.db.lock()?;
        let packages = db.get_all_software()?;
        let settings = build_checker_settings(&db);
        let timeout = parse_u64(&get_setting_opt(&db, "http_timeout").unwrap_or_default(), 30);
        let retry = parse_u32(&get_setting_opt(&db, "http_retry_count").unwrap_or_default(), 2);
        (packages, settings, timeout, retry)
    };
    let client = build_client(timeout);
    let mut results = Vec::new();
    for sw in &packages {
        let checker = checkers::get_checker(&sw.checker_type_id, settings.clone());
        let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
        let version_extract_regex = sw.version_extract_regex.as_deref();
        match check_with_retry(&*checker, &client, upstream_url, &sw.pkgname, version_extract_regex, retry).await {
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
                    last_checked: Some(Utc::now().naive_utc()),
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

#[tauri::command]
pub async fn check_selected_upstream(
    state: State<'_, AppState>,
    pkgname_list: Vec<String>,
) -> AppResult<Vec<(String, String)>> {
    info!("正在检查 {} 个软件包的上游版本", pkgname_list.len());
    let (settings, timeout, retry) = {
        let db = state.db.lock()?;
        let settings = build_checker_settings(&db);
        let timeout = parse_u64(&get_setting_opt(&db, "http_timeout").unwrap_or_default(), 30);
        let retry = parse_u32(&get_setting_opt(&db, "http_retry_count").unwrap_or_default(), 2);
        (settings, timeout, retry)
    };
    let client = build_client(timeout);
    let mut results = Vec::new();
    for pkgname in &pkgname_list {
        let sw = {
            let db = state.db.lock()?;
            db.get_software_by_name(pkgname)?
                .ok_or_else(|| AppError::PackageNotFound(pkgname.clone()))?
        };
        let checker = checkers::get_checker(&sw.checker_type_id, settings.clone());
        let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
        let version_extract_regex = sw.version_extract_regex.as_deref();
        match check_with_retry(&*checker, &client, upstream_url, &sw.pkgname, version_extract_regex, retry).await {
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
                    last_checked: Some(Utc::now().naive_utc()),
                };
                let _ = db.upsert_upstream_info(&upstream_info);
                results.push((sw.pkgname.clone(), version));
            }
            _ => {}
        }
    }
    Ok(results)
}
