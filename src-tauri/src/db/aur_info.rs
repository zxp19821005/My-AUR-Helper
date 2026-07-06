use crate::errors::AppResult; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database;  // 数据库结构体

impl Database {
    /// 插入或更新 AUR 包信息
    /// @param info - AUR 包信息（按 software_id 去重）
    pub fn upsert_aur_info(&self, info: &AurInfo) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO aur_info (software_id, pkgdesc, aur_version, license_id, last_updated, depends, makedepends, optdepends, out_of_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(software_id) DO UPDATE SET
                pkgdesc=excluded.pkgdesc, aur_version=excluded.aur_version, license_id=excluded.license_id,
                last_updated=excluded.last_updated, depends=excluded.depends, makedepends=excluded.makedepends,
                optdepends=excluded.optdepends, out_of_date=excluded.out_of_date",
            // 参数列表：9 个字段
            rusqlite::params![
                info.software_id, info.pkgdesc, info.aur_version, info.license_id,
                info.last_updated, info.depends, info.makedepends, info.optdepends,
                info.out_of_date.map(|b| b as i32), // bool 转 i32
            ],
        )?;
        Ok(())
    }

    /// 获取指定软件包的 AUR 信息
    /// @param software_id - 软件包 ID
    /// @returns 可选的 AUR 包信息
    pub fn get_aur_info(&self, software_id: i64) -> AppResult<Option<AurInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgdesc, aur_version, license_id, CAST(last_updated AS INTEGER), depends, makedepends, optdepends, out_of_date FROM aur_info WHERE software_id=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![software_id], |row| {
            Ok(AurInfo {
                software_id: row.get(0)?,
                pkgdesc: row.get(1)?,
                aur_version: row.get(2)?,
                license_id: row.get(3)?,
                last_updated: row.get(4)?,
                depends: row.get(5)?,
                makedepends: row.get(6)?,
                optdepends: row.get(7)?,
                out_of_date: row.get::<_, Option<i32>>(8)?.map(|v| v != 0), // i32 转 bool
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    /// 删除指定软件包的 AUR 信息
    /// @param software_id - 软件包 ID
    pub fn delete_aur_info(&self, software_id: i64) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM aur_info WHERE software_id=?1",
            rusqlite::params![software_id],
        )?;
        Ok(())
    }
}
