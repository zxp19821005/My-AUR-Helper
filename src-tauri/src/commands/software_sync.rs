use log::{debug, info, error};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use crate::aur;
use crate::commands::proxy_utils::{build_client, get_active_proxy};
use crate::errors::{AppError, AppResult};
use crate::models::*;
use crate::AppState;

fn get_setting_opt(db: &crate::db::Database, key: &str) -> Option<String> {
    db.get_setting(key)
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
}

fn parse_u64(val: &str, default: u64) -> u64 {
    val.parse().unwrap_or(default)
}

/// AUR 同步结果（用于内存缓冲）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AurSyncResult {
    pkgname: String,
    software_id: i64,
    desc: Option<String>,
    version: Option<String>,
    url: Option<String>,
    last_modified: Option<i64>,
    license_spdx: Option<String>,
    depends: Option<String>,
    makedepends: Option<String>,
    optdepends: Option<String>,
    out_of_date: Option<bool>,
    package_type: PackageType,
    checker_type: CheckerType,
    check_test_versions: bool,
    check_binary_files: bool,
    need_update_software: bool,
}

/// 上游检查结果（用于内存缓冲）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpstreamCheckResult {
    pkgname: String,
    software_id: i64,
    upstream_version: String,
    is_outdated: bool,
}

/// 后台任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub task_id: String,
    pub task_type: String,
    pub total: usize,
    pub completed: usize,
    pub status: String, // running, completed, failed
    pub results: Vec<serde_json::Value>,
}

/// 后台任务管理器
pub struct TaskManager {
    pub tasks: std::collections::HashMap<String, Arc<TokioMutex<TaskStatus>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: std::collections::HashMap::new(),
        }
    }

    pub fn create_task(&mut self, task_id: String, task_type: String, total: usize) -> Arc<TokioMutex<TaskStatus>> {
        let status = Arc::new(TokioMutex::new(TaskStatus {
            task_id: task_id.clone(),
            task_type,
            total,
            completed: 0,
            status: "running".to_string(),
            results: Vec::new(),
        }));
        self.tasks.insert(task_id, status.clone());
        status
    }

    pub fn get_task(&self, task_id: &str) -> Option<Arc<TokioMutex<TaskStatus>>> {
        self.tasks.get(task_id).cloned()
    }
}

/// 并行同步 AUR 信息
#[tauri::command]
pub async fn sync_from_aur(state: State<'_, AppState>) -> AppResult<i64> {
    info!("正在从 AUR 同步软件包");
    let (username, timeout, proxy_url) = {
        let db = state.db.lock()?;
        let username = db.get_setting("aur_username")?
            .map(|s| s.value)
            .unwrap_or_default();
        let timeout = parse_u64(&get_setting_opt(&db, "http_timeout").unwrap_or_default(), 30);
        let proxy_url = get_active_proxy(&db);
        (username, timeout, proxy_url)
    };
    if username.is_empty() {
        return Err(AppError::ConfigError("AUR 用户名未配置".to_string()));
    }
    let client = build_client(timeout, proxy_url.as_deref());

    let pkgnames = {
        let db = state.db.lock()?;
        db.get_all_software()?
            .into_iter()
            .map(|s| s.pkgname)
            .collect::<Vec<String>>()
    };
    info!("准备同步 {} 个软件包的 AUR 信息", pkgnames.len());

    let aur_results = aur::get_packages_info(&client, &pkgnames).await?;
    debug!("批量查询返回 {} 条结果", aur_results.len());

    let mut pkgname_to_data = std::collections::HashMap::new();
    for data in &aur_results {
        if let Some(name) = data["Name"].as_str() {
            pkgname_to_data.insert(name.to_string(), data.clone());
        }
    }

    // 收集所有同步结果到内存
    let mut sync_results: Vec<AurSyncResult> = Vec::new();
    
    for pkgname in &pkgnames {
        if let Some(data) = pkgname_to_data.get(pkgname) {
            debug!("处理软件包: {}", pkgname);

            let desc = data["Description"].as_str().map(|s| s.to_string());
            let version = data["Version"].as_str().map(|s| s.to_string());
            let url = data["URL"].as_str().map(|s| s.to_string());
            let last_modified = data["LastModified"].as_i64();
            let license_arr = data["License"].as_array();
            let license_str = license_arr.and_then(|a| a.first()).and_then(|v| v.as_str());
            let depends_arr = data["Depends"].as_array();
            let makedepends_arr = data["MakeDepends"].as_array();
            let optdepends_arr = data["OptDepends"].as_array();
            let out_of_date_val = data["OutOfDate"].as_i64();

            let db = state.db.lock()?;
            let sw = db.get_software_by_name(pkgname)?;
            if let Some(existing) = sw {
                if let Some(sid) = existing.software_id {
                    let (package_type, checker_type, check_test_versions, check_binary_files) = detect_package_defaults(&existing.pkgname);
                    let need_update = existing.checker_type_id != checker_type
                        || existing.package_type_id != package_type
                        || existing.check_test_versions != check_test_versions
                        || existing.check_binary_files != check_binary_files;

                    let license_spdx = license_str.map(|s| s.to_string());
                    let depends = depends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                    let makedepends = makedepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                    let optdepends = optdepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());

                    sync_results.push(AurSyncResult {
                        pkgname: pkgname.clone(),
                        software_id: sid,
                        desc,
                        version,
                        url,
                        last_modified,
                        license_spdx,
                        depends,
                        makedepends,
                        optdepends,
                        out_of_date: out_of_date_val.map(|v| v != 0),
                        package_type,
                        checker_type,
                        check_test_versions,
                        check_binary_files,
                        need_update_software: need_update,
                    });
                }
            }
        }
    }

    // 批量写入数据库
    let db = state.db.lock()?;
    let mut count = 0i64;
    for result in &sync_results {
        if result.need_update_software {
            if let Ok(Some(mut sw)) = db.get_software_by_name(&result.pkgname) {
                sw.checker_type_id = result.checker_type.clone();
                sw.package_type_id = result.package_type.clone();
                sw.check_test_versions = result.check_test_versions;
                sw.check_binary_files = result.check_binary_files;
                let _ = db.upsert_software(&sw);
            }
        }

        let license_id = result.license_spdx.as_deref()
            .and_then(|lic| {
                db.get_license_by_spdx_id(lic).ok().flatten()
                    .and_then(|e| e.id)
            });

        let aur_info = AurInfo {
            software_id: result.software_id,
            pkgdesc: result.desc.clone(),
            aur_version: result.version.clone(),
            license_id,
            last_updated: result.last_modified,
            depends: result.depends.clone(),
            makedepends: result.makedepends.clone(),
            optdepends: result.optdepends.clone(),
            out_of_date: result.out_of_date,
        };
        let _ = db.upsert_aur_info(&aur_info);
        count += 1;
    }

    info!("已从 AUR 同步 {} 个软件包", count);
    Ok(count)
}

