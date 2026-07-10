use chrono::Utc;
use log::{debug, error, info};
use tauri::State;

use crate::checkers::{self, CheckOptions, CheckResult, CheckerSettings};
use crate::commands::proxy_utils::{build_client, get_active_proxy};
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

async fn check_with_retry(
    checker: &dyn checkers::VersionChecker,
    client: &reqwest::Client,
    upstream_url: &str,
    pkgname: &str,
    version_extract_regex: Option<&str>,
    options: &CheckOptions,
    retry_count: u32,
) -> AppResult<CheckResult> {
    let mut last_error = None;
    for attempt in 0..=retry_count {
        if attempt > 0 {
            info!("[重试] 第 {} 次重试 {}", attempt, pkgname);
        }
        match checker
            .check(
                client,
                upstream_url,
                pkgname,
                version_extract_regex,
                options,
            )
            .await
        {
            Ok(result) => return Ok(result),
            Err(e) => {
                error!(
                    "检查 {} 失败 (尝试 {}/{}): {}",
                    pkgname,
                    attempt + 1,
                    retry_count + 1,
                    e
                );
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

fn compare_and_update(
    db: &crate::db::Database,
    software_id: i64,
    pkgname: &str,
    version: &str,
    license_spdx_id: Option<&str>,
    language_names: &[String],
) -> AppResult<()> {
    let aur_ver = db
        .get_aur_info(software_id)?
        .map(|a| a.aur_version.unwrap_or_default());

    // 清理版本号前缀（移除 v 前缀）用于存储
    let cleaned_version = version.strip_prefix('v').unwrap_or(version);

    let is_outdated = match aur_ver.as_deref() {
        Some(aur) => {
            versions::compare_versions(aur, version) == versions::VersionComparison::LessThan
        }
        None => true,
    };

    info!(
        "[版本检查结果] {}: AUR={:?} 上游={} 需更新={}",
        pkgname, aur_ver, version, is_outdated
    );

    // 获取 license ID
    let upstream_license_id = db.get_or_create_license_id(license_spdx_id)?;

    // 解析语言 ID 列表（如果语言不存在则自动创建）
    let language_ids = db.resolve_language_ids(language_names)?;
    info!(
        "[版本检查结果] {}: languages={:?} -> ids={:?}",
        pkgname, language_names, language_ids
    );

    db.update_software_outdated(software_id, is_outdated)?;
    
    // 更新软件的语言 ID 列表
    db.update_software_languages(software_id, &language_ids)?;
    
    let upstream_info = UpstreamInfo {
        software_id,
        upstream_version: Some(cleaned_version.to_string()),
        upstream_license_id,
        last_checked: Some(Utc::now().timestamp()),
    };
    db.upsert_upstream_info(&upstream_info)?;
    Ok(())
}

#[tauri::command]
pub async fn check_upstream_version(
    state: State<'_, AppState>,
    pkgname: String,
) -> AppResult<String> {
    info!("正在检查上游版本: {}", pkgname);
    let (sw, settings, timeout, retry, proxy_url) = {
        let db = state.db.lock()?;
        let sw = db
            .get_software_by_name(&pkgname)?
            .ok_or_else(|| AppError::PackageNotFound(pkgname.clone()))?;
        let settings = build_checker_settings(&db);
        let timeout = parse_u64(
            &get_setting_opt(&db, "http_timeout").unwrap_or_default(),
            30,
        );
        let retry = parse_u32(
            &get_setting_opt(&db, "http_retry_count").unwrap_or_default(),
            2,
        );
        let proxy_url = get_active_proxy(&db);
        (sw, settings, timeout, retry, proxy_url)
    };
    let has_aur_version = {
        let db = state.db.lock()?;
        db.get_aur_info(sw.software_id.unwrap_or(0))?
            .and_then(|a| a.aur_version)
            .filter(|v| !v.is_empty())
            .is_some()
    };
    if !has_aur_version {
        return Err(AppError::VersionCheckError(format!(
            "请先获取 {} 的 AUR 信息",
            pkgname
        )));
    }

    let client = build_client(timeout, proxy_url.as_deref());
    let checker = checkers::get_checker(&sw.checker_type_id, settings);
    let upstream_url = sw.upstream_url.as_deref().unwrap_or("");
    let version_extract_regex = sw.version_extract_regex.as_deref();
    let options = CheckOptions {
        check_test_versions: sw.check_test_versions,
        check_binary_files: sw.check_binary_files,
    };

    debug!("使用检查器: {} 检查 {}", checker.name(), pkgname);
    let check_result = match check_with_retry(
        &*checker,
        &client,
        upstream_url,
        &sw.pkgname,
        version_extract_regex,
        &options,
        retry,
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            return Err(AppError::VersionCheckError(format!("检查失败: {}", e)));
        }
    };

    let version = check_result
        .version
        .ok_or_else(|| AppError::VersionCheckError(format!("无法确定 {} 的上游版本", pkgname)))?;

    let db = state.db.lock()?;
    compare_and_update(
        &db,
        sw.software_id.unwrap_or(0),
        &sw.pkgname,
        &version,
        check_result.license.as_deref(),
        &check_result.language_names,
    )?;
    Ok(version)
}

// check_all_upstream 已移至 software_sync.rs 实现并行检查

#[tauri::command]
pub async fn check_selected_upstream(
    state: State<'_, AppState>,
    pkgname_list: Vec<String>,
) -> AppResult<Vec<(String, String)>> {
    info!("正在检查 {} 个软件包的上游版本", pkgname_list.len());
    let (settings, timeout, retry, proxy_url) = {
        let db = state.db.lock()?;
        let settings = build_checker_settings(&db);
        let timeout = parse_u64(
            &get_setting_opt(&db, "http_timeout").unwrap_or_default(),
            30,
        );
        let retry = parse_u32(
            &get_setting_opt(&db, "http_retry_count").unwrap_or_default(),
            2,
        );
        let proxy_url = get_active_proxy(&db);
        (settings, timeout, retry, proxy_url)
    };
    let client = build_client(timeout, proxy_url.as_deref());
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
        let options = CheckOptions {
            check_test_versions: sw.check_test_versions,
            check_binary_files: sw.check_binary_files,
        };

        if let Ok(check_result) = check_with_retry(
            &*checker,
            &client,
            upstream_url,
            &sw.pkgname,
            version_extract_regex,
            &options,
            retry,
        )
        .await
        {
            if let Some(version) = check_result.version {
                let db = state.db.lock()?;
                let _ = compare_and_update(
                    &db,
                    sw.software_id.unwrap_or(0),
                    &sw.pkgname,
                    &version,
                    check_result.license.as_deref(),
                    &check_result.language_names,
                );
                results.push((sw.pkgname.clone(), version));
            }
        }
    }
    Ok(results)
}