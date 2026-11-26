/**
 * software_sync/aur/engine.rs - AUR 同步公共同步引擎（包名解析 + 单包写入）
 *
 * 功能：把原先散落在 sync_from_aur / sync_packages_by_maintainer 两条链路里的
 * 通用步骤收敛到一处，供「从 AUR 同步」全量对齐与「按维护者同步」共用：
 * - resolve_pkgnames：解析待同步包名集合 = 本地已有 ∪（可选）该用户名下维护，
 *   用 HashSet 去重；用户名为空时降级为仅本地，不报错。
 * - merge_pkgnames：纯函数版并集去重，便于单测与复用。
 * - upsert_one：单包写入（存在则更新非上游字段，不存在则新建且上游地址留空），
 *   逻辑等价迁移自 maintainer_sync::write_one_package。
 *
 * 为什么 DB 参数是 Mutex 而不是 &Database：
 * resolve_pkgnames 内部既要做 DB 读取又要发网络请求，若接收 &Database，调用方
 * 就必须在整个 await 期间持有 std::sync::MutexGuard（!Send），导致 Tauri
 * async 命令的 future 不满足 Send 约束而无法编译。改为传入 &Mutex<Database>，
 * 由本函数在内部完成读取后立即释放锁，再发起网络请求。
 *
 * 使用场景：AUR 全量对齐（sync_from_aur）、按维护者同步（maintainer_sync）。
 */
use std::collections::HashSet;
use std::sync::Mutex;

use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::commands::sysops::software_sync::utils::{detect_package_defaults, parse_aur_fields};
use crate::db::Database;
use crate::errors::AppResult;
use crate::models::{AurInfo, SoftwareInfo};

/// 「从 AUR 同步」全量对齐的结果统计
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncFromAurResult {
    /// 成功写入（新建 + 更新）的包数量
    pub synced: i64,
    /// 本地新建的包数量
    pub created: i64,
    /// 失败的包数量（写入失败或 AUR 未返回）
    pub failed: i64,
}

