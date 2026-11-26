/**
 * aur.rs - AUR 信息同步和更新命令（编排入口）
 *
 * 功能：从 AUR RPC API / 全量元数据归档批量同步软件包信息，或更新指定包的 AUR 信息。
 * 本文件仅保留命令编排骨架与子模块声明；读设置下沉到 config.rs，
 * 包名解析与单包写入下沉到 engine.rs，update_aur_info 下沉到 update.rs。
 *
 * 工作流程（sync_from_aur，一键全量对齐）：
 * 1. config::read_sync_settings 读设置（超时 / 批量参数 / 归档偏好）
 * 2. engine::resolve_pkgnames 取包名集合 = 本地已有 ∪ 该用户名下维护（去重）
 * 3. aur::get_packages_info_smart 批量查询（归档优先，失败降级 RPC）
 * 4. 单次持锁逐包 engine::upsert_one：命中即新建/更新，未命中打失败标记
 *
 * 与旧实现的差异：旧实现只取本地已有包名，且只更新 aur_info 表（不新增 software_info
 * 记录），因此「AUR 有、本地缺」的包永远补不进来。现在改为全量对齐：刷新本地已有包
 * 的同时补齐用户维护但本地缺失的包；已存在的包保留用户配置的上游地址，新建包上游留空。
 */
mod config;
pub(super) mod engine;
mod update;

use std::collections::HashMap;

use log::{debug, info, warn};
use serde_json::Value;
use tauri::State;

use super::super::proxy_utils::build_client;
use crate::aur;
use crate::errors::AppResult;
use crate::AppState;

// 保持对外路径 commands::sysops::software_sync::aur::update_aur_info 不变。
// 必须用 glob 再导出：tauri/specta 双宏还会在子模块生成 __cmd__ / __specta__fn__
// 等隐藏符号，collect_commands! 按完整路径解析它们，单项 pub use 无法带出。
pub use update::*;

// 全量对齐的结果统计类型（定义在 engine.rs，供前端 bindings 使用）
pub use engine::SyncFromAurResult;

/// 从 AUR 全量对齐软件包（刷新本地已有 + 补齐新维护的包）
///
/// 一次点击完成本地与「本地 ∪ 该用户名下维护」的对齐：
/// - 已存在的包：更新除上游地址以外的所有值（保留用户配置的 upstream_url）；
/// - 不存在的包：新建，且 upstream_url 留空（AUR 的 URL 是项目主页，不是版本检查地址）；
/// - 新建包没有上游版本，is_outdated 重算自动跳过。
///
/// # 参数
/// - `state`: Tauri 应用状态，包含数据库连接
///
/// # 返回
/// - `Ok(SyncFromAurResult)`: 成功写入数 / 新建数 / 失败数
/// - `Err(e)`: 同步过程中发生错误
#[tauri::command]
#[specta::specta]
pub async fn sync_from_aur(state: State<'_, AppState>) -> AppResult<SyncFromAurResult> {
    info!("正在从 AUR 全量对齐软件包");

    // 1. 读设置（超时 / 批量参数 / 归档偏好；用户名由 engine 自行读取）
    let settings = {
        let db = state.db.lock()?;
        config::read_sync_settings(&db)?
    };
    let client = build_client(settings.timeout, false);

    // 2. 包名集合 = 本地已有 ∪ 该用户名下维护（用户名为空则降级为仅本地）
    //    engine 内部先完成 DB 读取再释放锁，之后才发网络请求
    let pkgnames = engine::resolve_pkgnames(&client, &state.db, true).await?;
    info!("准备同步 {} 个软件包的 AUR 信息", pkgnames.len());

    // 3. 批量查询：默认走全量元数据归档（一次请求 + 本地过滤），失败降级 RPC v5 批量查询
    let aur_results = aur::get_packages_info_smart(
        &client,
        &pkgnames,
        settings.batch_size.min(100),
        settings.batch_interval,
        settings.prefer_archive,
    )
    .await?;
    debug!("批量查询返回 {} 条结果", aur_results.len());

    // 4. 按 Name 索引结果，单次持锁遍历包名集合
    let mut by_name: HashMap<String, Value> = HashMap::with_capacity(aur_results.len());
    for data in &aur_results {
        if let Some(name) = data["Name"].as_str() {
            by_name.insert(name.to_string(), data.clone());
        }
    }

    let mut synced = 0i64;
    let mut created = 0i64;
    let mut failed = 0i64;
    let mut missing_names: Vec<String> = Vec::new();
    {
        let db = state.db.lock()?;
        for pkgname in &pkgnames {
            let data = match by_name.get(pkgname) {
                Some(d) => d,
                None => {
                    // AUR 未返回：本地已存在的包打失败标记并保留原版本信息；
                    // 本地不存在的包（新增候选）无从标记，只计入失败。
                    // missing_names 统一记录「AUR 未返回」的包名，保证日志口径一致。
                    if let Some(existing) = db.get_software_by_name(pkgname)? {
                        if let Some(sid) = existing.software_id {
                            if let Err(e) = db.mark_aur_sync_error(sid, "AUR 中未找到该包") {
                                warn!("[sync_from_aur] 标记 {} 同步失败出错: {}", pkgname, e);
                            }
                        }
                    }
                    missing_names.push(pkgname.clone());
                    failed += 1;
                    continue;
                }
            };

            match engine::upsert_one(&db, pkgname, data) {
                Ok(true) => {
                    created += 1;
                    synced += 1;
                }
                Ok(false) => synced += 1,
                Err(e) => {
                    failed += 1;
                    warn!("[sync_from_aur] 写入 {} 失败: {}", pkgname, e);
                }
            }
        }
    }

    if !missing_names.is_empty() {
        warn!(
            "[sync_from_aur] AUR 未返回 {} 个包（本地已有的已打同步失败标记）: {:?}",
            missing_names.len(),
            missing_names
        );
    }
    info!(
        "[sync_from_aur] 完成：包名 {} 个，成功写入 {} 个（其中新建 {} 个），失败 {} 个",
        pkgnames.len(),
        synced,
        created,
        failed
    );
    Ok(SyncFromAurResult {
        synced,
        created,
        failed,
    })
}
