/**
 * stats.rs - 仪表盘统计数据访问层
 *
 * 提供 get_dashboard_stats：通过单条聚合查询（多子查询合并）一次返回所有模块计数，
 * 供仪表盘页面展示。避免前端为统计而全量拉取各表数据（软件包/代理/License 等），
 * 同时避免对 8 张表各发一次往返（已合并为单次查询）。
 *
 * 使用场景：
 * - Dashboard 页面加载时调用
 */
use crate::errors::AppResult;
use crate::models::DashboardStats;

use super::Database;

impl Database {
    /// 获取仪表盘统计（各表行数与条件计数）
    ///
    /// 单条 SQL 内用 8 个标量子查询一次性返回所有计数，仅一次 DB 往返；
    /// 全部为 COUNT(*) 聚合，无用户输入拼接，天然防注入。
    /// @returns 统计结果（见 DashboardStats 字段说明）
    pub fn get_dashboard_stats(&self) -> AppResult<DashboardStats> {
        let row = self.conn.query_row(
            "SELECT
                (SELECT COUNT(*) FROM software_info),
                (SELECT COUNT(*) FROM software_info WHERE is_outdated = 1),
                (SELECT COUNT(*) FROM backup_software),
                (SELECT COUNT(*) FROM cache_software),
                (SELECT COUNT(*) FROM proxies_info),
                (SELECT COUNT(*) FROM proxies_info WHERE is_active = 1),
                (SELECT COUNT(*) FROM enum_licenses),
                (SELECT COUNT(*) FROM enum_programming_languages)",
            [],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, i64>(7)?,
                ))
            },
        )?;

        let pkg_total = row.0;
        let pkg_outdated = row.1;

        Ok(DashboardStats {
            pkg_total,
            pkg_updated: pkg_total - pkg_outdated,
            pkg_outdated,
            backup_total: row.2,
            cache_total: row.3,
            proxy_total: row.4,
            proxy_active: row.5,
            license_total: row.6,
            language_total: row.7,
        })
    }
}
