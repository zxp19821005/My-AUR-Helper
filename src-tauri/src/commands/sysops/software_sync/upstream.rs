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
use log::{error, info, warn};
use std::collections::HashMap;

use tauri::State;

use super::super::proxy_utils::build_client;
use super::batch::{batch_check_upstream, PackageTask, write_github_tag_cache};
use crate::db::github_tag_cache::CacheCheckResult;
use crate::checkers::utils::extract_owner_repo;
use crate::models::{CheckerType, PackageType};
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

    // ---- GitHub tags 缓存：查有效缓存，跳过命中仓库的包，完成后写回 ----
    // 按 (owner, repo) 去重后的仓库列表（用于缓存查询）
    let github_repos: Vec<(String, String)> = tasks
        .iter()
        .filter(|t| matches!(t.checker_type, CheckerType::GitHubTags | CheckerType::GitHubAPI)
            && t.package_type != PackageType::Git
            && extract_owner_repo(&t.upstream_url).is_some())
        .map(|t| {
            let (o, r) = extract_owner_repo(&t.upstream_url).unwrap();
            (o, r)
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    // ---- GitHub tags 增量校验：对过期缓存调 releases/latest 验证，避免全量重拉 ----
    // 逐包检查每个 GitHub 仓库的缓存状态，决定是否需要网络请求
    // check_and_extend_cache 是同步函数（含 blocking HTTP + DB 操作），直接串行执行
    // （GitHub 仓库数量有限，串行完全可接受）
    let skip_keys: std::collections::HashSet<(String, String)> = {
        let db = state.db.lock().unwrap();
        let mut skip: std::collections::HashSet<_> = std::collections::HashSet::new();
        for (owner, repo) in &github_repos {
            // 找该仓库对应包的版本提取正则（取第一个匹配的即可）
            let regex = tasks
                .iter()
                .filter(|t| {
                    matches!(t.checker_type, CheckerType::GitHubTags | CheckerType::GitHubAPI)
                        && t.package_type != PackageType::Git
                        && extract_owner_repo(&t.upstream_url).is_some_and(|(o, r)| o == *owner && r == *repo)
                })
                .next()
                .and_then(|t| t.version_extract_regex.as_deref());
            match db.check_and_extend_cache(owner, repo, regex) {
                Ok(result) => match result {
                    CacheCheckResult::Hit => {
                        skip.insert((owner.clone(), repo.clone()));
                    }
                    CacheCheckResult::Extend => {
                        skip.insert((owner.clone(), repo.clone()));
                    }
                    CacheCheckResult::Recalculate { .. } => {
                        // 需要从缓存 tags 重算或全量重拉，不跳过
                    }
                    CacheCheckResult::Deleted
                    | CacheCheckResult::Miss => {
                        // 无缓存或已清除，不跳过
                    }
                },
                Err(e) => {
                    warn!("[GitHub Cache] {}:{} 校验失败: {}", owner, repo, e);
                }
            }
        }
        info!(
            "[GitHub Cache] 跳过 {} 个仓库（缓存有效或已顺延）",
            skip.len()
        );
        skip
    };
    let tasks: Vec<PackageTask> = tasks
        .into_iter()
        .filter(|t| {
            if !matches!(t.checker_type, CheckerType::GitHubTags | CheckerType::GitHubAPI) {
                return true;
            }
            if let Some((owner, repo)) = extract_owner_repo(&t.upstream_url) {
                if skip_keys.contains(&(owner, repo)) {
                    return false; // 缓存命中或顺延，跳过
                }
            }
            true
        })
        .collect();

    // 分类并行检查：Manual 跳过网络，Browser 限严格并发，其余限全局并发
    let outcome = batch_check_upstream(tasks, client, github_client, settings, retry,
        |_owner, _repo, _tag_count, _json_str| {
            // 缓存写盘由 batch 返回后统一执行（避免跨 await 持有 db 引用）
        }).await;

    // 写回 GitHub tags 缓存
    if !outcome.github_cache_map.is_empty() {
        let db = state.db.lock().unwrap();
        if let Err(e) = write_github_tag_cache(&db, &outcome.github_cache_map) {
            error!("[GitHub Cache] 写盘失败: {}", e);
        }
    }

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
            _all_tags: None,
            _owner: String::new(),
            _repo: String::new(),
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
                // 命中即检查成功，清空上一次留下的失败标记
                last_check_error: None,
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
            // 检查失败 / 未返回版本：写入错误标记，供列表「上游更新失败」筛选定位
            let _ = db.mark_upstream_check_error(result.software_id, "检查未返回上游版本");
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

    // skip_keys 在任务过滤阶段已使用，此处无需额外处理

    info!("已完成 {} 个软件包的上游版本检查", success_results.len());
    Ok(success_results)
}