/// 解析待同步的包名集合：本地已有 ∪（可选）该用户名下维护的包。
///
/// - 始终并入本地已有包名（`db.get_all_software()`）；
/// - `include_maintained=true` 时读取 `aur_username` 并调 AUR search 并入其包名；
///   用户名为空则跳过（不报错，降级为仅本地）；
/// - 用 `HashSet` 去重后返回，顺序不保证。
///
/// # 参数
/// - `client`: 已配置超时 / User-Agent 的 HTTP 客户端
/// - `db`: 数据库连接（Mutex 保护，函数内部自行加锁并在网络请求前释放）
/// - `include_maintained`: 是否并入当前配置用户名下的维护包
///
/// # 返回
/// - `Ok(Vec<String>)`: 去重后的包名列表
/// - `Err(AppError)`: 数据库读取失败或 AUR search 失败
pub(in crate::commands::sysops::software_sync) async fn resolve_pkgnames(
    client: &Client,
    db: &Mutex<Database>,
    include_maintained: bool,
) -> AppResult<Vec<String>> {
    // DB 读取集中在此作用域内完成，出作用域即释放锁，避免跨 await 持锁
    let (names, username) = {
        let db = db.lock()?;
        let names = db
            .get_all_software()?
            .into_iter()
            .map(|s| s.pkgname)
            .collect::<HashSet<String>>();
        let username = db
            .get_setting("aur_username")?
            .map(|s| s.value)
            .unwrap_or_default();
        (names, username)
    };
    let local_total = names.len();

    let mut maintained_total = 0usize;
    let maintained_names = if include_maintained && !username.trim().is_empty() {
        // search(maintainer) + search(comaintainers)，两次请求拿全去重包名
        let maintained = crate::aur::fetch_packages_by_user(client, username.trim()).await?;
        maintained_total = maintained.len();
        maintained
            .into_iter()
            .map(|p| p.pkgname)
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    let names = merge_pkgnames(names, maintained_names);
    if include_maintained && username.trim().is_empty() {
        info!(
            "[AUR 同步] 未配置 aur_username，仅同步本地 {} 个包",
            local_total
        );
    } else {
        info!(
            "[AUR 同步] 包名集合：本地 {} 个 + 维护者 {} 个，去重后 {} 个",
            local_total,
            maintained_total,
            names.len()
        );
    }
    Ok(names)
}

/// 合并本地包名集合与维护者包名列表（纯函数，去重）。
///
/// # 参数
/// - `local`: 本地已有包名集合
/// - `maintained`: 维护者名下包名列表
///
/// # 返回
/// - 去重后的包名列表（顺序不保证）
pub(in crate::commands::sysops::software_sync) fn merge_pkgnames(
    mut local: HashSet<String>,
    maintained: Vec<String>,
) -> Vec<String> {
    for name in maintained {
        local.insert(name);
    }
    local.into_iter().collect()
}

/// 将单个 AUR 包写入数据库（存在则更新非上游字段，不存在则新建且上游留空）。
///
/// 等价迁移自 `maintainer_sync::write_one_package`：
/// - `parse_aur_fields` + `detect_package_defaults` 推断字段；
/// - `upsert_software_from_aur`：新建包 `upstream_url` 强制留空，已存在包保留原值；
/// - `upsert_aur_info`：刷新描述 / 版本 / 依赖等 AUR 字段并清空失败标记；
/// - `recompute_outdated_from_aur`：按新 AUR 版本重算 is_outdated（失败仅告警不中断）。
///
/// # 参数
/// - `db`: 数据库连接（调用方持有锁）
/// - `pkgname`: 包名
/// - `data`: AUR JSON 数据（完整字段或 search 基础字段）
///
/// # 返回
/// - `Ok(true)`: 新建；`Ok(false)`: 更新
/// - `Err(AppError)`: 写库失败
pub(in crate::commands::sysops::software_sync) fn upsert_one(
    db: &Database,
    pkgname: &str,
    data: &serde_json::Value,
) -> AppResult<bool> {
    let f = parse_aur_fields(data);
    let (package_type, checker_type, check_test_versions, check_binary_files) =
        detect_package_defaults(pkgname);
    let aur_flagged = f.out_of_date.unwrap_or(false);

    let sw = SoftwareInfo {
        software_id: None,
        pkgname: pkgname.to_string(),
        // 上游地址：新建分支会被强制置空，更新分支不会触及此字段
        upstream_url: None,
        package_type_id: package_type,
        checker_type_id: checker_type,
        is_outdated: false,
        check_test_versions,
        check_binary_files,
        auto_check_enabled: true,
        skip_check_upstream: false,
        language_ids: Vec::new(),
        version_extract_regex: None,
    };
    let (sid, created) = db.upsert_software_from_aur(&sw)?;

    let aur_info = AurInfo {
        software_id: sid,
        pkgdesc: f.desc,
        aur_version: f.version,
        license_id: f.license_spdx,
        last_updated: f.last_modified,
        depends: f.depends,
        makedepends: f.makedepends,
        optdepends: f.optdepends,
        out_of_date: f.out_of_date,
        // 命中即同步成功，清空上一次留下的失败标记
        last_sync_error: None,
    };
    db.upsert_aur_info(&aur_info)?;

    // AUR 版本已同步：与上游版本重新比较，重算 software_info.is_outdated。
    // 三条同步链路（sync_from_aur / update_aur_info / 维护者同步）共用同一判定。
    // 新建包没有上游版本，重算会自动跳过，is_outdated 保持默认 false。
    if let Err(e) =
        db.recompute_outdated_from_aur(sid, aur_info.aur_version.as_deref(), aur_flagged, pkgname)
    {
        warn!("[AUR 同步] 重算 {} 的 is_outdated 失败: {}", pkgname, e);
    }
    Ok(created)
}

#[cfg(test)]
mod engine_tests;
