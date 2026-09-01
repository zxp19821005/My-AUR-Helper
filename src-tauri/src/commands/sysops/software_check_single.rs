/**
 * software_check_single.rs - 单软件包版本检查命令
 *
 * 功能：
 * - check_upstream_version: 检查单个软件包的上游版本
 *
 * 本模块从 software_check.rs 拆分，避免单文件超过 300 行限制。
 */
use log::{debug, info};
use tauri::State;

use super::proxy_utils::build_client;
use super::software_check::{apply_upstream_check_result, check_with_retry};
use super::software_sync::utils::{build_checker_settings, read_http_settings};
use crate::checkers::{self, CheckOptions};
use crate::errors::{AppError, AppResult};
use crate::versions;
use crate::AppState;

/// 比较版本并更新数据库（单包场景）
fn compare_and_update_single(
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

/// 检查单个软件包的上游版本
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
    compare_and_update_single(
        &mut db,
        sw.software_id.unwrap_or(0),
        &sw.pkgname,
        &version,
        check_result.license.as_deref(),
        &check_result.language_names,
    )?;
    Ok(version)
}
