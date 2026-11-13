/**
 * software_update.rs - 软件更新模块的跨表只读查询
 *
 * 功能：一次 JOIN software_info / aur_info / upstream_info 取出版本判定所需的列；
 *      另提供按包名重读上游版本的方法，供写回命令二次校验使用。
 *
 * 设计说明：软件更新模块不新增表、不改 schema，本文件全部为只读查询。
 * SQL 一律参数化，禁止 format! 拼接用户输入。
 *
 * 适用场景：软件更新工作台列表、dry-run 预览、批量写回前的版本复核。
 */
use crate::errors::AppResult;
use crate::models::UpdateCandidateRow;

use super::Database;

impl Database {
    /// 仅查询「过期」标记为 1 的软件包（即版本检查已判定需要更新的包）
    ///
    /// 设计说明：`software_info.is_outdated` 由版本检查链路写入并维护，是「是否需要
    /// 更新」的唯一事实来源，且已有 idx_software_outdated 索引。软件更新页面只消费这个
    /// 结果，不在浏览时重新对全表跑 vercmp 比较，避免打开页面即扫描上千个包。
    ///
    /// # 返回
    /// - `Vec<UpdateCandidateRow>`: 按包名升序排列的 LEFT JOIN 结果，仅含 is_outdated = 1 的行
    pub fn list_update_candidates(&self) -> AppResult<Vec<UpdateCandidateRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.software_id, s.pkgname, a.aur_version, a.out_of_date, \
                    u.upstream_version, u.last_checked \
             FROM software_info s \
             LEFT JOIN aur_info a ON a.software_id = s.software_id \
             LEFT JOIN upstream_info u ON u.software_id = s.software_id \
             WHERE s.is_outdated = 1 \
             ORDER BY s.pkgname",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(UpdateCandidateRow {
                software_id: row.get(0)?,
                pkgname: row.get(1)?,
                aur_version: row.get(2)?,
                out_of_date: row.get::<_, Option<i64>>(3)?.unwrap_or(0) == 1,
                upstream_version: row.get(4)?,
                last_checked: row.get(5)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// 按包名重读上游版本（写回时以数据库为准，不信任前端传入的目标版本）
    ///
    /// # 参数
    /// - `pkgname`: 软件包名称
    ///
    /// # 返回
    /// - `Option<String>`: 上游版本；无记录或无版本时返回 None
    pub fn get_upstream_version_for(&self, pkgname: &str) -> AppResult<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT u.upstream_version FROM upstream_info u \
             JOIN software_info s ON s.software_id = u.software_id \
             WHERE s.pkgname = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![pkgname], |row| {
            row.get::<_, Option<String>>(0)
        })?;
        Ok(rows.next().transpose()?.flatten())
    }
}
