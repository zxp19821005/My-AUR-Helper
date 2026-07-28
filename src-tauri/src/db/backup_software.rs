/**
 * backup_software.rs - 备份记录表 CRUD
 *
 * 功能：
 * - insert_backup_software: 插入备份记录
 * - get_backup_software_by_pkg: 按软件包 ID 查询
 * - get_all_backup_software: 查询所有备份记录
 * - get_all_backup_entries: 查询所有备份记录（含软件包名称，用于列表展示）
 * - clear_backup_software: 清空备份表
 * - delete_backup_software_batch: 批量删除备份记录
 * - delete_backup_software: 删除单条备份记录
 * - get_backup_software_by_filename: 按文件名查找备份记录
 */
use crate::errors::AppResult;

use crate::models::*;

use super::Database;

impl Database {
    /// 插入备份软件记录
    /// @param bs - 备份软件信息
    /// @returns 新插入记录的 ID
    pub fn insert_backup_software(&self, bs: &BackupSoftware) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO backup_software (software_id, filename, epoch, pkgrel, arch, subdirectory) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![bs.software_id, bs.filename, bs.epoch, bs.pkgrel, bs.arch, bs.subdirectory],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 根据软件包 ID 获取备份记录
    /// @param software_id - 软件包 ID
    /// @returns 该包对应的所有备份记录
    pub fn get_backup_software_by_pkg(&self, software_id: i64) -> AppResult<Vec<BackupSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, software_id, filename, epoch, pkgrel, arch, subdirectory FROM backup_software WHERE software_id=?1 ORDER BY filename"
        )?;
        let rows = stmt.query_map(rusqlite::params![software_id], |row| {
            Ok(BackupSoftware {
                id: Some(row.get(0)?),
                software_id: row.get(1).ok(),
                filename: row.get(2)?,
                epoch: row.get(3)?,
                pkgrel: row.get(4)?,
                arch: row.get(5)?,
                subdirectory: row.get(6)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 获取所有备份记录（按软件包 ID 和文件名排序）
    /// @returns 所有备份记录列表
    pub fn get_all_backup_software(&self) -> AppResult<Vec<BackupSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, software_id, filename, epoch, pkgrel, arch, subdirectory FROM backup_software ORDER BY software_id, filename"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BackupSoftware {
                id: Some(row.get(0)?),
                software_id: row.get(1).ok(),
                filename: row.get(2)?,
                epoch: row.get(3)?,
                pkgrel: row.get(4)?,
                arch: row.get(5)?,
                subdirectory: row.get(6)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 获取所有备份记录（含软件包名称，用于列表展示）
    /// @returns 备份记录列表（含 pkgname）
    pub fn get_all_backup_entries(&self) -> AppResult<Vec<BackupSoftwareEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT b.id, b.software_id, s.pkgname, b.filename, b.epoch, b.pkgrel, b.arch, b.subdirectory
             FROM backup_software b
             LEFT JOIN software_info s ON b.software_id = s.software_id
             ORDER BY s.pkgname, b.filename"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BackupSoftwareEntry {
                id: row.get(0)?,
                software_id: row.get(1).ok(),
                pkgname: row.get(2).unwrap_or_default(),
                filename: row.get(3)?,
                epoch: row.get(4)?,
                pkgrel: row.get(5)?,
                arch: row.get(6)?,
                subdirectory: row.get(7)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 清空备份表
    /// @returns 删除的记录数
    pub fn clear_backup_software(&self) -> AppResult<usize> {
        let count = self.conn.execute("DELETE FROM backup_software", [])?;
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
            "SELECT id, software_id, filename, epoch, pkgrel, arch, subdirectory FROM backup_software WHERE filename=?1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![filename], |row| {
            Ok(BackupSoftware {
                id: Some(row.get(0)?),
                software_id: row.get(1).ok(),
                filename: row.get(2)?,
                epoch: row.get(3)?,
                pkgrel: row.get(4)?,
                arch: row.get(5)?,
                subdirectory: row.get(6)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }
}
