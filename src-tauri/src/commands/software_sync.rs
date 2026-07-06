/**
 * software_sync.rs - 软件包 AUR 同步相关命令
 */
use log::{info, error};
use tauri::{Emitter, State};

use crate::aur;
use crate::models::*;
use crate::AppState;

/// 从 AUR 同步软件包（按用户名）
#[tauri::command]
pub async fn sync_from_aur(state: State<'_, AppState>) -> Result<i64, String> {
    info!("正在从 AUR 同步软件包");
    let username = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_setting("aur_username")
            .map_err(|e| e.to_string())?
            .map(|s| s.value)
            .unwrap_or_default()
    };
    if username.is_empty() {
        return Err("AUR username not configured".to_string());
    }
    let client = reqwest::Client::new();
    let packages = aur::fetch_packages_by_user(&client, &username)
        .await
        .map_err(|e| e.to_string())?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    for pkg in &packages {
        let sw = db
            .get_software_by_name(&pkg.pkgname)
            .map_err(|e| e.to_string())?;
        let software_id = if let Some(existing) = sw {
            existing.software_id.unwrap_or(0)
        } else {
            let new_sw = SoftwareInfo {
                software_id: None,
                pkgname: pkg.pkgname.clone(),
                upstream_url: pkg.url.clone(),
                package_type_id: PackageType::Compiled,
                checker_type_id: CheckerType::Manual,
                is_outdated: false,
                check_test_versions: false,
                check_binary_files: false,
                auto_check_enabled: true,
                license_id: None,
                language_id: None,
            };
            db.insert_software(&new_sw).map_err(|e| e.to_string())?
        };
        let aur_info = AurInfo {
            software_id,
            pkgdesc: pkg.pkgdesc.clone(),
            aur_version: pkg.version.clone(),
            license_id: None,
            last_updated: None,
            depends: pkg
                .depends
                .as_ref()
                .map(|d| serde_json::to_string(d).unwrap_or_default()),
            makedepends: pkg
                .makedepends
                .as_ref()
                .map(|d| serde_json::to_string(d).unwrap_or_default()),
            optdepends: pkg
                .optdepends
                .as_ref()
                .map(|d| serde_json::to_string(d).unwrap_or_default()),
            out_of_date: pkg.out_of_date,
        };
        let _ = db.upsert_aur_info(&aur_info);
    }
    info!("已从 AUR 同步 {} 个软件包", packages.len());
    Ok(packages.len() as i64)
}

