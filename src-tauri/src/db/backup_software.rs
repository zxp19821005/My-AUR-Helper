use anyhow::Result; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database;  // 数据库结构体

impl Database {
    /// 插入备份软件记录
    /// @param bs - 备份软件信息
    /// @returns 新插入记录的 ID
    pub fn insert_backup_software(&self, bs: &BackupSoftware) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO backup_software (software_id, filename, epoch, pkgrel, arch, subdirectory) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![bs.software_id, bs.filename, bs.epoch, bs.pkgrel, bs.arch, bs.subdirectory],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 根据软件包 ID 获取备份记录
    /// @param software_id - 软件包 ID
    /// @returns 该包对应的所有备份记录
    pub fn get_backup_software_by_pkg(&self, software_id: i64) -> Result<Vec<BackupSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, software_id, filename, epoch, pkgrel, arch, subdirectory FROM backup_software WHERE software_id=?1 ORDER BY filename"
        )?;
        let rows = stmt.query_map(rusqlite::params![software_id], |row| {
            Ok(BackupSoftware {
                id: Some(row.get(0)?),
                software_id: row.get(1)?,
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
    pub fn get_all_backup_software(&self) -> Result<Vec<BackupSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, software_id, filename, epoch, pkgrel, arch, subdirectory FROM backup_software ORDER BY software_id, filename"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BackupSoftware {
                id: Some(row.get(0)?),
                software_id: row.get(1)?,
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

    /// 删除备份记录
    /// @param id - 备份记录 ID
    pub fn delete_backup_software(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM backup_software WHERE id=?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }
}
