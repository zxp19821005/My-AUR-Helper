/**
 * upstream.rs - 上游版本检查命令（并行执行）
 *
 * 功能：并行检查所有软件包的上游最新版本。
 * 使用 tokio::spawn 并行发起网络请求，每个包独立检查，
 * 结果收集到内存后批量写入数据库，减少锁竞争。
 *
 * 工作流程：
 * 1. 从数据库读取所有软件包及其检查器配置
 * 2. 为每个包创建 tokio::spawn 任务并行检查
 * 3. 每个任务调用对应检查器的 check 方法，支持重试
 * 4. 收集所有结果到内存
 * 5. 批量更新数据库中的 upstream_info 和 is_outdated 字段
 */
use log::{error, info};
use tauri::State;

use super::utils::{
    build_checker_settings, get_setting_opt, parse_u32, parse_u64, UpstreamCheckResult,
};
use crate::commands::proxy_utils::{build_client, get_active_proxy};
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

        let proxy_url_for_spawn = proxy_url.clone();
        let handle = tokio::spawn(async move {
            let checker = crate::checkers::get_checker(&checker_type_id, settings);
            let options = crate::checkers::CheckOptions {
                check_test_versions,
                check_binary_files,
                proxy_url: proxy_url_for_spawn,
            };

            let result = check_with_retry(
                &*checker,
                &client,
                &upstream_url,
                &pkgname,
                version_extract_regex.as_deref(),
                &options,
                retry,
            )
            .await;

            (pkgname, software_id, result)
        });
        handles.push(handle);
    }

    // 收集结果
    let mut check_results: Vec<UpstreamCheckResult> = Vec::new();
    for handle in handles {
        if let Ok((pkgname, software_id, result)) = handle.await {
            match result {
                Ok(check_result) => {
                    if let Some(version) = check_result.version {
                        let aur_ver = {
                            let db = state.db.lock()?;
                            db.get_aur_info(software_id)
                                .ok()
                                .flatten()
                                .and_then(|a| a.aur_version)
                                .filter(|v| !v.is_empty())
                        };

                        let is_outdated = match aur_ver.as_deref() {
                            Some(aur) => {
                                crate::versions::compare_versions(aur, &version)
                                    == crate::versions::VersionComparison::LessThan
                            }
                            None => true,
                        };

                        check_results.push(UpstreamCheckResult {
                            pkgname,
                            software_id,
                            upstream_version: version,
                            is_outdated,
                            license_spdx_id: check_result.license,
                            language_names: check_result.language_names,
                        });
                    } else {
                        check_results.push(UpstreamCheckResult {
                            pkgname,
                            software_id,
                            upstream_version: String::new(),
                            is_outdated: false,
                            license_spdx_id: None,
                            language_names: vec![],
                        });
                    }
                }
                _ => {
                    check_results.push(UpstreamCheckResult {
                        pkgname,
                        software_id,
                        upstream_version: String::new(),
                        is_outdated: false,
                        license_spdx_id: None,
                        language_names: vec![],
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
                error!("[版本检查] 更新 {} 的 is_outdated 失败: {}", result.pkgname, e);
            }
            
            // 更新软件的语言 ID 列表
            if let Err(e) = db.update_software_languages(result.software_id, &language_ids) {
                error!("[版本检查] 更新 {} 的 languages 失败: {}", result.pkgname, e);
            }
            
            let upstream_info = crate::models::UpstreamInfo {
                software_id: result.software_id,
                upstream_version: Some(cleaned_version.to_string()),
                upstream_license_id,
                last_checked: Some(chrono::Utc::now().timestamp()),
            };
            if let Err(e) = db.upsert_upstream_info(&upstream_info) {
                error!("[版本检查] 更新 {} 的 upstream_info 失败: {}", result.pkgname, e);
            } else {
                info!("[版本检查] {} 数据库更新完成: version={}, license={:?}", 
                    result.pkgname, cleaned_version, result.license_spdx_id);
            }

            success_results.push((result.pkgname.clone(), result.upstream_version.clone()));
        } else {
            if let Err(e) = db.update_software_outdated(result.software_id, false) {
                error!("[版本检查] 更新 {} 的 is_outdated 失败: {}", result.pkgname, e);
            }
        }
    }

    info!("已完成 {} 个软件包的上游版本检查", success_results.len());
    Ok(success_results)
}

/// 带重试的版本检查
///
/// # 参数
/// - `checker`: 版本检查器实例
/// - `client`: HTTP 客户端
/// - `upstream_url`: 上游仓库 URL
/// - `pkgname`: 软件包名称
/// - `version_extract_regex`: 版本提取正则表达式（可选）
/// - `options`: 检查选项
/// - `retry_count`: 最大重试次数
///
/// # 返回
/// - `Ok(CheckResult)`: 检查成功，包含版本号和 license 信息
/// - `Err(e)`: 所有重试均失败
async fn check_with_retry(
    checker: &dyn crate::checkers::VersionChecker,
    client: &reqwest::Client,
    upstream_url: &str,
    pkgname: &str,
    version_extract_regex: Option<&str>,
    options: &crate::checkers::CheckOptions,
    retry_count: u32,
) -> AppResult<crate::checkers::CheckResult> {
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
    Err(
        last_error.unwrap_or(crate::errors::AppError::VersionCheckError(
            "检查失败".to_string(),
        )),
    )
}