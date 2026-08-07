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
 *
 * 注意：AUR 同步只更新 aur_info 表（描述、版本、依赖等），不更新 software_info 表。
 * software_info 的字段（上游URL、检查器类型、包类型等）只在用户手动设置时更新。
 */
use log::{debug, info};
use tauri::State;

use super::super::proxy_utils::{build_client, get_active_proxy};
use super::utils::{get_setting_opt, parse_aur_fields, parse_u32, parse_u64, AurSyncResult};
use crate::aur;
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
    let (username, timeout, proxy_url, batch_size, batch_interval) = {
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
        let batch_size = parse_u32(
            &get_setting_opt(&db, "aur_batch_size").unwrap_or_default(),
            50,
        ) as usize;
        let batch_interval = parse_u64(
            &get_setting_opt(&db, "aur_batch_interval").unwrap_or_default(),
            5,
        );
        (username, timeout, proxy_url, batch_size, batch_interval)
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

    let aur_results =
        aur::get_packages_info(&client, &pkgnames, batch_size.min(100), batch_interval).await?;
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

            let fields = parse_aur_fields(data);

            let db = state.db.lock()?;
            let sw = db.get_software_by_name(pkgname)?;
            if let Some(existing) = sw {
                if let Some(sid) = existing.software_id {
                    // AUR 同步只更新 aur_info 表，不更新 software_info 表
                    // software_info 的字段（上游URL、检查器类型、包类型等）只在用户手动设置时更新
                    sync_results.push(AurSyncResult {
                        pkgname: pkgname.clone(),
                        software_id: sid,
                        desc: fields.desc,
                        version: fields.version,
                        url: fields.url,
                        last_modified: fields.last_modified,
                        license_spdx: fields.license_spdx,
                        depends: fields.depends,
                        makedepends: fields.makedepends,
                        optdepends: fields.optdepends,
                        out_of_date: fields.out_of_date,
                        package_type: existing.package_type_id,
                        checker_type: existing.checker_type_id,
                        check_test_versions: existing.check_test_versions,
                        check_binary_files: existing.check_binary_files,
                        need_update_software: false,
                    });
                }
            }
        }
    }

    // 批量写入数据库
    let db = state.db.lock()?;
    let mut count = 0i64;
    let mut errors = Vec::new();
    for result in &sync_results {
        // AUR 同步只更新 aur_info 表，不更新 software_info 表
        let aur_info = AurInfo {
            software_id: result.software_id,
            pkgdesc: result.desc.clone(),
            aur_version: result.version.clone(),
            license_id: result.license_spdx.clone(),
            last_updated: result.last_modified,
            depends: result.depends.clone(),
            makedepends: result.makedepends.clone(),
            optdepends: result.optdepends.clone(),
            out_of_date: result.out_of_date,
        };
        if let Err(e) = db.upsert_aur_info(&aur_info) {
            errors.push(format!("更新 {} 的 AUR 信息失败: {}", result.pkgname, e));
        }
        count += 1;
    }

    if !errors.is_empty() {
        log::warn!(
            "[sync_from_aur] 部分写入失败 ({} 个): {:?}",
            errors.len(),
            errors
        );
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
    let mut errors = Vec::new();
    for (pkgname, data) in &results {
        let fields = parse_aur_fields(data);

        let sw = db.get_software_by_name(pkgname)?;
        if let Some(existing) = sw {
            if let Some(sid) = existing.software_id {
                let info = AurInfo {
                    software_id: sid,
                    pkgdesc: fields.desc,
                    aur_version: fields.version,
                    license_id: fields.license_spdx,
                    last_updated: fields.last_modified,
                    depends: fields.depends,
                    makedepends: fields.makedepends,
                    optdepends: fields.optdepends,
                    out_of_date: fields.out_of_date,
                };
                if let Err(e) = db.upsert_aur_info(&info) {
                    errors.push(format!("更新 {} 的 AUR 信息失败: {}", pkgname, e));
                }
                count += 1;
            }
        }
    }

    if !errors.is_empty() {
        log::warn!(
            "[update_aur_info] 部分写入失败 ({} 个): {:?}",
            errors.len(),
            errors
        );
    }
    info!("已更新 {} 个软件包的 AUR 信息", count);
    Ok(count)
}
