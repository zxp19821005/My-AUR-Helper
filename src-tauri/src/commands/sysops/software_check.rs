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

use super::proxy_utils::build_client;
use super::software_sync::batch::{batch_check_upstream, PackageTask};
use super::software_sync::utils::{
    build_checker_settings, read_http_settings, UpstreamCheckResult,
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

/// 写入单个软件包的上游检查结果（is_outdated / 语言列表 / upstream_info）
///
/// 在调用方传入的连接（可处于事务中）上执行，便于批量写入原子化。
///
/// # Arguments
/// - `conn` 数据库连接（可为事务连接）
/// - `software_id` 软件包 ID
/// - `cleaned_version` 已去除 `v` 前缀的上游版本
/// - `is_outdated` 与 AUR 相比是否落后
/// - `upstream_license_id` 上游 License SPDX ID（可选）
/// - `language_ids` 解析后的语言 ID 列表（已在外层解析，避免在事务内借阅冲突）
/// - `fill_languages` 是否填充语言列表（仅当用户未手动设置时）
fn apply_upstream_check_result(
    conn: &rusqlite::Connection,
    software_id: i64,
    cleaned_version: &str,
    is_outdated: bool,
    upstream_license_id: Option<String>,
    language_ids: &[i64],
    fill_languages: bool,
) -> AppResult<()> {
    crate::db::Database::update_software_outdated_conn(conn, software_id, is_outdated)?;
    if fill_languages && !language_ids.is_empty() {
        crate::db::Database::update_software_languages_conn(conn, software_id, language_ids)?;
    }
    let upstream_info = UpstreamInfo {
        software_id,
        upstream_version: Some(cleaned_version.to_string()),
        upstream_license_id,
        last_checked: Some(Utc::now().timestamp()),
        upstream_url_status: None,
    };
    crate::db::Database::upsert_upstream_info_conn(conn, &upstream_info)?;
    Ok(())
}

fn compare_and_update(
    db: &mut crate::db::Database,
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

    let upstream_license_id = license_spdx_id.map(|s| s.to_string());
    let language_ids = db.resolve_language_ids(language_names)?;

    // 仅当用户未手动设置语言列表时，才用自动检测到的语言列表填充
    let fill_languages = db
        .get_software_by_name(pkgname)?
        .map(|sw| sw.language_ids.is_empty())
        .unwrap_or(false);

    // 单包三次写入（is_outdated / languages / upstream_info）包裹于同一事务，
    // 避免中途失败时留下部分写入。
    let tx = db.conn.transaction()?;
    apply_upstream_check_result(
        &tx,
        software_id,
        cleaned_version,
        is_outdated,
        upstream_license_id,
        &language_ids,
        fill_languages,
    )?;
    tx.commit()?;
    Ok(())
}

#[tauri::command]
pub async fn check_upstream_version(
    state: State<'_, AppState>,
    pkgname: String,
) -> AppResult<String> {
    info!("正在检查上游版本: {}", pkgname);
    let (sw, settings, timeout, retry) = {
        let db = state.db.lock()?;
        let sw = db
            .get_software_by_name(&pkgname)?
            .ok_or_else(|| AppError::PackageNotFound(pkgname.clone()))?;
        let settings = build_checker_settings(&db);
        let (timeout, retry) = read_http_settings(&db);
        (sw, settings, timeout, retry)
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

    let checker_type = sw.checker_type_id;
    let client = build_client(timeout, checker_type.is_github());
    let checker = checkers::get_checker(&checker_type, settings);
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

    let mut db = state.db.lock()?;
    compare_and_update(
        &mut db,
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
    let (packages, settings, timeout, retry) = {
        let db = state.db.lock()?;
        let mut packages = Vec::new();
        for pkgname in &pkgname_list {
            if let Some(sw) = db.get_software_by_name(pkgname)? {
                packages.push(sw);
            }
        }
        let settings = build_checker_settings(&db);
        let (timeout, retry) = read_http_settings(&db);
        (packages, settings, timeout, retry)
    };

    // 预先提取每个包的已有语言 ID，避免写库阶段逐包回查（消除 N+1 查询）
    let lang_by_id: HashMap<i64, Vec<i64>> = packages
        .iter()
        .map(|p| (p.software_id.unwrap_or(0), p.language_ids.clone()))
        .collect();

    // 一次性批量读取所有 AUR 版本（单条 SQL + 单次加锁），
    // 同时用于「无 AUR 版本前置过滤」与「版本比较」，避免循环内逐包回查（原 N+1 残留）。
    let aur_map: HashMap<i64, String> = {
        let db = state.db.lock()?;
        db.get_aur_versions_map()?
    };

    // 过滤掉没有 AUR 版本的包（前置检查，避免浪费网络请求）
    let mut tasks = Vec::new();
    for sw in &packages {
        let has_aur = aur_map
            .get(&sw.software_id.unwrap_or(0))
            .map(|v| !v.is_empty())
            .unwrap_or(false);
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

    if tasks.is_empty() {
        info!("[版本检查] 所有选中包均无 AUR 版本，跳过检查");
        return Ok(Vec::new());
    }

    let client = build_client(timeout, false);
    let github_client = build_client(timeout, true);

    // 并行检查：复用 batch_check_upstream 的分类并发引擎
    let outcome = batch_check_upstream(tasks, client, github_client, settings, retry).await;

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

    // 预先解析所有结果的语言 ID：解析过程会写库（get_or_create），
    // 须在事务外加锁完成，避免与下方 &mut db 的事务借阅冲突。
    let lang_ids_by_sw: HashMap<i64, Vec<i64>> = {
        let db = state.db.lock()?;
        let mut map = HashMap::new();
        for result in &check_results {
            if !result.upstream_version.is_empty() && !result.language_names.is_empty() {
                map.insert(
                    result.software_id,
                    db.resolve_language_ids(&result.language_names)?,
                );
            }
        }
        map
    };

    // 批量写入数据库：单个事务包裹全部写操作（is_outdated / 语言 / upstream_info），
    // 保证「全有或全无」的原子性，避免中途失败时留下部分写入（优化文档 C1）。
    let mut db = state.db.lock()?;
    let tx = db.conn.transaction()?;
    let mut results = Vec::new();
    for result in &check_results {
        if !result.upstream_version.is_empty() {
            let cleaned_version = result
                .upstream_version
                .strip_prefix('v')
                .unwrap_or(&result.upstream_version);
            let upstream_license_id = result.license_spdx_id.clone();
            // 语言 ID 已在事务外预解析；仅当用户未手动设置语言列表时填充
            let language_ids = lang_ids_by_sw
                .get(&result.software_id)
                .cloned()
                .unwrap_or_default();
            let fill_languages = lang_by_id
                .get(&result.software_id)
                .map(|l| l.is_empty())
                .unwrap_or(false)
                && !language_ids.is_empty();

            apply_upstream_check_result(
                &tx,
                result.software_id,
                cleaned_version,
                result.is_outdated,
                upstream_license_id,
                &language_ids,
                fill_languages,
            )?;

            info!(
                "[版本检查结果] {}: AUR={:?} 上游={} 需更新={}",
                result.pkgname,
                aur_map.get(&result.software_id),
                result.upstream_version,
                result.is_outdated
            );

            results.push((result.pkgname.clone(), result.upstream_version.clone()));
        } else {
            // 检查失败 / 无可解析版本：置为未过期，不写 upstream_info
            crate::db::Database::update_software_outdated_conn(&tx, result.software_id, false)?;
        }
    }
    tx.commit()?;

    // Manual 检查器包：跳过网络与写库，仅回传包名标记
    for pkgname in outcome.manual {
        results.push((pkgname, "manual".to_string()));
    }

    info!("已完成 {} 个软件包的上游版本检查", results.len());
    Ok(results)
}