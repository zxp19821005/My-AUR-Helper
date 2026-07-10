use super::Database;
use crate::errors::AppResult;
use crate::models::*;

impl Database {
    pub fn upsert_license(&self, lic: &EnumLicense) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO enum_licenses (spdx_id, full_name) VALUES (?1, ?2)
             ON CONFLICT(spdx_id) DO UPDATE SET full_name=excluded.full_name",
            rusqlite::params![lic.spdx_id, lic.full_name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_licenses(&self) -> AppResult<Vec<EnumLicense>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, spdx_id, full_name FROM enum_licenses ORDER BY spdx_id")?;
        let rows = stmt.query_map([], |row| {
            Ok(EnumLicense {
                id: Some(row.get(0)?),
                spdx_id: row.get(1)?,
                full_name: row.get(2)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn get_license_by_spdx_id(&self, spdx_id: &str) -> AppResult<Option<EnumLicense>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, spdx_id, full_name FROM enum_licenses WHERE spdx_id=?1")?;
        let mut rows = stmt.query_map(rusqlite::params![spdx_id], |row| {
            Ok(EnumLicense {
                id: Some(row.get(0)?),
                spdx_id: row.get(1)?,
                full_name: row.get(2)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn get_license_by_id(&self, id: i64) -> AppResult<Option<EnumLicense>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, spdx_id, full_name FROM enum_licenses WHERE id=?1")?;
        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok(EnumLicense {
                id: Some(row.get(0)?),
                spdx_id: row.get(1)?,
                full_name: row.get(2)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn update_license(&self, lic: &EnumLicense) -> AppResult<()> {
        self.conn.execute(
            "UPDATE enum_licenses SET spdx_id=?1, full_name=?2 WHERE id=?3",
            rusqlite::params![lic.spdx_id, lic.full_name, lic.id],
        )?;
        Ok(())
    }

    pub fn delete_license_by_id(&self, id: i64) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM enum_licenses WHERE id=?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    /// 获取或创建 License ID
    ///
    /// 根据 SPDX ID 查询 enum_licenses 表，如果不存在则自动创建
    ///
    /// # 参数
    /// - `spdx_id`: License 的 SPDX ID（如 "MIT", "Apache-2.0"）
    ///
    /// # 返回
    /// - `Ok(Some(id))`: License 存在或创建成功，返回 ID
    /// - `Ok(None)`: spdx_id 为 None
    /// - `Err(e)`: 数据库操作失败
    pub fn get_or_create_license_id(&self, spdx_id: Option<&str>) -> AppResult<Option<i64>> {
        match spdx_id {
            Some(spdx) => {
                let lic = self.get_license_by_spdx_id(spdx)?;
                if let Some(license) = lic {
                    Ok(license.id)
                } else {
                    let new_lic = EnumLicense {
                        id: None,
                        spdx_id: spdx.to_string(),
                        full_name: spdx.to_string(),
                    };
                    Ok(Some(self.upsert_license(&new_lic)?))
                }
            }
            None => Ok(None),
        }
    }
}
