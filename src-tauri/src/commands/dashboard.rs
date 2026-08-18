/**
 * dashboard.rs - 仪表盘统计 Tauri 命令
 *
 * 提供 get_dashboard_stats 命令：一次 IPC 返回所有模块计数，
 * 供 Dashboard 页面加载使用（替代前端分别全量拉取各表数据）。
 */
use log::debug;
use tauri::State;

use crate::models::DashboardStats;
use crate::AppState;

/// 获取仪表盘统计
///
/// 内部通过 COUNT(*) 聚合查询各表，返回全部模块计数，
/// 相比前端全量拉取列表数据开销可忽略。
/// @param state - 应用状态（含数据库连接）
/// @returns 各模块计数（DashboardStats）
#[tauri::command]
pub async fn get_dashboard_stats(
    state: State<'_, AppState>,
) -> Result<DashboardStats, String> {
    debug!("正在获取仪表盘统计");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let stats = db.get_dashboard_stats().map_err(|e| e.to_string())?;
    debug!(
        "仪表盘统计获取成功: 软件包总数={}, 有更新={}, 代理激活={}/{}, 备份={}, 缓存={}, License={}, 语言={}",
        stats.pkg_total, stats.pkg_outdated, stats.proxy_active, stats.proxy_total,
        stats.backup_total, stats.cache_total, stats.license_total, stats.language_total
    );
    Ok(stats)
}
