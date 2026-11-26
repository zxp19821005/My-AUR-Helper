/**
 * maintainer_sync.rs - 按 AUR 维护者用户名批量同步软件包
 *
 * 功能：
 * - 从 AUR 按 maintainer + comaintainers 拉取该用户名下的全部包名
 * - 用元数据归档（或 RPC 批量 info）一次性补全完整字段
 * - 已存在的包：更新除上游地址以外的全部字段，保留用户配置的上游地址；
 *   is_outdated 依据新同步的 AUR 版本与上游版本重新比较得出
 * - 不存在的包：新建，上游地址留空（AUR 的 URL 是项目主页，不是版本检查地址）
 *
 * 工作流程：
 * 1. 读取配置（用户名、超时、批量参数、归档偏好）
 * 2. search 拉取去重包名列表（两次请求，避免逐包 info 触发限流）
 * 3. 批量补全完整字段（归档优先，失败降级 RPC 分批 info）
 * 4. 逐包经 engine::upsert_one 写入，收集新增/更新/失败统计
 *
 * 单包写入复用 aur/engine.rs 的公用实现（与「从 AUR 同步」全量对齐同源），
 * 本文件不再保留 write_one_package 副本。
 *
 * 使用场景：
 * - 用户在 AUR 维护大量包（上千个），需要本地一次性对齐时
 */
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use super::super::proxy_utils::build_client;
use super::aur::engine;
use super::utils::{parse_u32, parse_u64, read_archive_preference, read_http_timeout};
use crate::aur;
use crate::errors::{AppError, AppResult};
use crate::AppState;

/// 按维护者同步的结果统计
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MaintainerSyncResult {
    /// AUR 上 search 返回的去重包名总数
    pub aur_total: usize,
    /// 本地新建的包数量
    pub created: usize,
    /// 本地已存在并更新的包数量
    pub updated: usize,
    /// 写入失败的数量
    pub failed: usize,
    /// 未能补全完整字段的包名（可能已失效或被 AUR 删除）
    pub missing_detail: Vec<String>,
}

/// 按 AUR 维护者用户名批量同步软件包
///
/// 从配置读取 `aur_username`，拉取其维护/共同维护的全部包并对齐到本地数据库。
/// 已存在的包只更新 AUR 可推断字段并保留上游地址；不存在的包新建且上游地址留空。
///
/// # 参数
/// - `state`: Tauri 应用状态
///
/// # 返回
/// - `Ok(MaintainerSyncResult)`: 本次同步的统计数据
#[tauri::command]
#[specta::specta]
pub async fn sync_packages_by_maintainer(
    state: State<'_, AppState>,
) -> AppResult<MaintainerSyncResult> {
    // 读取配置后尽快释放 db 锁
    let (username, timeout, batch_size, batch_interval, prefer_archive) = {
        let db = state.db.lock()?;
        let username = db
            .get_setting("aur_username")?
            .map(|s| s.value)
            .unwrap_or_default();
        let timeout = read_http_timeout(&db);
        let batch_size = parse_u32(
            &db.get_setting("aur_batch_size")?
                .map(|s| s.value)
                .unwrap_or_default(),
            50,
        ) as usize;
        let batch_interval = parse_u64(
            &db.get_setting("aur_batch_interval")?
                .map(|s| s.value)
                .unwrap_or_default(),
            5,
        );
        let prefer_archive = read_archive_preference(&db);
        (
            username,
            timeout,
            batch_size,
            batch_interval,
            prefer_archive,
        )
    };
    if username.trim().is_empty() {
        return Err(AppError::ConfigError("AUR 维护者用户名未配置".to_string()));
    }
    let client = build_client(timeout, false);

    // 1. search 拉取去重包名（只两次请求，不逐包 info）
    info!("[维护者同步] 开始拉取用户 '{}' 维护的包列表", username);
    let base = aur::fetch_packages_by_user(&client, &username).await?;
    let pkgnames: Vec<String> = base.iter().map(|p| p.pkgname.clone()).collect();
    let aur_total = pkgnames.len();
    info!(
        "[维护者同步] AUR 共维护 {} 个包，开始补全完整字段",
        aur_total
    );

    // 2. 批量补全完整字段（归档优先，降级 RPC 分批）
    let full_results = aur::get_packages_info_smart(
        &client,
        &pkgnames,
        batch_size.min(100),
        batch_interval,
        prefer_archive,
    )
    .await?;

    // search 基础字段作为补全失败时的兜底（版本/描述/URL 仍可用）
    let mut by_name = std::collections::HashMap::new();
    for data in &full_results {
        if let Some(name) = data["Name"].as_str() {
            by_name.insert(name.to_string(), data.clone());
        }
    }

    // 3. 逐包写入（db 锁仅在此阶段持有）
    let mut result = MaintainerSyncResult {
        aur_total,
        created: 0,
        updated: 0,
        failed: 0,
        missing_detail: Vec::new(),
    };
    let db = state.db.lock()?;
    for (idx, pkg) in base.iter().enumerate() {
        // 优先用补全的 JSON；缺失时用 search 阶段的基础字段兜底
        let data = by_name.get(&pkg.pkgname).cloned().unwrap_or_else(|| {
            serde_json::json!({
                "Name": pkg.pkgname,
                "Description": pkg.pkgdesc,
                "Version": pkg.version,
                "URL": pkg.url,
                "OutOfDate": pkg.out_of_date.map(|v| if v { 1 } else { 0 }),
                "LastModified": pkg.last_modified,
            })
        });
        if !by_name.contains_key(&pkg.pkgname) {
            result.missing_detail.push(pkg.pkgname.clone());
        }

        // 写入复用全量对齐引擎：存在则更新非上游字段，不存在则新建且上游留空
        match engine::upsert_one(&db, &pkg.pkgname, &data) {
            Ok(created) => {
                if created {
                    result.created += 1;
                } else {
                    result.updated += 1;
                }
            }
            Err(e) => {
                result.failed += 1;
                warn!("[维护者同步] 写入 {} 失败: {}", pkg.pkgname, e);
            }
        }
        if (idx + 1) % 100 == 0 {
            debug!("[维护者同步] 已处理 {}/{} 个包", idx + 1, aur_total);
        }
    }

    info!(
        "[维护者同步] 完成：AUR {} 个，新建 {}，更新 {}，失败 {}，补全缺失 {}",
        result.aur_total,
        result.created,
        result.updated,
        result.failed,
        result.missing_detail.len()
    );
    Ok(result)
}