/// 从 PKGBUILD 文件同步软件包
#[tauri::command]
pub async fn sync_from_pkgbuild(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    pkgname: Option<String>,
) -> Result<i64, String> {
    info!("正在从 PKGBUILD 文件同步软件包");
    let aur_dir = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_setting("aur_packages_dir")
            .map_err(|e| e.to_string())?
            .map(|s| s.value)
            .unwrap_or_default()
    };
    if aur_dir.is_empty() {
        return Err("AUR files directory not configured".to_string());
    }
    let path = std::path::Path::new(&aur_dir);

    // 先收集所有目录
    let mut dir_entries = Vec::new();
    let mut entries = tokio::fs::read_dir(&path).await.map_err(|e| e.to_string())?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        if entry.file_type().await.map_err(|e| e.to_string())?.is_dir() {
            if let Some(ref filter_name) = pkgname {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if dir_name != *filter_name {
                    continue;
                }
            }
            dir_entries.push(entry);
        }
    }

    let total = dir_entries.len();
    info!("找到 {} 个软件包目录待同步", total);

    // 发送初始进度（总数）
    let _ = app.emit("sync-progress", serde_json::json!({
        "current": 0,
        "total": total,
        "pkgname": "",
        "message": format!("开始同步，共 {} 个包", total),
    }));

    let mut count = 0i64;
    for (i, entry) in dir_entries.iter().enumerate() {
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let pkg_path = entry.path();

        // 发送开始处理事件
        let _ = app.emit("sync-progress", serde_json::json!({
            "current": i,
            "total": total,
            "pkgname": dir_name,
            "message": format!("[{}/{}] 正在处理: {}", i + 1, total, dir_name),
        }));

        match aur::read_pkgbuild(&pkg_path).await {
            Ok(Some((sw, _))) => {
                let pkg_type = match sw.package_type_id.as_id() {
                    2 => "二进制包",
                    3 => "Git",
                    4 => "AppImage",
                    _ => "编译安装",
                };
                info!(
                    "[{}/{}] {} - 类型: {}, 自动检查: {}, 检查测试版: {}, 检查二进制: {}",
                    i + 1, total, sw.pkgname, pkg_type,
                    sw.auto_check_enabled, sw.check_test_versions, sw.check_binary_files
                );

                let db = state.db.lock().map_err(|e| e.to_string())?;
                let _ = db.upsert_software(&sw);
                count += 1;

                // 发送完成单个包事件
                let _ = app.emit("sync-progress", serde_json::json!({
                    "current": i + 1,
                    "total": total,
                    "pkgname": sw.pkgname,
                    "message": format!("[{}/{}] 已完成: {}", i + 1, total, dir_name),
                }));
            }
            Ok(None) => {
                info!("[{}/{}] {} - 跳过（无 PKGBUILD 文件）", i + 1, total, dir_name);
                let _ = app.emit("sync-progress", serde_json::json!({
                    "current": i + 1,
                    "total": total,
                    "pkgname": dir_name,
                    "message": format!("[{}/{}] 跳过: {} (无 PKGBUILD)", i + 1, total, dir_name),
                }));
            }
            Err(e) => {
                error!("[{}/{}] {} - 解析失败: {}", i + 1, total, dir_name, e);
                let _ = app.emit("sync-progress", serde_json::json!({
                    "current": i + 1,
                    "total": total,
                    "pkgname": dir_name,
                    "message": format!("[{}/{}] 失败: {} ({})", i + 1, total, dir_name, e),
                }));
            }
        }
    }

    // 发送完成事件
    let _ = app.emit("sync-progress", serde_json::json!({
        "current": total,
        "total": total,
        "pkgname": "",
        "message": format!("同步完成，成功 {} 个", count),
    }));

    info!("已从 PKGBUILD 文件同步 {} 个软件包", count);
    Ok(count)
}

/// 更新指定软件包的 AUR 信息
#[tauri::command]
pub async fn update_aur_info(
    state: State<'_, AppState>,
    pkgname_list: Option<Vec<String>>,
) -> Result<i64, String> {
    info!("正在更新软件包的 AUR 信息");
    let pkgnames: Vec<String> = if let Some(list) = pkgname_list {
        list
    } else {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_all_software()
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|s| s.pkgname)
            .collect()
    };
    let client = reqwest::Client::new();
    let mut count = 0i64;
    for pkgname in &pkgnames {
        if let Ok(Some(data)) = aur::get_package_info(&client, pkgname).await {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let sw = db.get_software_by_name(pkgname).map_err(|e| e.to_string())?;
            if let Some(existing) = sw {
                if let Some(sid) = existing.software_id {
                    let info = AurInfo {
                        software_id: sid,
                        pkgdesc: data["Description"].as_str().map(|s| s.to_string()),
                        aur_version: data["Version"].as_str().map(|s| s.to_string()),
                        license_id: None,
                        last_updated: data["LastModified"].as_i64(),
                        depends: data["Depends"]
                            .as_array()
                            .map(|a| serde_json::to_string(a).unwrap_or_default()),
                        makedepends: data["MakeDepends"]
                            .as_array()
                            .map(|a| serde_json::to_string(a).unwrap_or_default()),
                        optdepends: data["OptDepends"]
                            .as_array()
                            .map(|a| serde_json::to_string(a).unwrap_or_default()),
                        out_of_date: data["OutOfDate"].as_i64().map(|v| v != 0),
                    };
                    let _ = db.upsert_aur_info(&info);
                    count += 1;
                }
            }
        }
    }
    info!("已更新 {} 个软件包的 AUR 信息", count);
    Ok(count)
}
