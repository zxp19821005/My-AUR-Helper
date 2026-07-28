/**
 * cache_software.rs - 缓存记录表 CRUD
 *
 * 功能：
 * - insert_cache_software: 插入缓存记录
 * - get_all_cache_software: 查询所有缓存记录
 * - clear_cache_software: 清空缓存表（重置自增 ID）
 * - delete_cache_software: 删除单条缓存记录
 */
use crate::errors::AppResult;

use crate::models::*;

use super::Database;

impl Database {
    /// 插入缓存软件记录
    /// @param cs - 缓存软件信息
    /// @returns 新插入记录的 ID
    pub fn insert_cache_software(&self, cs: &CacheSoftware) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO cache_software (software_id, filename, name, epoch, version, pkgrel, arch, size, source_dir, cache_directory)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                cs.software_id,
                cs.filename,
                cs.name,
                cs.epoch,
                cs.version,
                cs.pkgrel,
                cs.arch,
                cs.size,
                cs.source_dir,
                cs.cache_directory
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 获取所有缓存记录
    /// @returns 所有缓存记录列表
    pub fn get_all_cache_software(&self) -> AppResult<Vec<CacheSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, software_id, filename, name, epoch, version, pkgrel, arch, size, source_dir, cache_directory
             FROM cache_software ORDER BY name, version",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CacheSoftware {
                id: Some(row.get(0)?),
                software_id: row.get(1)?,
                filename: row.get(2)?,
                name: row.get(3)?,
                epoch: row.get(4)?,
                version: row.get(5)?,
                pkgrel: row.get(6)?,
                arch: row.get(7)?,
                size: row.get(8)?,
                source_dir: row.get(9).ok(),
                cache_directory: row.get(10)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 清空缓存表并重置自增 ID
    /// @returns 删除的记录数
    pub fn clear_cache_software(&self) -> AppResult<usize> {
        let count = self.conn.execute("DELETE FROM cache_software", [])?;
        self.conn.execute(
            "DELETE FROM sqlite_sequence WHERE name='cache_software'",
            [],
        )?;
        Ok(count)
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
