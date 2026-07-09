use crate::errors::AppResult; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database;  // 数据库结构体

impl Database {
    /// 插入或更新 License 记录（按 spdx_id 去重）
    /// @param lic - License 信息
    /// @returns 新插入或更新的记录 ID
    pub fn upsert_license(&self, lic: &EnumLicense) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO enum_licenses (spdx_id, full_name, url, is_deprecated, is_osi_approved, description, category)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(spdx_id) DO UPDATE SET
                full_name=excluded.full_name, url=excluded.url,
                is_deprecated=excluded.is_deprecated, is_osi_approved=excluded.is_osi_approved,
                description=excluded.description, category=excluded.category",
            rusqlite::params![
                lic.spdx_id, lic.full_name, lic.url, lic.is_deprecated as i32,
                lic.is_osi_approved as i32, lic.description, lic.category
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 获取所有 License 列表（按 spdx_id 排序）
    /// @returns 所有 License 记录
    pub fn get_all_licenses(&self) -> AppResult<Vec<EnumLicense>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, spdx_id, full_name, url, is_deprecated, is_osi_approved, description, category, created_at FROM enum_licenses ORDER BY spdx_id"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(EnumLicense {
                id: Some(row.get(0)?),
                spdx_id: row.get(1)?,
                full_name: row.get(2)?,
                url: row.get(3)?,
                is_deprecated: row.get::<_, i32>(4)? != 0,
                is_osi_approved: row.get::<_, i32>(5)? != 0,
                description: row.get(6)?,
                category: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 根据 SPDX ID 获取 License
    /// @param spdx_id - SPDX 标准 ID
    /// @returns 可选的 License 记录
    pub fn get_license_by_spdx_id(&self, spdx_id: &str) -> AppResult<Option<EnumLicense>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, spdx_id, full_name, url, is_deprecated, is_osi_approved, description, category, created_at FROM enum_licenses WHERE spdx_id=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![spdx_id], |row| {
            Ok(EnumLicense {
                id: Some(row.get(0)?),
                spdx_id: row.get(1)?,
                full_name: row.get(2)?,
                url: row.get(3)?,
                is_deprecated: row.get::<_, i32>(4)? != 0,
                is_osi_approved: row.get::<_, i32>(5)? != 0,
                description: row.get(6)?,
                category: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn get_license_by_id(&self, id: i64) -> AppResult<Option<EnumLicense>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, spdx_id, full_name, url, is_deprecated, is_osi_approved, description, category, created_at FROM enum_licenses WHERE id=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(EnumLicense {
                id: Some(row.get(0)?),
                spdx_id: row.get(1)?,
                full_name: row.get(2)?,
                url: row.get(3)?,
                is_deprecated: row.get::<_, i32>(4)? != 0,
                is_osi_approved: row.get::<_, i32>(5)? != 0,
                description: row.get(6)?,
                category: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn update_license(&self, lic: &EnumLicense) -> AppResult<()> {
        self.conn.execute(
            "UPDATE enum_licenses SET spdx_id=?1, full_name=?2, url=?3, is_deprecated=?4, is_osi_approved=?5, description=?6, category=?7 WHERE id=?8",
            rusqlite::params![
                lic.spdx_id, lic.full_name, lic.url, lic.is_deprecated as i32,
                lic.is_osi_approved as i32, lic.description, lic.category, lic.id
            ],
        )?;
        Ok(())
    }

    pub fn delete_license_by_id(&self, id: i64) -> AppResult<()> {
        self.conn.execute("DELETE FROM enum_licenses WHERE id=?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn delete_all_licenses(&self) -> AppResult<()> {
        self.conn.execute("DELETE FROM enum_licenses", [])?;
        Ok(())
    }
}
