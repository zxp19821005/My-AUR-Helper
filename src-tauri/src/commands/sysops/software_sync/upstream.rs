/**
 * upstream.rs - 上游版本检查命令（分类并行执行）
 *
 * 功能：并行检查所有软件包的上游最新版本。
 * 先按检查器类型分类，再交给批量执行引擎（batch 模块）在受控并发下检查，
 * 结果收集到内存后批量写入数据库，减少锁竞争。
 *
 * 工作流程：
 * 1. 从数据库读取所有软件包及其检查器配置
 * 2. 映射为 PackageTask，交由 batch_check_upstream 分类并行检查
 * 3. 收集结果，与 AUR 版本比较得出 is_outdated
 * 4. 批量更新数据库中的 upstream_info 和 is_outdated 字段
 * 5. Manual 检查器包跳过网络请求，仅回传包名（不写库）
 */
use log::{error, info};
use tauri::State;

use super::super::proxy_utils::{build_client, get_active_proxy};
use super::batch::{batch_check_upstream, PackageTask};
use super::utils::{
    build_checker_settings, get_setting_opt, parse_u32, parse_u64, UpstreamCheckResult,
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

    // 将 SoftwareInfo 映射为批量检查任务（保留包类型，为后续按包类型批量优化预留）
    let tasks: Vec<PackageTask> = packages
        .into_iter()
        .map(|sw| PackageTask {
            pkgname: sw.pkgname,
            software_id: sw.software_id.unwrap_or(0),
            upstream_url: sw.upstream_url.unwrap_or_default(),
            version_extract_regex: sw.version_extract_regex,
            check_test_versions: sw.check_test_versions,
            check_binary_files: sw.check_binary_files,
            checker_type: sw.checker_type_id,
            package_type: sw.package_type_id,
        })
        .collect();

    let client = build_client(timeout, proxy_url.as_deref());

    // 分类并行检查：Manual 跳过网络，Browser 限严格并发，其余限全局并发
    let outcome = batch_check_upstream(tasks, client, settings, retry).await;

    // 收集检查结果，与 AUR 版本比较得出是否过期
    let mut check_results: Vec<UpstreamCheckResult> = Vec::new();
    for r in outcome.checked {
        if r.upstream_version.is_empty() {
            // 检查失败 / 无版本：仍记录，写库阶段置 is_outdated=false
            check_results.push(r);
            continue;
        }
        let aur_ver = {
            let db = state.db.lock()?;
            db.get_aur_info(r.software_id)
                .ok()
                .flatten()
                .and_then(|a| a.aur_version)
                .filter(|v| !v.is_empty())
        };

        let is_outdated = match aur_ver.as_deref() {
            Some(aur) => {
                crate::versions::compare_versions(aur, &r.upstream_version)
                    == crate::versions::VersionComparison::LessThan
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

    // 批量写入数据库
    let db = state.db.lock()?;
    let mut success_results = Vec::new();
    for result in &check_results {
        if !result.upstream_version.is_empty() {
            let cleaned_version = result
                .upstream_version
                .strip_prefix('v')
                .unwrap_or(&result.upstream_version);

            // 获取 license JSON（直接存储数组）
            let upstream_license_id = result.license_spdx_id.clone();

            // 解析语言 ID 列表（如果语言不存在则自动创建）
            let language_ids = db.resolve_language_ids(&result.language_names)?;
            info!(
                "[版本检查结果] {}: languages={:?} -> ids={:?}",
                result.pkgname, result.language_names, language_ids
            );

            if let Err(e) = db.update_software_outdated(result.software_id, result.is_outdated) {
                error!(
                    "[版本检查] 更新 {} 的 is_outdated 失败: {}",
                    result.pkgname, e
                );
            }

            // 只有当用户没有手动设置语言列表时，才用自动检测到的语言列表填充
            let existing_sw = db.get_software_by_name(&result.pkgname)?;
            if let Some(ref sw) = existing_sw {
                if sw.language_ids.is_empty() && !language_ids.is_empty() {
                    if let Err(e) = db.update_software_languages(result.software_id, &language_ids)
                    {
                        error!(
                            "[版本检查] 更新 {} 的 languages 失败: {}",
                            result.pkgname, e
                        );
                    }
                }
            }

            let upstream_info = crate::models::UpstreamInfo {
                software_id: result.software_id,
                upstream_version: Some(cleaned_version.to_string()),
                upstream_license_id,
                last_checked: Some(chrono::Utc::now().timestamp()),
                upstream_url_status: None,
            };
            if let Err(e) = db.upsert_upstream_info(&upstream_info) {
                error!(
                    "[版本检查] 更新 {} 的 upstream_info 失败: {}",
                    result.pkgname, e
                );
            } else {
                info!(
                    "[版本检查] {} 数据库更新完成: version={}, license={:?}",
                    result.pkgname, cleaned_version, result.license_spdx_id
                );
            }

            success_results.push((result.pkgname.clone(), result.upstream_version.clone()));
        } else {
            if let Err(e) = db.update_software_outdated(result.software_id, false) {
                error!(
                    "[版本检查] 更新 {} 的 is_outdated 失败: {}",
                    result.pkgname, e
                );
            }
        }
    }

    // Manual 检查器包：跳过网络与写库，仅回传包名标记（前端可展示「需手动更新」）
    for pkgname in outcome.manual {
        success_results.push((pkgname, "manual".to_string()));
    }

    info!("已完成 {} 个软件包的上游版本检查", success_results.len());
    Ok(success_results)
}
