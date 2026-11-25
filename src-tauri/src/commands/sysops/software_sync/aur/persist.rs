/**
 * software_sync/aur/persist.rs - AUR 同步结果批量写库与失败标记
 *
 * 功能：
 * - 把收集阶段的 AurSyncResult 批量 upsert 到 aur_info 表（只更新 aur_info）
 * - 命中即清空上次失败标记；写入失败则回写失败原因
 * - 同步重算 software_info.is_outdated（与 update_aur_info 同源逻辑）
 * - AUR 未返回的包只打失败标记，保留原有版本信息
 *
 * 工作流程：
 * 1. 单次加锁遍历 sync_results，逐条构造 AurInfo 并 upsert
 * 2. AUR 与上游版本均就绪时用 is_outdated_with_aur_signal 重算标记
 * 3. 缺包按 software_id 打「AUR 中未找到该包」失败标记
 * 4. 汇总失败数与计数，返回成功写入数量
 *
 * 适用场景：sync_from_aur 的写库阶段，减少数据库锁竞争。
 */
use log::warn;

use crate::commands::sysops::software_sync::utils::AurSyncResult;
use crate::errors::AppResult;
use crate::models::AurInfo;
use crate::AppState;

/// 批量写入 AUR 同步结果并标记缺包。
///
/// # 参数
/// - `state`: Tauri 应用状态（含数据库连接）
/// - `sync_results`: 收集阶段产出的待写库结果
/// - `missing_ids`: AUR 未返回包的 software_id 列表
/// - `missing_names`: AUR 未返回包的包名列表（仅用于日志）
///
/// # 返回
/// - `Ok(count)`: 成功处理的同步结果数量
/// - `Err(AppError)`: 数据库锁获取失败
pub(super) fn persist_sync_results(
    state: &AppState,
    sync_results: &[AurSyncResult],
    missing_ids: &[i64],
    missing_names: &[String],
) -> AppResult<i64> {
    let db = state.db.lock()?;
    let mut count = 0i64;
    let mut errors = Vec::new();
    for result in sync_results {
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
            // 命中即同步成功，清空上一次留下的失败标记
            last_sync_error: None,
        };
        if let Err(e) = db.upsert_aur_info(&aur_info) {
            // 写入失败本身也是同步失败，回写错误原因便于列表中筛选定位
            let _ = db.mark_aur_sync_error(result.software_id, "写入数据库失败");
            errors.push(format!("更新 {} 的 AUR 信息失败: {}", result.pkgname, e));
        }
        // 同步：AUR 版本已变更，重算 software_info.is_outdated（与 update_aur_info 同源修复）
        if let (Some(aur_ver), Some(up_ver)) = (
            result.version.as_deref(),
            db.get_upstream_info(result.software_id)
                .ok()
                .flatten()
                .and_then(|u| u.upstream_version)
                .filter(|s| !s.is_empty())
                .as_deref(),
        ) {
            let flagged = result.out_of_date.unwrap_or(false);
            let is_outdated = crate::versions::is_outdated_with_aur_signal(
                aur_ver,
                up_ver,
                flagged,
                &result.pkgname,
            );
            if let Err(e) = db.update_software_outdated(result.software_id, is_outdated) {
                errors.push(format!(
                    "重算 {} 的 is_outdated 失败: {}",
                    result.pkgname, e
                ));
            }
        }
        count += 1;
    }

    // AUR 中查不到的包：只打失败标记，保留原有版本信息
    for sid in missing_ids {
        if let Err(e) = db.mark_aur_sync_error(*sid, "AUR 中未找到该包") {
            errors.push(format!("标记 software_id={} 同步失败失败: {}", sid, e));
        }
    }
    if !missing_names.is_empty() {
        warn!(
            "[sync_from_aur] AUR 未返回 {} 个包，已标记同步失败: {:?}",
            missing_names.len(),
            missing_names
        );
    }

    if !errors.is_empty() {
        warn!(
            "[sync_from_aur] 部分写入失败 ({} 个): {:?}",
            errors.len(),
            errors
        );
    }
    Ok(count)
}
