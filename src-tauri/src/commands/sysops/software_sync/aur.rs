/**
 * aur.rs - AUR 信息同步和更新命令（编排入口）
 *
 * 功能：从 AUR RPC API / 全量元数据归档批量同步软件包信息，或更新指定包的 AUR 信息。
 * 本文件仅保留命令编排骨架与子模块声明；读设置 / 收集 / 写库分别下沉到
 * config.rs / collect.rs / persist.rs，update_aur_info 下沉到 update.rs。
 *
 * 工作流程（sync_from_aur）：
 * 1. config::read_sync_settings 读设置并校验用户名
 * 2. 取包名列表 → aur::get_packages_info_smart 批量查询（归档优先，失败降级 RPC）
 * 3. collect::collect_sync_results 收集结果并对缺包分类
 * 4. persist::persist_sync_results 批量写库 + 失败标记 + 重算 is_outdated
 *
 * 注意：AUR 同步只更新 aur_info 表（描述、版本、依赖等），不更新 software_info 表。
 * software_info 的字段（上游URL、检查器类型、包类型等）只在用户手动设置时更新。
 */
mod collect;
mod config;
mod persist;
mod update;

use log::{debug, info};
use tauri::State;

use super::super::proxy_utils::build_client;
use crate::aur;
use crate::errors::AppResult;
use crate::AppState;

// 保持对外路径 commands::sysops::software_sync::aur::update_aur_info 不变。
// 必须用 glob 再导出：tauri/specta 双宏还会在子模块生成 __cmd__ / __specta__fn__
// 等隐藏符号，collect_commands! 按完整路径解析它们，单项 pub use 无法带出。
pub use update::*;

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
#[specta::specta]
pub async fn sync_from_aur(state: State<'_, AppState>) -> AppResult<i64> {
    info!("正在从 AUR 同步软件包");

    // 1. 读设置并校验用户名
    let settings = {
        let db = state.db.lock()?;
        config::read_sync_settings(&db)?
    };
    config::require_username(&settings.username)?;
    let client = build_client(settings.timeout, false);

    // 2. 取包名列表
    let pkgnames = {
        let db = state.db.lock()?;
        db.get_all_software()?
            .into_iter()
            .map(|s| s.pkgname)
            .collect::<Vec<String>>()
    };
    info!("准备同步 {} 个软件包的 AUR 信息", pkgnames.len());

    // 默认走全量元数据归档（一次请求 + 本地过滤），失败自动降级到 RPC v5 批量查询
    let aur_results = aur::get_packages_info_smart(
        &client,
        &pkgnames,
        settings.batch_size.min(100),
        settings.batch_interval,
        settings.prefer_archive,
    )
    .await?;
    debug!("批量查询返回 {} 条结果", aur_results.len());

    // 3. 收集结果 + 缺包分类
    let collected = collect::collect_sync_results(&state, &pkgnames, &aur_results)?;

    // 4. 批量写库 + 失败标记 + 重算 is_outdated
    let count = persist::persist_sync_results(
        &state,
        &collected.results,
        &collected.missing_ids,
        &collected.missing_names,
    )?;
    info!("已从 AUR 同步 {} 个软件包", count);
    Ok(count)
}
