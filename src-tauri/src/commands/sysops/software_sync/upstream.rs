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
use std::collections::HashMap;

use tauri::State;

use super::super::proxy_utils::build_client;
use super::batch::{batch_check_upstream, PackageTask};
use super::utils::{
    build_checker_settings, read_http_settings, UpstreamCheckResult,
};
use crate::errors::AppResult;
use crate::AppState;

/// 并行检查上游版本
#[tauri::command]
pub async fn check_all_upstream(state: State<'_, AppState>) -> AppResult<Vec<(String, String)>> {
    info!("正在检查所有软件包的上游版本");
    let (packages, settings, timeout, retry) = {
        let db = state.db.lock()?;
        let packages = db.get_all_software()?;
        let settings = build_checker_settings(&db);
        let (timeout, retry) = read_http_settings(&db);
        (packages, settings, timeout, retry)
    };

    // 预先提取每个包的已有语言 ID：初始 get_all_software 已携带该字段，
    // 避免在写库阶段对每个包再按 pkgname 回查数据库（消除 N+1 查询）
    let lang_by_id: HashMap<i64, Vec<i64>> = packages
        .iter()
        .map(|p| (p.software_id.unwrap_or(0), p.language_ids.clone()))
        .collect();

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

    let client = build_client(timeout, false);
    let github_client = build_client(timeout, true);

    // 分类并行检查：Manual 跳过网络，Browser 限严格并发，其余限全局并发
    let outcome = batch_check_upstream(tasks, client, github_client, settings, retry).await;

    // 一次性批量读取所有 AUR 版本（单条 SQL + 单次加锁），替代循环内逐包
    // get_aur_info 的 N+1 查询与反复加锁，显著降低批量检查的数据库开销
    let aur_map: HashMap<i64, String> = {
        let db = state.db.lock()?;
        db.get_aur_versions_map()?
    };

    // 收集检查结果，与 AUR 版本比较得出是否过期
    let mut check_results: Vec<UpstreamCheckResult> = Vec::new();
    for r in outcome.checked {
        if r.upstream_version.is_empty() {
            // 检查失败 / 无版本：仍记录，写库阶段置 is_outdated=false
            check_results.push(r);
            continue;
        }
        // 纯内存比较，不再访问数据库
        let aur_ver = aur_map
            .get(&r.software_id)
            .filter(|v| !v.is_empty())
            .map(|s| s.as_str());

        let is_outdated = match aur_ver {
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
            // 直接用初始加载的 lang_by_id 内存映射判断，无需再次查询数据库
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