/**
 * list.rs - 软件更新工作台列表命令
 *
 * 功能：仅取出「过期」标记（software_info.is_outdated = 1）的软件包，
 *      结合本地 PKGBUILD 实际 pkgver 推导写回状态，返回给「软件更新」页面展示。
 *
 * 设计说明：列表内容完全由版本检查链路已落库的 is_outdated 决定，本命令
 * 不在浏览时重新执行 AUR / 上游 vercmp 比较，也不扫描未过期的软件包。
 * 浏览阶段仅对上游版本做一次静默归一化（UpstreamVersion::normalized，无日志），
 * 不调用带逐条日志的 parse，也不输出任何逐包 DEBUG。
 * 状态不落库：写回本地 PKGBUILD 后 AUR 现网版本并未改变，因此不能用 aur_version
 * 判断「是否已处理」，改以磁盘实际 pkgver 与目标版本做字面相等判断，
 * 区分 pending（待写回）与 local_updated（已写回 · 待推送 AUR）。
 * 本命令全程只读，不改动任何文件与数据库。
 *
 * 适用场景：软件更新页面首次加载、写回后刷新、工具栏刷新按钮。
 */
use std::path::PathBuf;

use tauri::State;

use crate::errors::{AppError, AppResult};
use crate::models::{SoftwareUpdateItem, SoftwareUpdateStatus};
use crate::versions::UpstreamVersion;
use crate::AppState;

use super::pkgbuild_io::{read_local_pkgbuild, LocalPkgbuild};

/// 依据本地磁盘 pkgver 推导写回状态（不做任何版本比较，也不输出日志）
///
/// 列表已由 `is_outdated = 1` 筛出，因此这里只回答「本地是否已经写回」，
/// 不重新判定「是否需要更新」。写回时实际写入 PKGBUILD 的 pkgver 就是
/// `target_version`，故直接做字面相等判断即可，无需再归一化本地值——
/// 既省掉每包一次冗余解析，也比二次归一化更严格。
///
/// # 参数
/// - `upstream_raw`: 上游原始版本串
/// - `target_version`: 上游静默归一化后的目标版本（写回实际写入 pkgver 的值）
/// - `local`: 本地 PKGBUILD 解析结果，None 表示文件缺失
///
/// # 返回
/// - `SoftwareUpdateStatus`: 推导出的状态枚举（不含 UpToDate）
fn derive_status(
    upstream_raw: &str,
    target_version: &str,
    local: Option<&LocalPkgbuild>,
) -> SoftwareUpdateStatus {
    if upstream_raw.trim().is_empty() {
        return SoftwareUpdateStatus::NoUpstream;
    }
    let Some(local) = local else {
        return SoftwareUpdateStatus::Missing;
    };
    if local.pkgver.is_empty() {
        return SoftwareUpdateStatus::Missing;
    }
    // 写回写入的就是 target_version，字面相等即表示已写回
    if local.pkgver.trim() == target_version {
        SoftwareUpdateStatus::LocalUpdated
    } else {
        SoftwareUpdateStatus::Pending
    }
}

/// 列出「过期」标记为 1 的软件包的写回状态（只读）
///
/// # 参数
/// - `state`: Tauri 应用状态，提供数据库连接与设置项
///
/// # 返回
/// - `Vec<SoftwareUpdateItem>`: 按包名升序的列表行，含版本差、本地 pkgver 与状态
/// - `Err(AppError::ConfigError)`: 未配置 `aur_packages_dir`，无法定位本地 PKGBUILD
#[tauri::command]
pub async fn list_software_updates(
    state: State<'_, AppState>,
) -> AppResult<Vec<SoftwareUpdateItem>> {
    let (rows, aur_dir) = {
        let db = state.db.lock()?;
        let rows = db.list_update_candidates()?;
        let dir = db
            .get_setting("aur_packages_dir")?
            .map(|s| s.value)
            .unwrap_or_default();
        (rows, dir)
    };
    if aur_dir.trim().is_empty() {
        return Err(AppError::ConfigError(
            "AUR 文件目录未配置，请先在设置中填写 aur_packages_dir".to_string(),
        ));
    }
    let root = PathBuf::from(&aur_dir);

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let upstream_raw = row.upstream_version.clone().unwrap_or_default();
        // 静默归一化：仅为展示/状态判定提供目标值，不输出逐包日志
        let target_version = UpstreamVersion::normalized(&upstream_raw);

        // 文件读取失败按「缺失」处理，不因单个异常中断整张列表
        let local: Option<LocalPkgbuild> = read_local_pkgbuild(&root, &row.pkgname)
            .await
            .ok()
            .flatten();
        let (local_pkgver, local_pkgrel, pkgbuild_path) = match &local {
            Some(l) => (
                Some(l.pkgver.clone()),
                Some(l.pkgrel.clone()),
                Some(l.path.to_string_lossy().to_string()),
            ),
            None => (None, None, None),
        };

        let status = derive_status(&upstream_raw, &target_version, local.as_ref());

        items.push(SoftwareUpdateItem {
            software_id: row.software_id,
            pkgname: row.pkgname,
            aur_version: row.aur_version,
            upstream_version: row.upstream_version,
            target_version,
            local_pkgver,
            local_pkgrel,
            pkgbuild_path,
            aur_out_of_date: row.out_of_date,
            status,
            last_checked: row.last_checked,
        });
    }
    Ok(items)
}
