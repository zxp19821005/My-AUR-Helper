/**
 * software_info.rs - 软件包信息表数据访问层
 *
 * 提供 software_info 表的查询、写入、详情获取等数据库操作，
 * 是软件包元信息的持久化核心。
 */
use crate::errors::AppResult;

use crate::models::*;
use rusqlite::Connection;

use super::Database;

impl Database {
    pub fn insert_software(&self, sw: &SoftwareInfo) -> AppResult<i64> {
        let language_ids_json = serde_json::to_string(&sw.language_ids).unwrap_or_default();

        self.conn.execute(
            "INSERT INTO software_info (pkgname, upstream_url, package_type_id, checker_type_id, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, language_id, version_extract_regex)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                sw.pkgname, sw.upstream_url, sw.package_type_id.as_id(), sw.checker_type_id.as_id(),
                sw.is_outdated as i32, sw.check_test_versions as i32, sw.check_binary_files as i32,
                sw.auto_check_enabled as i32, language_ids_json, sw.version_extract_regex
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn upsert_software(&self, sw: &SoftwareInfo) -> AppResult<()> {
        let language_ids_json = serde_json::to_string(&sw.language_ids).unwrap_or_default();

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
                sw.auto_check_enabled as i32, language_ids_json, sw.version_extract_regex
            ],
        )?;
        Ok(())
    }

    pub fn update_software_outdated(&self, software_id: i64, is_outdated: bool) -> AppResult<()> {
        Self::update_software_outdated_conn(&self.conn, software_id, is_outdated)
    }

    /// `update_software_outdated` 的底层变体：在指定连接（含事务）上执行
    pub(crate) fn update_software_outdated_conn(
        conn: &Connection,
        software_id: i64,
        is_outdated: bool,
    ) -> AppResult<()> {
        conn.execute(
            "UPDATE software_info SET is_outdated=?1 WHERE software_id=?2",
            rusqlite::params![is_outdated as i32, software_id],
        )?;
        Ok(())
    }

    fn parse_language_ids(json_str: &str) -> Vec<i64> {
        serde_json::from_str(json_str).unwrap_or_default()
    }

