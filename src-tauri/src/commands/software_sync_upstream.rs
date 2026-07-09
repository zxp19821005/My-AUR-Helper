use log::{info, error};
use tauri::State;

use crate::commands::proxy_utils::{build_client, get_active_proxy};
use crate::commands::software_sync_utils::{
    get_setting_opt, parse_u64, parse_u32, build_checker_settings, UpstreamCheckResult,
};
use crate::errors::AppResult;
use crate::AppState;

/// 并行检查上游版本
#[tauri::command]
pub async fn check_all_upstream(state: State<'_, AppState>) -> AppResult<Vec<(String, String)>> {
    info!("正在检查所有软件包的上游版本");
    let (packages, settings, timeout, retry, proxy_url) = {
        let db = state.db.lock()?;
        let packages = db.get_all_software()?;
        let settings = build_checker_settings(&db);
        let timeout = parse_u64(&get_setting_opt(&db, "http_timeout").unwrap_or_default(), 30);
        let retry = parse_u32(&get_setting_opt(&db, "http_retry_count").unwrap_or_default(), 2);
        let proxy_url = get_active_proxy(&db);
        (packages, settings, timeout, retry, proxy_url)
    };
    let client = build_client(timeout, proxy_url.as_deref());

    // 并行检查所有包
    let mut handles = Vec::new();
    for sw in &packages {
        let client = client.clone();
        let settings = settings.clone();
        let pkgname = sw.pkgname.clone();
        let upstream_url = sw.upstream_url.clone().unwrap_or_default();
        let version_extract_regex = sw.version_extract_regex.clone();
        let check_test_versions = sw.check_test_versions;
        let check_binary_files = sw.check_binary_files;
        let checker_type_id = sw.checker_type_id.clone();
        let software_id = sw.software_id.unwrap_or(0);
        let retry = retry;

        let handle = tokio::spawn(async move {
            let checker = crate::checkers::get_checker(&checker_type_id, settings);
            let options = crate::checkers::CheckOptions {
                check_test_versions,
                check_binary_files,
            };

            let result = check_with_retry(
                &*checker, &client, &upstream_url, &pkgname, version_extract_regex.as_deref(), &options, retry,
            ).await;

            (pkgname, software_id, result)
        });
        handles.push(handle);
    }

    // 收集结果
    let mut check_results: Vec<UpstreamCheckResult> = Vec::new();
    for handle in handles {
        if let Ok((pkgname, software_id, result)) = handle.await {
            match result {
                Ok(Some(version)) => {
                    let aur_ver = {
                        let db = state.db.lock()?;
                        db.get_aur_info(software_id).ok().flatten()
                            .and_then(|a| a.aur_version)
                            .filter(|v| !v.is_empty())
                    };

                    let is_outdated = match aur_ver.as_deref() {
                        Some(aur) => crate::versions::compare_versions(aur, &version) == crate::versions::VersionComparison::LessThan,
                        None => true,
                    };

                    check_results.push(UpstreamCheckResult {
                        pkgname,
                        software_id,
                        upstream_version: version,
                        is_outdated,
                    });
                }
                _ => {
                    check_results.push(UpstreamCheckResult {
                        pkgname,
                        software_id,
                        upstream_version: String::new(),
                        is_outdated: false,
                    });
                }
            }
        }
    }

    // 批量写入数据库
    let db = state.db.lock()?;
    let mut success_results = Vec::new();
    for result in &check_results {
        if !result.upstream_version.is_empty() {
            let cleaned_version = result.upstream_version.strip_prefix('v').unwrap_or(&result.upstream_version);
            
            let _ = db.update_software_outdated(result.software_id, result.is_outdated);
            let upstream_info = crate::models::UpstreamInfo {
                software_id: result.software_id,
                upstream_version: Some(cleaned_version.to_string()),
                upstream_license_id: None,
                last_checked: Some(chrono::Utc::now().timestamp()),
            };
            let _ = db.upsert_upstream_info(&upstream_info);
            
            success_results.push((result.pkgname.clone(), result.upstream_version.clone()));
        } else {
            let _ = db.update_software_outdated(result.software_id, false);
        }
    }

    info!("已完成 {} 个软件包的上游版本检查", success_results.len());
    Ok(success_results)
}

async fn check_with_retry(
    checker: &dyn crate::checkers::VersionChecker,
    client: &reqwest::Client,
    upstream_url: &str,
    pkgname: &str,
    version_extract_regex: Option<&str>,
    options: &crate::checkers::CheckOptions,
    retry_count: u32,
) -> AppResult<Option<String>> {
    let mut last_error = None;
    for attempt in 0..=retry_count {
        if attempt > 0 {
            info!("[重试] 第 {} 次重试 {}", attempt, pkgname);
        }
        match checker
            .check(client, upstream_url, pkgname, version_extract_regex, options)
            .await
        {
            Ok(result) => return Ok(result),
            Err(e) => {
                error!("检查 {} 失败 (尝试 {}/{}): {}", pkgname, attempt + 1, retry_count + 1, e);
                last_error = Some(e);
            }
        }
    }
    Err(last_error.unwrap_or(crate::errors::AppError::VersionCheckError("检查失败".to_string())))
}