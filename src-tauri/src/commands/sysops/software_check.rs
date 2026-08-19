/**
 * software_check.rs - 版本检查命令
 *
 * 功能：
 * - check_upstream_version: 检查单个软件包的上游版本
 * - check_selected_upstream: 检查选中的软件包上游版本（并行检查 + 批量写入）
 */
use chrono::Utc;
use log::{debug, error, info};
use std::collections::HashMap;
use tauri::State;

use super::proxy_utils::{build_client, get_active_proxy};
use super::software_sync::batch::{batch_check_upstream, PackageTask};
use super::software_sync::utils::{
    build_checker_settings, get_setting_opt, parse_u32, parse_u64, UpstreamCheckResult,
};
use crate::checkers::{self, CheckOptions, CheckResult};
use crate::errors::{AppError, AppResult};
use crate::models::*;
use crate::versions;
use crate::AppState;

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
            // 指数退避延迟：1s, 2s, 4s ...
            let delay_secs = 1u64 << (attempt - 1);
            info!(
                "[重试] 第 {} 次重试 {} (等待 {}s)",
                attempt, pkgname, delay_secs
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
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
                // 永久性错误（DNS 失败、404、403 等）不重试
                if !e.is_retryable() {
                    debug!("错误不可重试，跳过剩余重试");
                    return Err(e);
                }
                last_error = Some(e);
            }
        }
    }
    Err(last_error.unwrap_or(AppError::VersionCheckError("检查失败".to_string())))
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

    // 获取 license JSON（直接存储数组）
    let upstream_license_id = license_spdx_id.map(|s| s.to_string());
    debug!(
        "[版本检查] {} license_spdx_id={:?} -> upstream_license_id={:?}",
        pkgname, license_spdx_id, upstream_license_id
    );

    // 解析语言 ID 列表（如果语言不存在则自动创建）
    let language_ids = db.resolve_language_ids(language_names)?;
    debug!(
        "[版本检查] {} languages={:?} -> ids={:?}",
        pkgname, language_names, language_ids
    );

    debug!(
        "[版本检查] {} 准备写入: software_id={}, is_outdated={}, version={}, license_id={:?}",
        pkgname, software_id, is_outdated, cleaned_version, upstream_license_id
    );

    db.update_software_outdated(software_id, is_outdated)?;
    debug!(
        "[版本检查] {} step1: update_software_outdated 成功",
        pkgname
    );

    // 只有当用户没有手动设置语言列表时，才用自动检测到的语言列表填充
    let existing_sw = db.get_software_by_name(pkgname)?;
    if let Some(ref sw) = existing_sw {
        if sw.language_ids.is_empty() && !language_ids.is_empty() {
            db.update_software_languages(software_id, &language_ids)?;
            debug!(
                "[版本检查] {} step2: update_software_languages 成功（语言列表为空，自动填充）",
                pkgname
            );
        } else {
            debug!(
                "[版本检查] {} step2: 跳过语言列表更新（已有用户设置: {:?}）",
                pkgname, sw.language_ids
            );
        }
    }

    let upstream_info = UpstreamInfo {
        software_id,
        upstream_version: Some(cleaned_version.to_string()),
        upstream_license_id,
        last_checked: Some(Utc::now().timestamp()),
        upstream_url_status: None,
    };
    db.upsert_upstream_info(&upstream_info)?;
    debug!("[版本检查] {} step3: upsert_upstream_info 成功", pkgname);
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

    // 一次性读取所有选中软件包的信息 + 配置，单次加锁
    let (packages, settings, timeout, retry, proxy_url) = {
        let db = state.db.lock()?;
        let mut packages = Vec::new();
        for pkgname in &pkgname_list {
            if let Some(sw) = db.get_software_by_name(pkgname)? {
                packages.push(sw);
            }
        }
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
        (packages, settings, timeout, retry, proxy_url)
    };

    // 预先提取每个包的已有语言 ID，避免写库阶段逐包回查（消除 N+1 查询）
    let lang_by_id: HashMap<i64, Vec<i64>> = packages
        .iter()
        .map(|p| (p.software_id.unwrap_or(0), p.language_ids.clone()))
        .collect();

    // 过滤掉没有 AUR 版本的包（前置检查，避免浪费网络请求）
    let mut tasks = Vec::new();
    {
        let db = state.db.lock()?;
        for sw in &packages {
            let has_aur = db
                .get_aur_info(sw.software_id.unwrap_or(0))?
                .and_then(|a| a.aur_version)
                .filter(|v| !v.is_empty())
                .is_some();
            if has_aur {
                tasks.push(PackageTask {
                    pkgname: sw.pkgname.clone(),
                    software_id: sw.software_id.unwrap_or(0),
                    upstream_url: sw.upstream_url.clone().unwrap_or_default(),
                    version_extract_regex: sw.version_extract_regex.clone(),
                    check_test_versions: sw.check_test_versions,
                    check_binary_files: sw.check_binary_files,
                    checker_type: sw.checker_type_id.clone(),
                    package_type: sw.package_type_id.clone(),
                });
            }
        }
    }

    if tasks.is_empty() {
        info!("[版本检查] 所有选中包均无 AUR 版本，跳过检查");
        return Ok(Vec::new());
    }

    let client = build_client(timeout, proxy_url.as_deref());

    // 并行检查：复用 batch_check_upstream 的分类并发引擎
    let outcome = batch_check_upstream(tasks, client, settings, retry).await;

    // 一次性批量读取所有 AUR 版本（单条 SQL + 单次加锁）
    let aur_map: HashMap<i64, String> = {
        let db = state.db.lock()?;
        db.get_aur_versions_map()?
    };

    // 在内存中比较版本，得出 is_outdated
    let mut check_results: Vec<UpstreamCheckResult> = Vec::new();
    for r in outcome.checked {
        if r.upstream_version.is_empty() {
            check_results.push(r);
            continue;
        }
        let aur_ver = aur_map
            .get(&r.software_id)
            .filter(|v| !v.is_empty())
            .map(|s| s.as_str());
        let is_outdated = match aur_ver {
            Some(aur) => {
                versions::compare_versions(aur, &r.upstream_version)
                    == versions::VersionComparison::LessThan
            }
            None => true,
        };
        check_results.push(UpstreamCheckResult {
            pkgname: r.pkgname,
            software_id: r.software_id,
            upstream_version: r.upstream_version,
            is_outdated,
            license_spdx_id: r.license_spdx_id,
            language_names: r.language_names,
        });
    }

    // 批量写入数据库：单次加锁，循环写入所有结果
    let db = state.db.lock()?;
    let mut results = Vec::new();
    for result in &check_results {
        if !result.upstream_version.is_empty() {
            let cleaned_version = result
                .upstream_version
                .strip_prefix('v')
                .unwrap_or(&result.upstream_version);

            let upstream_license_id = result.license_spdx_id.clone();

            let language_ids = db.resolve_language_ids(&result.language_names)?;

            if let Err(e) = db.update_software_outdated(result.software_id, result.is_outdated) {
                error!(
                    "[版本检查] 更新 {} 的 is_outdated 失败: {}",
                    result.pkgname, e
                );
            }

            // 只有当用户没有手动设置语言列表时，才用自动检测到的语言列表填充
            if let Some(existing_langs) = lang_by_id.get(&result.software_id) {
                if existing_langs.is_empty() && !language_ids.is_empty() {
                    if let Err(e) = db.update_software_languages(result.software_id, &language_ids)
                    {
                        error!(
                            "[版本检查] 更新 {} 的 languages 失败: {}",
                            result.pkgname, e
                        );
                    }
                }
            }

            let upstream_info = UpstreamInfo {
                software_id: result.software_id,
                upstream_version: Some(cleaned_version.to_string()),
                upstream_license_id,
                last_checked: Some(Utc::now().timestamp()),
                upstream_url_status: None,
            };
            if let Err(e) = db.upsert_upstream_info(&upstream_info) {
                error!(
                    "[版本检查] 更新 {} 的 upstream_info 失败: {}",
                    result.pkgname, e
                );
            } else {
                info!(
                    "[版本检查结果] {}: AUR={:?} 上游={} 需更新={}",
                    result.pkgname,
                    aur_map.get(&result.software_id),
                    result.upstream_version,
                    result.is_outdated
                );
            }

            results.push((result.pkgname.clone(), result.upstream_version.clone()));
        } else {
            if let Err(e) = db.update_software_outdated(result.software_id, false) {
                error!(
                    "[版本检查] 更新 {} 的 is_outdated 失败: {}",
                    result.pkgname, e
                );
            }
        }
    }

    // Manual 检查器包：跳过网络与写库，仅回传包名标记
    for pkgname in outcome.manual {
        results.push((pkgname, "manual".to_string()));
    }

    info!("已完成 {} 个软件包的上游版本检查", results.len());
    Ok(results)
}