    fn row_to_software_info(row: &rusqlite::Row) -> rusqlite::Result<SoftwareInfo> {
        let lang_json: String = row.get(9)?;
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
            language_ids: Self::parse_language_ids(&lang_json),
            version_extract_regex: row.get(10)?,
        })
    }

    pub fn get_all_software(&self) -> AppResult<Vec<SoftwareInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id, \
             is_outdated, check_test_versions, check_binary_files, auto_check_enabled, \
             language_id, version_extract_regex \
             FROM software_info ORDER BY pkgname",
        )?;
        let rows = stmt.query_map([], |row| Self::row_to_software_info(row))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    pub fn get_software_by_name(&self, pkgname: &str) -> AppResult<Option<SoftwareInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id, \
             is_outdated, check_test_versions, check_binary_files, auto_check_enabled, \
             language_id, version_extract_regex \
             FROM software_info WHERE pkgname=?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![pkgname], |row| {
            Self::row_to_software_info(row)
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn update_software_language(
        &self,
        software_id: i64,
        language_id: Option<i64>,
    ) -> AppResult<()> {
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
        let escaped_keyword = keyword.replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%{}%", escaped_keyword);
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id, \
             is_outdated, check_test_versions, check_binary_files, auto_check_enabled, \
             language_id, version_extract_regex \
             FROM software_info \
             WHERE pkgname LIKE ?1 ESCAPE '\\' OR upstream_url LIKE ?1 ESCAPE '\\' \
             ORDER BY pkgname",
        )?;
        let rows = stmt.query_map(rusqlite::params![pattern], |row| {
            Self::row_to_software_info(row)
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
                    a.depends, a.makedepends, a.optdepends,
                    a.license_id,
                    u.upstream_version, CAST(u.last_checked AS INTEGER),
                    u.upstream_license_id
             FROM software_info s
             LEFT JOIN aur_info a ON s.software_id = a.software_id
             LEFT JOIN upstream_info u ON s.software_id = u.software_id
             WHERE s.pkgname = ?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![pkgname], |row| {
            let aur_license_json: Option<String> = row.get(17)?;
            let upstream_license_json: Option<String> = row.get(20)?;
            let lang_json: String = row.get(9)?;
            // 仅记录是否命中而非完整 JSON 载荷，避免每次详情查询输出噪声与潜在敏感信息泄露
            log::debug!(
                "get_software_detail: pkgname={}, has_aur_license={}, has_upstream_license={}",
                pkgname,
                aur_license_json.is_some(),
                upstream_license_json.is_some()
            );
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
                language_ids: Self::parse_language_ids(&lang_json),
                version_extract_regex: row.get(10)?,
                aur_version: row.get(11)?,
                aur_last_updated: row.get(12)?,
                aur_pkgdesc: row.get(13)?,
                depends: row.get(14)?,
                makedepends: row.get(15)?,
                optdepends: row.get(16)?,
                aur_license_name: aur_license_json,
                upstream_version: row.get(18)?,
                upstream_last_checked: row.get(19)?,
                upstream_license_name: upstream_license_json,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn get_prev_next_software(
        &self,
        pkgname: &str,
    ) -> AppResult<(Option<String>, Option<String>)> {
        let mut stmt = self.conn.prepare(
            "SELECT pkgname FROM software_info WHERE pkgname < ?1 ORDER BY pkgname DESC LIMIT 1",
        )?;
        let prev = stmt
            .query_map(rusqlite::params![pkgname], |row| row.get(0))?
            .next()
            .transpose()?;

        let mut stmt = self.conn.prepare(
            "SELECT pkgname FROM software_info WHERE pkgname > ?1 ORDER BY pkgname ASC LIMIT 1",
        )?;
        let next = stmt
            .query_map(rusqlite::params![pkgname], |row| row.get(0))?
            .next()
            .transpose()?;

        Ok((prev, next))
    }

    /// 将一行查询结果转换为列表视图条目
    fn row_to_list_entry(row: &rusqlite::Row) -> rusqlite::Result<SoftwareListEntry> {
        let status_str: Option<String> = row.get(10)?;
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
            upstream_url: row.get(9)?,
            upstream_url_status: status_str.map(|s| UpstreamUrlStatus::from_str(&s)),
            upstream_license_id: row.get(11)?,
        })
    }

    pub fn get_software_list_entries(&self) -> AppResult<Vec<SoftwareListEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.software_id, s.pkgname, s.package_type_id, s.checker_type_id, s.is_outdated,
                    a.aur_version, CAST(a.last_updated AS INTEGER),
                    u.upstream_version, CAST(u.last_checked AS INTEGER),
                    s.upstream_url, u.upstream_url_status, u.upstream_license_id
             FROM software_info s
             LEFT JOIN aur_info a ON s.software_id = a.software_id
             LEFT JOIN upstream_info u ON s.software_id = u.software_id
             ORDER BY s.pkgname",
        )?;
        let rows = stmt.query_map([], |row| Self::row_to_list_entry(row))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 按 pkgname 获取单条列表视图条目（用于定向刷新，避免整表重载）
    /// @param pkgname - 软件包名
    /// @returns 若存在返回 Some，否则 None（例如已被删除）
    pub fn get_software_list_entry(&self, pkgname: &str) -> AppResult<Option<SoftwareListEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.software_id, s.pkgname, s.package_type_id, s.checker_type_id, s.is_outdated,
                    a.aur_version, CAST(a.last_updated AS INTEGER),
                    u.upstream_version, CAST(u.last_checked AS INTEGER),
                    s.upstream_url, u.upstream_url_status, u.upstream_license_id
             FROM software_info s
             LEFT JOIN aur_info a ON s.software_id = a.software_id
             LEFT JOIN upstream_info u ON s.software_id = u.software_id
             WHERE s.pkgname = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![pkgname], |row| {
            Self::row_to_list_entry(row)
        })?;
        Ok(rows.next().transpose()?)
    }

    /// 更新软件的语言 ID 列表
    /// @param software_id - 软件 ID
    /// @param language_ids - 语言 ID 列表
    pub fn update_software_languages(
        &self,
        software_id: i64,
        language_ids: &[i64],
    ) -> AppResult<()> {
        Self::update_software_languages_conn(&self.conn, software_id, language_ids)
    }

    /// `update_software_languages` 的底层变体：在指定连接（含事务）上执行
    pub(crate) fn update_software_languages_conn(
        conn: &Connection,
        software_id: i64,
        language_ids: &[i64],
    ) -> AppResult<()> {
        let language_ids_json = serde_json::to_string(language_ids).unwrap_or_default();
        conn.execute(
            "UPDATE software_info SET language_id = ?1 WHERE software_id = ?2",
            rusqlite::params![language_ids_json, software_id],
        )?;
        Ok(())
    }
}
