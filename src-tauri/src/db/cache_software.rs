use crate::errors::AppResult; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database; // 数据库结构体

impl Database {
    /// 插入缓存软件记录
    /// @param cs - 缓存软件信息
    /// @returns 新插入记录的 ID
    pub fn insert_cache_software(&self, cs: &CacheSoftware) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO cache_software (software_id, filename, epoch, pkgrel, arch, cache_directory) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![cs.software_id, cs.filename, cs.epoch, cs.pkgrel, cs.arch, cs.cache_directory],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 根据软件包 ID 获取缓存记录
    /// @param software_id - 软件包 ID
    /// @returns 该包对应的所有缓存记录
    pub fn get_cache_software_by_pkg(&self, software_id: i64) -> AppResult<Vec<CacheSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, software_id, filename, epoch, pkgrel, arch, cache_directory FROM cache_software WHERE software_id=?1 ORDER BY filename"
        )?;
        let rows = stmt.query_map(rusqlite::params![software_id], |row| {
            Ok(CacheSoftware {
                id: Some(row.get(0)?),
                software_id: row.get(1)?,
                filename: row.get(2)?,
                epoch: row.get(3)?,
                pkgrel: row.get(4)?,
                arch: row.get(5)?,
                cache_directory: row.get(6)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 获取所有缓存记录（按缓存目录和文件名排序）
    /// @returns 所有缓存记录列表
    pub fn get_all_cache_software(&self) -> AppResult<Vec<CacheSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, software_id, filename, epoch, pkgrel, arch, cache_directory FROM cache_software ORDER BY cache_directory, filename"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CacheSoftware {
                id: Some(row.get(0)?),
                software_id: row.get(1)?,
                filename: row.get(2)?,
                epoch: row.get(3)?,
                pkgrel: row.get(4)?,
                arch: row.get(5)?,
                cache_directory: row.get(6)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 删除缓存记录
    /// @param id - 缓存记录 ID
    pub fn delete_cache_software(&self, id: i64) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM cache_software WHERE id=?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }
}
