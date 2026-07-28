/**
 * backup_software.rs - 备份记录表 CRUD
 *
 * 功能：
 * - insert_backup_software: 插入备份记录
 * - get_all_backup_software: 查询所有备份记录
 * - get_all_backup_entries: 查询所有备份记录（含解析出的包名，用于列表展示）
 * - clear_backup_software: 清空备份表（重置自增 ID）
 * - delete_backup_software_batch: 批量删除备份记录
 * - delete_backup_software: 删除单条备份记录
 * - get_backup_software_by_filename: 按文件名查找备份记录
 */
use crate::errors::AppResult;

use crate::models::*;

use super::Database;

/// 从文件名解析包名（去掉版本号、pkgrel、arch 后缀）
fn extract_pkgname(filename: &str) -> String {
    let base = filename.strip_suffix(".pkg.tar.zst").unwrap_or(filename);
    let parts: Vec<&str> = base.rsplitn(3, '-').collect();
    if parts.len() < 3 {
        return base.to_string();
    }
    // parts[0]=arch, parts[1]=pkgrel, parts[2]=name-version
    let name_ver = parts[2];
    if let Some(pos) = name_ver.rfind('-') {
        name_ver[..pos].to_string()
    } else {
        name_ver.to_string()
    }
}

impl Database {
    /// 插入备份软件记录
    /// @param bs - 备份软件信息
    /// @returns 新插入记录的 ID
    pub fn insert_backup_software(&self, bs: &BackupSoftware) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO backup_software (filename, epoch, pkgver, pkgrel, arch, subdirectory, full_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                bs.filename,
                bs.epoch,
                bs.pkgver,
                bs.pkgrel,
                bs.arch,
                bs.subdirectory,
                bs.full_path
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 获取所有备份记录
    /// @returns 所有备份记录列表
    pub fn get_all_backup_software(&self) -> AppResult<Vec<BackupSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, epoch, pkgver, pkgrel, arch, subdirectory, full_path
             FROM backup_software ORDER BY filename",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BackupSoftware {
                id: Some(row.get(0)?),
                filename: row.get(1)?,
                epoch: row.get(2)?,
                pkgver: row.get(3)?,
                pkgrel: row.get(4)?,
                arch: row.get(5)?,
                subdirectory: row.get(6).ok(),
                full_path: row.get(7)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 获取所有备份记录（含解析出的包名，用于列表展示）
    /// @returns 备份记录列表
    pub fn get_all_backup_entries(&self) -> AppResult<Vec<BackupSoftwareEntry>> {
        let entries = self.get_all_backup_software()?;
        Ok(entries
            .into_iter()
            .map(|e| {
                let pkgname = extract_pkgname(&e.filename);
                BackupSoftwareEntry {
                    id: e.id.unwrap_or(0),
                    pkgname,
                    filename: e.filename,
                    epoch: e.epoch,
                    pkgver: e.pkgver,
                    pkgrel: e.pkgrel,
                    arch: e.arch,
                    subdirectory: e.subdirectory,
                    full_path: e.full_path,
                }
            })
            .collect())
    }

    /// 清空备份表并重置自增 ID
    /// @returns 删除的记录数
    pub fn clear_backup_software(&self) -> AppResult<usize> {
        let count = self.conn.execute("DELETE FROM backup_software", [])?;
        self.conn.execute(
            "DELETE FROM sqlite_sequence WHERE name='backup_software'",
            [],
        )?;
        Ok(count)
    }

    /// 批量删除备份记录
    /// @param ids - 要删除的记录 ID 列表
    /// @returns 删除的记录数
    pub fn delete_backup_software_batch(&self, ids: &[i64]) -> AppResult<usize> {
        let mut count = 0;
        for id in ids {
            self.conn.execute(
                "DELETE FROM backup_software WHERE id=?1",
                rusqlite::params![id],
            )?;
            count += 1;
        }
        Ok(count)
    }

    /// 删除备份记录
    /// @param id - 备份记录 ID
    pub fn delete_backup_software(&self, id: i64) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM backup_software WHERE id=?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    /// 根据文件名查找备份记录
    /// @param filename - 文件名
    /// @returns 备份记录
    pub fn get_backup_software_by_filename(
        &self,
        filename: &str,
    ) -> AppResult<Option<BackupSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, epoch, pkgver, pkgrel, arch, subdirectory, full_path
             FROM backup_software WHERE filename=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![filename], |row| {
            Ok(BackupSoftware {
                id: Some(row.get(0)?),
                filename: row.get(1)?,
                epoch: row.get(2)?,
                pkgver: row.get(3)?,
                pkgrel: row.get(4)?,
                arch: row.get(5)?,
                subdirectory: row.get(6).ok(),
                full_path: row.get(7)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    /// 获取所有不重复的子目录列表
    /// @returns 子目录名称列表（不含空值）
    pub fn get_backup_subdirectories(&self) -> AppResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT subdirectory FROM backup_software
             WHERE subdirectory IS NOT NULL AND subdirectory != ''
             ORDER BY subdirectory"
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 按包名查询备份记录（模糊匹配文件名开头）
    /// @param pkgname - 软件包名称
    /// @returns 匹配的备份记录列表
    pub fn get_backup_entries_by_pkgname(
        &self,
        pkgname: &str,
    ) -> AppResult<Vec<BackupSoftwareEntry>> {
        let like_pattern = format!("{}%-", pkgname);
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, epoch, pkgver, pkgrel, arch, subdirectory, full_path
             FROM backup_software WHERE filename LIKE ?1 ORDER BY filename"
        )?;
        let rows = stmt.query_map(rusqlite::params![like_pattern], |row| {
            let filename: String = row.get(1)?;
            let pkgname = extract_pkgname(&filename);
            Ok(BackupSoftwareEntry {
                id: row.get(0)?,
                pkgname,
                filename,
                epoch: row.get(2)?,
                pkgver: row.get(3)?,
                pkgrel: row.get(4)?,
                arch: row.get(5)?,
                subdirectory: row.get(6).ok(),
                full_path: row.get(7)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }
}
