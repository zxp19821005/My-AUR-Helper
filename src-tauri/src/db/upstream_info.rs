use crate::errors::AppResult;

use crate::models::*;

use super::Database;

impl Database {
    pub fn upsert_upstream_info(&self, info: &UpstreamInfo) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO upstream_info (software_id, upstream_version, upstream_license_id, last_checked, upstream_url_status)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(software_id) DO UPDATE SET
                upstream_version=excluded.upstream_version,
                upstream_license_id=excluded.upstream_license_id,
                last_checked=excluded.last_checked,
                upstream_url_status=excluded.upstream_url_status",
            rusqlite::params![
                info.software_id, info.upstream_version,
                info.upstream_license_id, info.last_checked,
                info.upstream_url_status.as_ref().map(|s| s.as_str())
            ],
        )?;
        Ok(())
    }

    pub fn get_upstream_info(&self, software_id: i64) -> AppResult<Option<UpstreamInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, upstream_version, upstream_license_id, last_checked, upstream_url_status 
             FROM upstream_info WHERE software_id=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![software_id], |row| {
            let status_str: Option<String> = row.get(4)?;
            Ok(UpstreamInfo {
                software_id: row.get(0)?,
                upstream_version: row.get(1)?,
                upstream_license_id: row.get(2)?,
                last_checked: row.get(3)?,
                upstream_url_status: status_str.map(|s| UpstreamUrlStatus::from_str(&s)),
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
