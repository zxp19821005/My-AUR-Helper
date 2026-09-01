use crate::errors::AppResult;

use crate::models::*;
use rusqlite::Connection;

use super::Database;

impl Database {
    pub fn upsert_upstream_info(&self, info: &UpstreamInfo) -> AppResult<()> {
        Self::upsert_upstream_info_conn(&self.conn, info)
    }

    /// `upsert_upstream_info` 的底层变体：在指定连接（含事务）上执行
    pub(crate) fn upsert_upstream_info_conn(
        conn: &Connection,
        info: &UpstreamInfo,
    ) -> AppResult<()> {
        conn.execute(
            "INSERT INTO upstream_info (software_id, upstream_version, upstream_license_id, last_checked, upstream_url_status, last_check_error)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(software_id) DO UPDATE SET
                upstream_version=excluded.upstream_version,
                upstream_license_id=excluded.upstream_license_id,
                last_checked=excluded.last_checked,
                upstream_url_status=excluded.upstream_url_status,
                last_check_error=excluded.last_check_error",
            rusqlite::params![
                info.software_id, info.upstream_version,
                info.upstream_license_id, info.last_checked,
                info.upstream_url_status.as_ref().map(|s| s.as_str()),
                info.last_check_error
            ],
        )?;
        Ok(())
    }

    /// 仅记录上游检查失败原因（保留原有版本等字段，只覆盖错误列）
    ///
    /// 用于检查器抛错、网络异常等无法得到版本号的场景：
    /// 版本列保持原值，只打失败标记，列表据此区分「检查失败」与「无版本」。
    /// @param software_id - 软件包 ID
    /// @param reason - 失败原因（人类可读的简短描述）
    pub fn mark_upstream_check_error(&self, software_id: i64, reason: &str) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO upstream_info (software_id, last_check_error)
             VALUES (?1, ?2)
             ON CONFLICT(software_id) DO UPDATE SET
                last_check_error=excluded.last_check_error",
            rusqlite::params![software_id, reason],
        )?;
        Ok(())
    }

    pub fn get_upstream_info(&self, software_id: i64) -> AppResult<Option<UpstreamInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, upstream_version, upstream_license_id, last_checked, upstream_url_status, last_check_error
             FROM upstream_info WHERE software_id=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![software_id], |row| {
            let status_str: Option<String> = row.get(4)?;
            Ok(UpstreamInfo {
                software_id: row.get(0)?,
                upstream_version: row.get(1)?,
                upstream_license_id: row.get(2)?,
                last_checked: row.get(3)?,
                upstream_url_status: status_str.map(|s| UpstreamUrlStatus::parse_from_str(&s)),
                last_check_error: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn update_upstream_url_status(
        &self,
        software_id: i64,
        status: &UpstreamUrlStatus,
    ) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO upstream_info (software_id, upstream_url_status)
             VALUES (?1, ?2)
             ON CONFLICT(software_id) DO UPDATE SET
                upstream_url_status=excluded.upstream_url_status",
            rusqlite::params![software_id, status.as_str()],
        )?;
        Ok(())
    }

    pub fn delete_upstream_info(&self, software_id: i64) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM upstream_info WHERE software_id=?1",
            rusqlite::params![software_id],
        )?;
        Ok(())
    }
}
