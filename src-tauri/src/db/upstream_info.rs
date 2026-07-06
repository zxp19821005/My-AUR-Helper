use crate::errors::AppResult; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database;  // 数据库结构体

impl Database {
    /// 插入或更新上游版本信息
    /// @param info - 上游版本信息（按 software_id 去重）
    pub fn upsert_upstream_info(&self, info: &UpstreamInfo) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO upstream_info (software_id, upstream_url, upstream_version, upstream_license, last_checked)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(software_id) DO UPDATE SET
                upstream_url=excluded.upstream_url, upstream_version=excluded.upstream_version,
                upstream_license=excluded.upstream_license, last_checked=excluded.last_checked",
            rusqlite::params![
                info.software_id, info.upstream_url, info.upstream_version,
                info.upstream_license, info.last_checked
            ],
        )?;
        Ok(())
    }

    /// 获取指定软件包的上游版本信息
    /// @param software_id - 软件包 ID
    /// @returns 可选的的上游版本信息
    pub fn get_upstream_info(&self, software_id: i64) -> AppResult<Option<UpstreamInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, upstream_url, upstream_version, upstream_license, last_checked FROM upstream_info WHERE software_id=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![software_id], |row| {
            Ok(UpstreamInfo {
                software_id: row.get(0)?,
                upstream_url: row.get(1)?,
                upstream_version: row.get(2)?,
                upstream_license: row.get(3)?,
                last_checked: row.get(4)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    /// 删除指定软件包的上游版本信息
    /// @param software_id - 软件包 ID
    pub fn delete_upstream_info(&self, software_id: i64) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM upstream_info WHERE software_id=?1",
            rusqlite::params![software_id],
        )?;
        Ok(())
    }
}