/// 并行更新 AUR 信息
#[tauri::command]
pub async fn update_aur_info(
    state: State<'_, AppState>,
    pkgname_list: Option<Vec<String>>,
) -> AppResult<i64> {
    info!("正在更新软件包的 AUR 信息");
    let pkgnames: Vec<String> = if let Some(list) = pkgname_list {
        list
    } else {
        let db = state.db.lock()?;
        db.get_all_software()?
            .into_iter()
            .map(|s| s.pkgname)
            .collect()
    };
    let (timeout, proxy_url) = {
        let db = state.db.lock()?;
        let timeout = parse_u64(&get_setting_opt(&db, "http_timeout").unwrap_or_default(), 30);
        let proxy_url = get_active_proxy(&db);
        (timeout, proxy_url)
    };
    let client = build_client(timeout, proxy_url.as_deref());
    
    // 并行获取所有 AUR 信息
    let mut handles = Vec::new();
    for pkgname in &pkgnames {
        let client = client.clone();
        let pkgname_clone = pkgname.clone();
        let pkgname_for_handle = pkgname.clone();
        let handle = tokio::spawn(async move {
            debug!("请求 AUR 信息: {}", pkgname_clone);
            aur::get_package_info(&client, &pkgname_clone).await.ok().flatten()
        });
        handles.push((pkgname_for_handle, handle));
    }

    // 收集结果到内存
    let mut results: Vec<(String, serde_json::Value)> = Vec::new();
    for (pkgname, handle) in handles {
        if let Ok(Some(data)) = handle.await {
            results.push((pkgname, data));
        }
    }

    // 批量写入数据库
    let db = state.db.lock()?;
    let mut count = 0i64;
    for (pkgname, data) in &results {
        let desc = data["Description"].as_str().map(|s| s.to_string());
        let version = data["Version"].as_str().map(|s| s.to_string());
        let _url = data["URL"].as_str().map(|s| s.to_string());
        let last_modified = data["LastModified"].as_i64();
        let license_arr = data["License"].as_array();
        let license_str = license_arr.and_then(|a| a.first()).and_then(|v| v.as_str());
        let depends_arr = data["Depends"].as_array();
        let makedepends_arr = data["MakeDepends"].as_array();
        let optdepends_arr = data["OptDepends"].as_array();
        let out_of_date_val = data["OutOfDate"].as_i64();

        let sw = db.get_software_by_name(pkgname)?;
        if let Some(existing) = sw {
            if let Some(sid) = existing.software_id {
                let license_id = license_str
                    .and_then(|lic| {
                        db.get_license_by_spdx_id(lic).ok().flatten()
                            .and_then(|e| e.id)
                    });

                let depends = depends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                let makedepends = makedepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                let optdepends = optdepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());

                let info = AurInfo {
                    software_id: sid,
                    pkgdesc: desc,
                    aur_version: version,
                    license_id,
                    last_updated: last_modified,
                    depends,
                    makedepends,
                    optdepends,
                    out_of_date: out_of_date_val.map(|v| v != 0),
                };
                let _ = db.upsert_aur_info(&info);
                count += 1;
            }
        }
    }

    info!("已更新 {} 个软件包的 AUR 信息", count);
    Ok(count)
}

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
    Err(last_error.unwrap_or(AppError::VersionCheckError("检查失败".to_string())))
}

fn build_checker_settings(db: &crate::db::Database) -> crate::checkers::CheckerSettings {
    crate::checkers::CheckerSettings {
        github_token: get_setting_opt(db, "github_token"),
        gitee_token: get_setting_opt(db, "gitee_token"),
        gitlab_token: get_setting_opt(db, "gitlab_token"),
    }
}

fn parse_u32(val: &str, default: u32) -> u32 {
    val.parse().unwrap_or(default)
}

pub fn detect_package_defaults(pkgname: &str) -> (PackageType, CheckerType, bool, bool) {
    if pkgname.ends_with("-git") {
        (PackageType::Git, CheckerType::GitHubAPI, true, false)
    } else if pkgname.ends_with("-bin") {
        (PackageType::Binary, CheckerType::GitHubAPI, false, true)
    } else if pkgname.ends_with("-appimage") {
        (PackageType::AppImage, CheckerType::GitHubAPI, false, true)
    } else {
        (PackageType::Compiled, CheckerType::GitHubTags, false, false)
    }
}