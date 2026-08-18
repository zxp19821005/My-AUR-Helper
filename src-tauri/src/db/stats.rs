/**
 * stats.rs - 仪表盘统计数据访问层
 *
 * 提供 get_dashboard_stats：通过 COUNT(*) 聚合查询一次返回所有模块计数，
 * 供仪表盘页面展示。避免前端为统计而全量拉取各表数据（软件包/代理/License 等）。
 *
 * 使用场景：
 * - Dashboard 页面加载时调用
 */
use crate::errors::AppResult;
use crate::models::DashboardStats;
use rusqlite::Connection;

use super::Database;

impl Database {
    /// 获取仪表盘统计（各表行数与条件计数）
    ///
    /// 全部为 COUNT(*) 聚合查询，无用户输入拼接，天然防注入。
    /// @returns 统计结果（见 DashboardStats 字段说明）
    pub fn get_dashboard_stats(&self) -> AppResult<DashboardStats> {
        let conn = &self.conn;

        let pkg_total = count(conn, "SELECT COUNT(*) FROM software_info")?;
        let pkg_outdated = count(
            conn,
            "SELECT COUNT(*) FROM software_info WHERE is_outdated = 1",
        )?;
        let backup_total = count(conn, "SELECT COUNT(*) FROM backup_software")?;
        let cache_total = count(conn, "SELECT COUNT(*) FROM cache_software")?;
        let proxy_total = count(conn, "SELECT COUNT(*) FROM proxies_info")?;
        let proxy_active = count(
            conn,
            "SELECT COUNT(*) FROM proxies_info WHERE is_active = 1",
        )?;
        let license_total = count(conn, "SELECT COUNT(*) FROM enum_licenses")?;
        let language_total = count(
            conn,
            "SELECT COUNT(*) FROM enum_programming_languages",
        )?;

        Ok(DashboardStats {
            pkg_total,
            pkg_updated: pkg_total - pkg_outdated,
            pkg_outdated,
            backup_total,
            cache_total,
            proxy_total,
            proxy_active,
            license_total,
            language_total,
        })
    }
}

/// 执行单列整数聚合查询（COUNT/SUM 等）
/// @param conn - 数据库连接
/// @param sql - 聚合 SQL（固定字符串，无拼接）
/// @returns 聚合结果
fn count(conn: &Connection, sql: &str) -> AppResult<i64> {
    conn.query_row(sql, [], |row| row.get(0))
        .map_err(|e| crate::errors::AppError::DatabaseError(e.to_string()))
}
