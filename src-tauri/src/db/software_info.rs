use crate::errors::AppResult;

use crate::models::*;

use super::Database;

impl Database {
    pub fn insert_software(&self, sw: &SoftwareInfo) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO software_info (pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, language_id, version_extract_regex)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                sw.pkgname, sw.upstream_url, sw.package_type_id.as_id(), sw.checker_type_id.as_id(),
                sw.is_outdated as i32, sw.check_test_versions as i32, sw.check_binary_files as i32,
                sw.auto_check_enabled as i32, sw.language_id, sw.version_extract_regex
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn upsert_software(&self, sw: &SoftwareInfo) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO software_info (pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, language_id, version_extract_regex)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(pkgname) DO UPDATE SET
                upstream_url=excluded.upstream_url, package_type_id=excluded.package_type_id,
                checker_type_id=excluded.checker_type_id, is_outdated=excluded.is_outdated,
                check_test_versions=excluded.check_test_versions, check_binary_files=excluded.check_binary_files,
                auto_check_enabled=excluded.auto_check_enabled,
                language_id=excluded.language_id,
                version_extract_regex=excluded.version_extract_regex",
            rusqlite::params![
                sw.pkgname, sw.upstream_url, sw.package_type_id.as_id(), sw.checker_type_id.as_id(),
                sw.is_outdated as i32, sw.check_test_versions as i32, sw.check_binary_files as i32,
                sw.auto_check_enabled as i32, sw.language_id, sw.version_extract_regex
            ],
        )?;
        Ok(())
    }

    pub fn update_software_outdated(&self, software_id: i64, is_outdated: bool) -> AppResult<()> {
        self.conn.execute(
            "UPDATE software_info SET is_outdated=?1 WHERE software_id=?2",
            rusqlite::params![is_outdated as i32, software_id],
        )?;
        Ok(())
    }

    pub fn get_all_software(&self) -> AppResult<Vec<SoftwareInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, language_id, version_extract_regex FROM software_info ORDER BY pkgname"
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
                language_id: row.get(9)?,
                version_extract_regex: row.get(10)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn get_software_by_name(&self, pkgname: &str) -> AppResult<Option<SoftwareInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, language_id, version_extract_regex FROM software_info WHERE pkgname=?1"
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
                language_id: row.get(9)?,
                version_extract_regex: row.get(10)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn update_software_language(&self, software_id: i64, language_id: Option<i64>) -> AppResult<()> {
        self.conn.execute(
            "UPDATE software_info SET language_id=?1 WHERE software_id=?2",
            rusqlite::params![language_id, software_id],
        )?;
        Ok(())
    }

    pub fn delete_software(&self, software_id: i64) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM software_info WHERE software_id=?1",
            rusqlite::params![software_id],
        )?;
        Ok(())
    }

    pub fn search_software(&self, keyword: &str) -> AppResult<Vec<SoftwareInfo>> {
        let pattern = format!("%{}%", keyword);
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, language_id, version_extract_regex FROM software_info WHERE pkgname LIKE ?1 OR upstream_url LIKE ?1 ORDER BY pkgname"
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
                language_id: row.get(9)?,
                version_extract_regex: row.get(10)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn get_software_detail_by_name(&self, pkgname: &str) -> AppResult<Option<SoftwareDetail>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.software_id, s.pkgname, s.upstream_url, s.package_type_id, s.checker_type_id,
                    s.is_outdated, s.check_test_versions, s.check_binary_files, s.auto_check_enabled,
                    s.language_id, s.version_extract_regex,
                    a.aur_version, CAST(a.last_updated AS INTEGER), a.pkgdesc,
                    u.upstream_version, CAST(u.last_checked AS INTEGER)
             FROM software_info s
             LEFT JOIN aur_info a ON s.software_id = a.software_id
             LEFT JOIN upstream_info u ON s.software_id = u.software_id
             WHERE s.pkgname = ?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![pkgname], |row| {
            Ok(SoftwareDetail {
                software_id: Some(row.get(0)?),
                pkgname: row.get(1)?,
                upstream_url: row.get(2)?,
                package_type_id: PackageType::from_id(row.get(3)?),
                checker_type_id: CheckerType::from_id(row.get(4)?),
                is_outdated: row.get::<_, i32>(5)? != 0,
                check_test_versions: row.get::<_, i32>(6)? != 0,
                check_binary_files: row.get::<_, i32>(7)? != 0,
                auto_check_enabled: row.get::<_, i32>(8)? != 0,
                language_id: row.get(9)?,
                version_extract_regex: row.get(10)?,
                aur_version: row.get(11)?,
                aur_last_updated: row.get(12)?,
                aur_pkgdesc: row.get(13)?,
                upstream_version: row.get(14)?,
                upstream_last_checked: row.get(15)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn get_prev_next_software(&self, pkgname: &str) -> AppResult<(Option<String>, Option<String>)> {
        let mut stmt = self.conn.prepare(
            "SELECT pkgname FROM software_info WHERE pkgname < ?1 ORDER BY pkgname DESC LIMIT 1"
        )?;
        let prev = stmt.query_map(rusqlite::params![pkgname], |row| row.get(0))?
            .next().transpose()?;

        let mut stmt = self.conn.prepare(
            "SELECT pkgname FROM software_info WHERE pkgname > ?1 ORDER BY pkgname ASC LIMIT 1"
        )?;
        let next = stmt.query_map(rusqlite::params![pkgname], |row| row.get(0))?
            .next().transpose()?;

        Ok((prev, next))
    }

    pub fn get_software_list_entries(&self) -> AppResult<Vec<SoftwareListEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.software_id, s.pkgname, s.package_type_id, s.checker_type_id, s.is_outdated,
                    a.aur_version, CAST(a.last_updated AS INTEGER),
                    u.upstream_version, CAST(u.last_checked AS INTEGER)
             FROM software_info s
             LEFT JOIN aur_info a ON s.software_id = a.software_id
             LEFT JOIN upstream_info u ON s.software_id = u.software_id
             ORDER BY s.pkgname"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SoftwareListEntry {
                software_id: row.get(0)?,
                pkgname: row.get(1)?,
                package_type_id: PackageType::from_id(row.get(2)?),
                checker_type_id: CheckerType::from_id(row.get(3)?),
                is_outdated: row.get::<_, i32>(4)? != 0,
                aur_version: row.get(5)?,
                aur_last_updated: row.get(6)?,
                upstream_version: row.get(7)?,
                upstream_last_checked: row.get(8)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }
}
