use log::{debug, info};
use tauri::State;

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

    let mut count = 0i64;
    for pkgname in &pkgnames {
        if let Some(data) = pkgname_to_data.get(pkgname) {
            debug!("处理软件包: {}", pkgname);
            debug!("AUR API 原始返回: {}", serde_json::to_string_pretty(&data).unwrap_or_default());

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

            debug!("  解析字段: Description={:?}, Version={:?}, URL={:?}, LastModified={:?}", desc, version, url, last_modified);
            debug!("  License数组={:?}, 首个License={:?}", license_arr, license_str);
            debug!("  Depends={:?}, MakeDepends={:?}, OptDepends={:?}", depends_arr, makedepends_arr, optdepends_arr);
            debug!("  OutOfDate={:?}", out_of_date_val);

            let db = state.db.lock()?;
            let sw = db.get_software_by_name(pkgname)?;
            if let Some(mut existing) = sw {
                if let Some(sid) = existing.software_id {
                    let (package_type_id, checker_type_id, check_test_versions, check_binary_files) = detect_package_defaults(&existing.pkgname);
                    if existing.checker_type_id != checker_type_id
                        || existing.package_type_id != package_type_id
                        || existing.check_test_versions != check_test_versions
                        || existing.check_binary_files != check_binary_files
                    {
                        debug!("  更新检查器: {} 当前={:?} → 期望={:?}", existing.pkgname, existing.checker_type_id, checker_type_id);
                        existing.checker_type_id = checker_type_id;
                        existing.package_type_id = package_type_id;
                        existing.check_test_versions = check_test_versions;
                        existing.check_binary_files = check_binary_files;
                        let _ = db.upsert_software(&existing);
                    }

                    let license_id = license_str
                        .and_then(|lic| {
                            debug!("  查找 License: spdx_id='{}'", lic);
                            let result = db.get_license_by_spdx_id(lic).ok();
                            match &result {
                                Some(Some(e)) => debug!("  匹配到 License: id={:?}", e.id),
                                Some(None) => debug!("  enum_licenses 中无匹配"),
                                None => debug!("  查询失败"),
                            }
                            result
                        })
                        .flatten()
                        .and_then(|e| {
                            debug!("  最终 license_id={:?}", e.id);
                            e.id
                        });

                    let depends = depends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                    let makedepends = makedepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                    let optdepends = optdepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                    debug!("  最终字段: license_id={:?}, depends={:?}, makedepends={:?}, optdepends={:?}, out_of_date={:?}",
                        license_id, depends, makedepends, optdepends, out_of_date_val);

                    let aur_info = AurInfo {
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
                    debug!("  写入 aur_info: {:?}", aur_info);
                    let _ = db.upsert_aur_info(&aur_info);
                    if let Ok(Some(stored)) = db.get_aur_info(sid) {
                        debug!("  upsert后验证: license_id={:?}", stored.license_id);
                    }
                    count += 1;
                }
            } else {
                debug!("  未在 software_info 中找到: {}", pkgname);
            }
        } else {
            debug!("  AUR API 返回空或无结果: {}", pkgname);
        }
    }
    info!("已从 AUR 同步 {} 个软件包", count);
    Ok(count)
}

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
    let (settings_timeout, proxy_url) = {
        let db = state.db.lock()?;
        let timeout = parse_u64(&get_setting_opt(&db, "http_timeout").unwrap_or_default(), 30);
        let proxy_url = get_active_proxy(&db);
        (timeout, proxy_url)
    };
    let client = build_client(settings_timeout, proxy_url.as_deref());
    let mut count = 0i64;
    for pkgname in &pkgnames {
        debug!("请求 AUR 信息: {}", pkgname);
        if let Ok(Some(data)) = aur::get_package_info(&client, pkgname).await {
            debug!("AUR API 原始返回: {}", serde_json::to_string_pretty(&data).unwrap_or_default());

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

            debug!("  解析字段: Description={:?}, Version={:?}, URL={:?}, LastModified={:?}", desc, version, url, last_modified);
            debug!("  License数组={:?}, 首个License={:?}", license_arr, license_str);
            debug!("  Depends={:?}", depends_arr);
            debug!("  MakeDepends={:?}", makedepends_arr);
            debug!("  OptDepends={:?}", optdepends_arr);
            debug!("  OutOfDate={:?}", out_of_date_val);

            let db = state.db.lock()?;
            let sw = db.get_software_by_name(pkgname)?;
            if let Some(existing) = sw {
                if let Some(sid) = existing.software_id {
                    let license_id = license_str
                        .and_then(|lic| {
                            debug!("  查找 License: spdx_id='{}'", lic);
                            let result = db.get_license_by_spdx_id(lic).ok();
                            match &result {
                                Some(Some(e)) => debug!("  匹配到 License: id={:?}", e.id),
                                Some(None) => debug!("  enum_licenses 中无匹配"),
                                None => debug!("  查询失败"),
                            }
                            result
                        })
                        .flatten()
                        .and_then(|e| {
                            debug!("  最终 license_id={:?}", e.id);
                            e.id
                        });

                    let depends = depends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                    let makedepends = makedepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                    let optdepends = optdepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                    debug!("  最终字段: license_id={:?}, depends={:?}, makedepends={:?}, optdepends={:?}, out_of_date={:?}",
                        license_id, depends, makedepends, optdepends, out_of_date_val);

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
                    debug!("  写入 aur_info: {:?}", info);
                    let _ = db.upsert_aur_info(&info);
                    if let Ok(Some(stored)) = db.get_aur_info(sid) {
                        debug!("  upsert后验证: license_id={:?}", stored.license_id);
                    }
                    count += 1;
                }
            } else {
                debug!("  未在 software_info 中找到: {}", pkgname);
            }
        } else {
            debug!("  AUR API 返回空或无结果: {}", pkgname);
        }
    }
    info!("已更新 {} 个软件包的 AUR 信息", count);
    Ok(count)
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