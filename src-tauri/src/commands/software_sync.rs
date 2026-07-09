use log::{debug, info};
use tauri::State;

use crate::aur;
use crate::errors::{AppError, AppResult};
use crate::models::*;
use crate::AppState;

#[tauri::command]
pub async fn sync_from_aur(state: State<'_, AppState>) -> AppResult<i64> {
    info!("正在从 AUR 同步软件包");
    let username = {
        let db = state.db.lock()?;
        db.get_setting("aur_username")?
            .map(|s| s.value)
            .unwrap_or_default()
    };
    if username.is_empty() {
        return Err(AppError::ConfigError("AUR 用户名未配置".to_string()));
    }
    let client = reqwest::Client::new();
    let packages = aur::fetch_packages_by_user(&client, &username).await?;
    info!("获取到 {} 个软件包的原始数据", packages.len());
    let db = state.db.lock()?;
    for pkg in &packages {
        debug!("处理软件包: {}", pkg.pkgname);
        debug!("  AUR 返回字段: pkgdesc={:?}, version={:?}, license={:?}, last_modified={:?}",
            pkg.pkgdesc, pkg.version, pkg.license, pkg.last_modified);
        debug!("  depends={:?}, makedepends={:?}, optdepends={:?}, out_of_date={:?}",
            pkg.depends, pkg.makedepends, pkg.optdepends, pkg.out_of_date);

        let sw = db.get_software_by_name(&pkg.pkgname)?;
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
                language_id: None,
                version_extract_regex: None,
            };
            db.insert_software(&new_sw)?
        };

        let license_id = pkg.license.as_deref()
            .and_then(|lic| {
                debug!("  查找 License spdx_id='{}'", lic);
                let result = db.get_license_by_spdx_id(lic).ok();
                match &result {
                    Some(Some(e)) => debug!("  找到 License: id={:?}, spdx_id={}", e.id, e.spdx_id),
                    Some(None) => debug!("  未找到匹配的 License: spdx_id='{}'", lic),
                    None => debug!("  查询 License 出错"),
                }
                result
            })
            .flatten()
            .and_then(|e| {
                debug!("  最终 license_id={:?}", e.id);
                e.id
            });
        debug!("  license_id 解析结果: {:?}", license_id);

        let depends = pkg.depends.as_ref().map(|d| serde_json::to_string(d).unwrap_or_default());
        let makedepends = pkg.makedepends.as_ref().map(|d| serde_json::to_string(d).unwrap_or_default());
        let optdepends = pkg.optdepends.as_ref().map(|d| serde_json::to_string(d).unwrap_or_default());
        debug!("  depends序列化: {:?}", depends);
        debug!("  makedepends序列化: {:?}", makedepends);
        debug!("  optdepends序列化: {:?}", optdepends);

        let aur_info = AurInfo {
            software_id,
            pkgdesc: pkg.pkgdesc.clone(),
            aur_version: pkg.version.clone(),
            license_id,
            last_updated: pkg.last_modified,
            depends,
            makedepends,
            optdepends,
            out_of_date: pkg.out_of_date,
        };
        debug!("  写入 aur_info: software_id={}, license_id={:?}, depends={:?}, makedepends={:?}, optdepends={:?}, out_of_date={:?}",
            aur_info.software_id, aur_info.license_id, aur_info.depends, aur_info.makedepends, aur_info.optdepends, aur_info.out_of_date);
        let _ = db.upsert_aur_info(&aur_info);
        if let Ok(Some(stored)) = db.get_aur_info(aur_info.software_id) {
            debug!("  upsert后验证: license_id={:?}", stored.license_id);
        }
    }
    info!("已从 AUR 同步 {} 个软件包", packages.len());
    Ok(packages.len() as i64)
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
    let client = reqwest::Client::new();
    let mut count = 0i64;
    for pkgname in &pkgnames {
        debug!("请求 AUR 信息: {}", pkgname);
        if let Ok(Some(data)) = aur::get_package_info(&client, pkgname).await {
            debug!("AUR API 原始返回: {}", serde_json::to_string_pretty(&data).unwrap_or_default());

            let desc = data["Description"].as_str().map(|s| s.to_string());
            let version = data["Version"].as_str().map(|s| s.to_string());
            let last_modified = data["LastModified"].as_i64();
            let license_arr = data["License"].as_array();
            let license_str = license_arr.and_then(|a| a.first()).and_then(|v| v.as_str());
            let depends_arr = data["Depends"].as_array();
            let makedepends_arr = data["MakeDepends"].as_array();
            let optdepends_arr = data["OptDepends"].as_array();
            let out_of_date_val = data["OutOfDate"].as_i64();

            debug!("  解析字段: Description={:?}, Version={:?}, LastModified={:?}", desc, version, last_modified);
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
