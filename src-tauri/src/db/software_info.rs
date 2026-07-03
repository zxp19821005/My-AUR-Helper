use anyhow::Result;

use crate::models::*;

use super::Database;

impl Database {
    /// 插入新的软件包记录
    pub fn insert_software(&self, sw: &SoftwareInfo) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO software_info (pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id, language_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                sw.pkgname, sw.upstream_url, sw.package_type_id.as_id(), sw.checker_type_id.as_id(),
                sw.is_outdated as i32, sw.check_test_versions as i32, sw.check_binary_files as i32,
                sw.auto_check_enabled as i32, sw.license_id, sw.language_id
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 插入或更新软件包记录（按 pkgname 去重）
    pub fn upsert_software(&self, sw: &SoftwareInfo) -> Result<()> {
        self.conn.execute(
            "INSERT INTO software_info (pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id, language_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(pkgname) DO UPDATE SET
                upstream_url=excluded.upstream_url, package_type_id=excluded.package_type_id,
                checker_type_id=excluded.checker_type_id, is_outdated=excluded.is_outdated,
                check_test_versions=excluded.check_test_versions, check_binary_files=excluded.check_binary_files,
                auto_check_enabled=excluded.auto_check_enabled,
                license_id=excluded.license_id, language_id=excluded.language_id",
            rusqlite::params![
                sw.pkgname, sw.upstream_url, sw.package_type_id.as_id(), sw.checker_type_id.as_id(),
                sw.is_outdated as i32, sw.check_test_versions as i32, sw.check_binary_files as i32,
                sw.auto_check_enabled as i32, sw.license_id, sw.language_id
            ],
        )?;
        Ok(())
    }

    /// 更新软件包的过期状态
    pub fn update_software_outdated(&self, software_id: i64, is_outdated: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE software_info SET is_outdated=?1 WHERE software_id=?2",
            rusqlite::params![is_outdated as i32, software_id],
        )?;
        Ok(())
    }

    /// 获取所有软件包列表
    pub fn get_all_software(&self) -> Result<Vec<SoftwareInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id, language_id FROM software_info ORDER BY pkgname"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SoftwareInfo {
                software_id: Some(row.get(0)?),
                pkgname: row.get(1)?,
                upstream_url: row.get(2)?,
                package_type_id: PackageType::from_id(row.get(3)?),
                checker_type_id: CheckerType::from_id(row.get(4)?),
                is_outdated: row.get::<_, i32>(5)? != 0,
                check_test_versions: row.get::<_, i32>(6)? != 0,
                check_binary_files: row.get::<_, i32>(7)? != 0,
                auto_check_enabled: row.get::<_, i32>(8)? != 0,
                license_id: row.get(9)?,
                language_id: row.get(10)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 根据包名获取软件包信息
    pub fn get_software_by_name(&self, pkgname: &str) -> Result<Option<SoftwareInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id, language_id FROM software_info WHERE pkgname=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![pkgname], |row| {
            Ok(SoftwareInfo {
                software_id: Some(row.get(0)?),
                pkgname: row.get(1)?,
                upstream_url: row.get(2)?,
                package_type_id: PackageType::from_id(row.get(3)?),
                checker_type_id: CheckerType::from_id(row.get(4)?),
                is_outdated: row.get::<_, i32>(5)? != 0,
                check_test_versions: row.get::<_, i32>(6)? != 0,
                check_binary_files: row.get::<_, i32>(7)? != 0,
                auto_check_enabled: row.get::<_, i32>(8)? != 0,
                license_id: row.get(9)?,
                language_id: row.get(10)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    /// 更新软件包的 License
    pub fn update_software_license(&self, software_id: i64, license_id: Option<i64>) -> Result<()> {
        self.conn.execute(
            "UPDATE software_info SET license_id=?1 WHERE software_id=?2",
            rusqlite::params![license_id, software_id],
        )?;
        Ok(())
    }

    /// 更新软件包的编程语言
    pub fn update_software_language(&self, software_id: i64, language_id: Option<i64>) -> Result<()> {
        self.conn.execute(
            "UPDATE software_info SET language_id=?1 WHERE software_id=?2",
            rusqlite::params![language_id, software_id],
        )?;
        Ok(())
    }

    /// 删除软件包
    pub fn delete_software(&self, software_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM software_info WHERE software_id=?1",
            rusqlite::params![software_id],
        )?;
        Ok(())
    }

    /// 搜索软件包
    pub fn search_software(&self, keyword: &str) -> Result<Vec<SoftwareInfo>> {
        let pattern = format!("%{}%", keyword);
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id, language_id FROM software_info WHERE pkgname LIKE ?1 OR upstream_url LIKE ?1 ORDER BY pkgname"
        )?;
        let rows = stmt.query_map(rusqlite::params![pattern], |row| {
            Ok(SoftwareInfo {
                software_id: Some(row.get(0)?),
                pkgname: row.get(1)?,
                upstream_url: row.get(2)?,
                package_type_id: PackageType::from_id(row.get(3)?),
                checker_type_id: CheckerType::from_id(row.get(4)?),
                is_outdated: row.get::<_, i32>(5)? != 0,
                check_test_versions: row.get::<_, i32>(6)? != 0,
                check_binary_files: row.get::<_, i32>(7)? != 0,
                auto_check_enabled: row.get::<_, i32>(8)? != 0,
                license_id: row.get(9)?,
                language_id: row.get(10)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }
}
