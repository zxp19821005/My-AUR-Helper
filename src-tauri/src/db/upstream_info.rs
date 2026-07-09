use crate::errors::AppResult;

use crate::models::*;

use super::Database;

impl Database {
    pub fn upsert_upstream_info(&self, info: &UpstreamInfo) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO upstream_info (software_id, upstream_version, upstream_license_id, last_checked)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(software_id) DO UPDATE SET
                upstream_version=excluded.upstream_version,
                upstream_license_id=excluded.upstream_license_id,
                last_checked=excluded.last_checked",
            rusqlite::params![
                info.software_id, info.upstream_version,
                info.upstream_license_id, info.last_checked
            ],
        )?;
        Ok(())
    }

    pub fn get_upstream_info(&self, software_id: i64) -> AppResult<Option<UpstreamInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, upstream_version, upstream_license_id, last_checked FROM upstream_info WHERE software_id=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![software_id], |row| {
            Ok(UpstreamInfo {
                software_id: row.get(0)?,
                upstream_version: row.get(1)?,
                upstream_license_id: row.get(2)?,
                last_checked: row.get(3)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn delete_upstream_info(&self, software_id: i64) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM upstream_info WHERE software_id=?1",
            rusqlite::params![software_id],
        )?;
        Ok(())
    }
}
