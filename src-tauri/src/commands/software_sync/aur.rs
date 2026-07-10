/**
 * aur.rs - AUR 信息同步和更新命令
 *
 * 功能：从 AUR RPC API 批量同步软件包信息，或更新指定软件包的 AUR 信息。
 * 使用 tokio::spawn 并行发起网络请求，结果收集到内存后批量写入数据库。
 *
 * 工作流程：
 * 1. 从数据库读取所有软件包列表
 * 2. 通过 AUR RPC API 批量查询软件包信息
 * 3. 在内存中收集所有同步结果
 * 4. 批量写入数据库，减少锁竞争
 */
use log::{debug, info};
use tauri::State;

use super::utils::{detect_package_defaults, get_setting_opt, parse_u64, AurSyncResult};
use crate::aur;
use crate::commands::proxy_utils::{build_client, get_active_proxy};
use crate::errors::{AppError, AppResult};
use crate::models::*;
use crate::AppState;

/// 并行同步 AUR 信息
///
/// 从 AUR RPC API 批量获取所有软件包的最新信息，
/// 自动推断包类型和检查器类型，并更新数据库。
///
/// # 参数
/// - `state`: Tauri 应用状态，包含数据库连接
///
/// # 返回
/// - `Ok(count)`: 成功同步的软件包数量
/// - `Err(e)`: 同步过程中发生错误
#[tauri::command]
pub async fn sync_from_aur(state: State<'_, AppState>) -> AppResult<i64> {
    info!("正在从 AUR 同步软件包");
    let (username, timeout, proxy_url) = {
        let db = state.db.lock()?;
        let username = db
            .get_setting("aur_username")?
            .map(|s| s.value)
            .unwrap_or_default();
        let timeout = parse_u64(
            &get_setting_opt(&db, "http_timeout").unwrap_or_default(),
            30,
        );
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
                    let (package_type, checker_type, check_test_versions, check_binary_files) =
                        detect_package_defaults(&existing.pkgname);
                    let need_update = existing.checker_type_id != checker_type
                        || existing.package_type_id != package_type
                        || existing.check_test_versions != check_test_versions
                        || existing.check_binary_files != check_binary_files;

                    let license_spdx = license_str.map(|s| s.to_string());
                    let depends = depends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                    let makedepends =
                        makedepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                    let optdepends =
                        optdepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());

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

        let license_id = db.get_or_create_license_id(result.license_spdx.as_deref())?;

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
///
/// 更新指定软件包（或全部）的 AUR 信息，包括描述、版本、依赖等。
///
/// # 参数
/// - `state`: Tauri 应用状态，包含数据库连接
/// - `pkgname_list`: 可选的软件包名称列表，None 表示更新全部
///
/// # 返回
/// - `Ok(count)`: 成功更新的软件包数量
/// - `Err(e)`: 更新过程中发生错误
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
        let timeout = parse_u64(
            &get_setting_opt(&db, "http_timeout").unwrap_or_default(),
            30,
        );
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
            aur::get_package_info(&client, &pkgname_clone)
                .await
                .ok()
                .flatten()
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
                let license_id = db.get_or_create_license_id(license_str)?;

                let depends = depends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                let makedepends =
                    makedepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());
                let optdepends =
                    optdepends_arr.map(|a| serde_json::to_string(a).unwrap_or_default());

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
