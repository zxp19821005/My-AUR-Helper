use anyhow::Result; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database;  // 数据库结构体

impl Database {
    /// 插入新的软件包记录
    /// @param sw - 软件包信息（不包含 software_id）
    /// @returns 新插入记录的软件包 ID
    pub fn insert_software(&self, sw: &SoftwareInfo) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO software_info (pkgname, upstream_url, package_type, checker_type, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id, language_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                sw.pkgname, sw.upstream_url, sw.package_type.as_id(), sw.checker_type.as_id(),
                sw.is_outdated as i32, sw.check_test_versions as i32, sw.check_binary_files as i32,
                sw.auto_check_enabled as i32, sw.license_id, sw.language_id, sw.created_at
            ],
        )?;
        Ok(self.conn.last_insert_rowid()) // 返回自增 ID
    }

    /// 插入或更新软件包记录（按 pkgname 去重）
    /// @param sw - 软件包信息
    pub fn upsert_software(&self, sw: &SoftwareInfo) -> Result<()> {
        self.conn.execute(
            "INSERT INTO software_info (pkgname, upstream_url, package_type, checker_type, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id, language_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(pkgname) DO UPDATE SET
                upstream_url=excluded.upstream_url, package_type=excluded.package_type,
                checker_type=excluded.checker_type, is_outdated=excluded.is_outdated,
                check_test_versions=excluded.check_test_versions, check_binary_files=excluded.check_binary_files,
                auto_check_enabled=excluded.auto_check_enabled,
                license_id=excluded.license_id, language_id=excluded.language_id",
            rusqlite::params![
                sw.pkgname, sw.upstream_url, sw.package_type.as_id(), sw.checker_type.as_id(),
                sw.is_outdated as i32, sw.check_test_versions as i32, sw.check_binary_files as i32,
                sw.auto_check_enabled as i32, sw.license_id, sw.language_id, sw.created_at
            ],
        )?;
        Ok(())
    }

    /// 更新软件包的过期状态
    /// @param software_id - 软件包 ID
    /// @param is_outdated - 是否过期
    pub fn update_software_outdated(&self, software_id: i64, is_outdated: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE software_info SET is_outdated=?1 WHERE software_id=?2",
            rusqlite::params![is_outdated as i32, software_id],
        )?;
        Ok(())
    }

    /// 获取所有软件包列表（按包名排序）
    /// @returns 所有软件包信息列表
    pub fn get_all_software(&self) -> Result<Vec<SoftwareInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type, checker_type, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id, language_id, created_at FROM software_info ORDER BY pkgname"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SoftwareInfo {
                software_id: Some(row.get(0)?),
                pkgname: row.get(1)?,
                upstream_url: row.get(2)?,
                package_type: PackageType::from_id(row.get(3)?),  // 数字 ID 转枚举
                checker_type: CheckerType::from_id(row.get(4)?),  // 数字 ID 转枚举
                is_outdated: row.get::<_, i32>(5)? != 0,         // 整数转布尔
                check_test_versions: row.get::<_, i32>(6)? != 0,
                check_binary_files: row.get::<_, i32>(7)? != 0,
                auto_check_enabled: row.get::<_, i32>(8)? != 0,
                license_id: row.get(9)?,
                language_id: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 根据包名获取软件包信息
    /// @param pkgname - 包名
    /// @returns 可选的软件包信息
    pub fn get_software_by_name(&self, pkgname: &str) -> Result<Option<SoftwareInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type, checker_type, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id, language_id, created_at FROM software_info WHERE pkgname=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![pkgname], |row| {
            Ok(SoftwareInfo {
                software_id: Some(row.get(0)?),
                pkgname: row.get(1)?,
                upstream_url: row.get(2)?,
                package_type: PackageType::from_id(row.get(3)?),
                checker_type: CheckerType::from_id(row.get(4)?),
                is_outdated: row.get::<_, i32>(5)? != 0,
                check_test_versions: row.get::<_, i32>(6)? != 0,
                check_binary_files: row.get::<_, i32>(7)? != 0,
                auto_check_enabled: row.get::<_, i32>(8)? != 0,
                license_id: row.get(9)?,
                language_id: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;
        Ok(rows.next().transpose()?) // 取第一条或 None
    }

    /// 更新软件包的 License
    /// @param software_id - 软件包 ID
    /// @param license_id - License ID（None 表示清除）
    pub fn update_software_license(&self, software_id: i64, license_id: Option<i64>) -> Result<()> {
        self.conn.execute(
            "UPDATE software_info SET license_id=?1 WHERE software_id=?2",
            rusqlite::params![license_id, software_id],
        )?;
        Ok(())
    }

    /// 更新软件包的编程语言
    /// @param software_id - 软件包 ID
    /// @param language_id - 编程语言 ID（None 表示清除）
    pub fn update_software_language(&self, software_id: i64, language_id: Option<i64>) -> Result<()> {
        self.conn.execute(
            "UPDATE software_info SET language_id=?1 WHERE software_id=?2",
            rusqlite::params![language_id, software_id],
        )?;
        Ok(())
    }

    /// 删除软件包（级联删除关联的 aur_info、upstream_info 等）
    /// @param software_id - 要删除的软件包 ID
    pub fn delete_software(&self, software_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM software_info WHERE software_id=?1",
            rusqlite::params![software_id],
        )?;
        Ok(())
    }

    /// 搜索软件包（按包名或上游 URL 模糊匹配）
    /// @param keyword - 搜索关键词
    /// @returns 匹配的软件包列表
    pub fn search_software(&self, keyword: &str) -> Result<Vec<SoftwareInfo>> {
        let pattern = format!("%{}%", keyword);  // SQL LIKE 模糊匹配模式
        let mut stmt = self.conn.prepare(
            "SELECT software_id, pkgname, upstream_url, package_type, checker_type, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id, language_id, created_at FROM software_info WHERE pkgname LIKE ?1 OR upstream_url LIKE ?1 ORDER BY pkgname"
        )?;
        let rows = stmt.query_map(rusqlite::params![pattern], |row| {
            Ok(SoftwareInfo {
                software_id: Some(row.get(0)?),
                pkgname: row.get(1)?,
                upstream_url: row.get(2)?,
                package_type: PackageType::from_id(row.get(3)?),
                checker_type: CheckerType::from_id(row.get(4)?),
                is_outdated: row.get::<_, i32>(5)? != 0,
                check_test_versions: row.get::<_, i32>(6)? != 0,
                check_binary_files: row.get::<_, i32>(7)? != 0,
                auto_check_enabled: row.get::<_, i32>(8)? != 0,
                license_id: row.get(9)?,
                language_id: row.get(10)?,
                created_at: row.get(11)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }
}
