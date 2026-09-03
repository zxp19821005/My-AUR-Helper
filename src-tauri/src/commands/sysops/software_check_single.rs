//! software_check_single.rs — 单软件包上游版本检查命令
//!
//! 与「选中检查」共用同一套核心流程（`software_check::selected`），
//! 确保 GitHub 缓存校验、批量并行检查、缓存写盘逻辑在所有入口完全一致，
//! 避免缓存逻辑分散到多处后再次遗漏。
use log::info;
use tauri::State;

use super::software_check::selected::check_selected_upstream;
use crate::errors::{AppError, AppResult};
use crate::AppState;

/// 检查单个软件包的上游版本
///
/// 先校验包存在且已有 AUR 信息，再复用选中检查流程：
/// 内部已包含 GitHub 仓库级缓存校验（命中则不发起任何网络请求）、
/// 批量并行检查、检查成功后的缓存写盘与结果入库。
#[tauri::command]
pub async fn check_upstream_version(
    state: State<'_, AppState>,
    pkgname: String,
) -> AppResult<String> {
    info!("正在检查上游版本: {}", pkgname);

    // 提前校验：无 AUR 版本时核心流程会静默跳过该包，这里给出明确错误
    {
        let db = state.db.lock()?;
        let sw = db
            .get_software_by_name(&pkgname)?
            .ok_or_else(|| AppError::PackageNotFound(pkgname.clone()))?;
        let has_aur_version = db
            .get_aur_info(sw.software_id.unwrap_or(0))?
            .and_then(|a| a.aur_version)
            .filter(|v| !v.is_empty())
            .is_some();
        if !has_aur_version {
            return Err(AppError::VersionCheckError(format!(
                "请先获取 {} 的 AUR 信息",
                pkgname
            )));
        }
    }

    let results = check_selected_upstream(state, vec![pkgname.clone()]).await?;
    results
        .into_iter()
        .next()
        .map(|(_, version)| version)
        .filter(|v| !v.is_empty() && v != "manual")
        .ok_or_else(|| AppError::VersionCheckError(format!("无法确定 {} 的上游版本", pkgname)))
}
