/**
 * software_sync/aur/collect.rs - AUR 同步结果收集与缺包分类
 *
 * 功能：把 AUR 查询返回的 Value 列表按包名索引，逐包解析字段构造
 * AurSyncResult；对 RPC/归档未返回的包单独分类为「同步失败」标记项。
 *
 * 工作流程：
 * 1. 以 data["Name"] 建立 pkgname -> Value 索引
 * 2. 遍历 pkgnames：命中则 parse_aur_fields 并读取既有软件记录组装结果
 * 3. 未命中则记录 software_id / 包名，供 persist 阶段打失败标记
 *
 * 适用场景：sync_from_aur 的中间收集阶段，减少数据库锁竞争。
 */
use std::collections::HashMap;

use log::debug;
use serde_json::Value;

use crate::commands::sysops::software_sync::utils::{parse_aur_fields, AurSyncResult};
use crate::errors::AppResult;
use crate::AppState;

/// 收集阶段产物：可写库结果 + 缺包标记项
pub(super) struct CollectedSync {
    /// 成功解析、待写入 aur_info 的结果
    pub(super) results: Vec<AurSyncResult>,
    /// AUR 未返回包的 software_id
    pub(super) missing_ids: Vec<i64>,
    /// AUR 未返回包的包名（仅用于日志）
    pub(super) missing_names: Vec<String>,
}

/// 按包名收集 AUR 同步结果，并对未返回包分类。
///
/// # 参数
/// - `state`: Tauri 应用状态（含数据库连接）
/// - `pkgnames`: 待同步的包名列表
/// - `aur_results`: AUR 归档或 RPC 返回的包信息列表
///
/// # 返回
/// - `Ok(CollectedSync)`: 待写库结果与缺包标记项
/// - `Err(AppError)`: 数据库查询失败
pub(super) fn collect_sync_results(
    state: &AppState,
    pkgnames: &[String],
    aur_results: &[Value],
) -> AppResult<CollectedSync> {
    let mut pkgname_to_data = HashMap::new();
    for data in aur_results {
        if let Some(name) = data["Name"].as_str() {
            pkgname_to_data.insert(name.to_string(), data.clone());
        }
    }

    let mut results: Vec<AurSyncResult> = Vec::new();
    // AUR RPC 未返回的包名：视为同步失败，需打错误标记（此前静默跳过，
    // 导致「AUR 中查不到」与「同步成功但无版本」在列表里无法区分）
    let mut missing_ids: Vec<i64> = Vec::new();
    let mut missing_names: Vec<String> = Vec::new();

    for pkgname in pkgnames {
        let data = match pkgname_to_data.get(pkgname) {
            Some(d) => d,
            None => {
                let db = state.db.lock()?;
                if let Some(existing) = db.get_software_by_name(pkgname)? {
                    if let Some(sid) = existing.software_id {
                        missing_ids.push(sid);
                        missing_names.push(pkgname.clone());
                    }
                }
                continue;
            }
        };
        debug!("处理软件包: {}", pkgname);

        let fields = parse_aur_fields(data);

        let db = state.db.lock()?;
        let sw = db.get_software_by_name(pkgname)?;
        if let Some(existing) = sw {
            if let Some(sid) = existing.software_id {
                // AUR 同步只更新 aur_info 表，不更新 software_info 表
                // software_info 的字段（上游URL、检查器类型、包类型等）只在用户手动设置时更新
                results.push(AurSyncResult {
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

    Ok(CollectedSync {
        results,
        missing_ids,
        missing_names,
    })
}
