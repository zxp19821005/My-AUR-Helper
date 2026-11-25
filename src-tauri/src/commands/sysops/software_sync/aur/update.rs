/**
 * software_sync/aur/update.rs - 并行更新指定软件包 AUR 信息命令
 *
 * 功能：从 aur 模块整体迁入 update_aur_info 命令，保持函数签名与
 * #[tauri::command] + #[specta::specta] 双宏不变。
 * 从 AUR RPC API（归档优先，失败降级）批量取回信息并批量写库，
 * 同时重算 software_info.is_outdated。
 *
 * 工作流程：
 * 1. 取待更新包名（指定列表或全部）与超时、归档偏好
 * 2. 调 get_packages_info_smart 批量查询并按 Name 映射
 * 3. 单次加锁遍历结果，upsert aur_info 并按需重算 is_outdated
 *
 * 适用场景：前端「更新选中/全部 AUR 信息」命令，对外路径与 bindings 不变。
 */
use log::{info, warn};
use tauri::State;

use crate::aur;
use crate::commands::sysops::proxy_utils::build_client;
use crate::commands::sysops::software_sync::utils::{
    parse_aur_fields, read_archive_preference, read_http_timeout,
};
use crate::errors::AppResult;
use crate::models::*;
use crate::AppState;

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
#[specta::specta]
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
    let timeout = {
        let db = state.db.lock()?;
        read_http_timeout(&db)
    };
    let client = build_client(timeout, false);

    // 批量获取 AUR 信息：默认一次归档请求 + 本地过滤，失败自动降级到 RPC v5。
    // 旧实现逐包 tokio::spawn 打 info，包多时易触发 4000 req/day/IP 限流与 Anubis 拦截。
    let prefer_archive = {
        let db = state.db.lock()?;
        read_archive_preference(&db)
    };
    let results: Vec<(String, serde_json::Value)> =
        aur::get_packages_info_smart(&client, &pkgnames, 100, 0, prefer_archive)
            .await?
            .into_iter()
            .filter_map(|data| {
                data["Name"]
                    .as_str()
                    .map(|name| (name.to_string(), data.clone()))
            })
            .collect();

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
                    // 更新成功，清空上一次失败的标记
                    last_sync_error: None,
                };
                if let Err(e) = db.upsert_aur_info(&info) {
                    errors.push(format!("更新 {} 的 AUR 信息失败: {}", pkgname, e));
                }
                // AUR 版本已变更：与已存的上游版本重新比较，重算 software_info.is_outdated。
                // 修复：此前 AUR 同步只写 aur_info，不重算该标记，导致用户更新 AUR 后
                // is_outdated 仍停留于旧值（stale），出现「已是最新却仍显示需更新」。
                // 仅在 AUR 与上游版本都就绪时才重算，避免无依据地翻转标记。
                if let (Some(aur_ver), Some(up_ver)) = (
                    info.aur_version.as_deref(),
                    db.get_upstream_info(sid)
                        .ok()
                        .flatten()
                        .and_then(|u| u.upstream_version)
                        .filter(|s| !s.is_empty())
                        .as_deref(),
                ) {
                    let flagged = fields.out_of_date.unwrap_or(false);
                    let is_outdated = crate::versions::is_outdated_with_aur_signal(
                        aur_ver, up_ver, flagged, pkgname,
                    );
                    if let Err(e) = db.update_software_outdated(sid, is_outdated) {
                        errors.push(format!("重算 {} 的 is_outdated 失败: {}", pkgname, e));
                    }
                }
                count += 1;
            }
        }
    }

    if !errors.is_empty() {
        warn!(
            "[update_aur_info] 部分写入失败 ({} 个): {:?}",
            errors.len(),
            errors
        );
    }
    info!("已更新 {} 个软件包的 AUR 信息", count);
    Ok(count)
}
