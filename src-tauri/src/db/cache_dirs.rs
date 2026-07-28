/**
 * cache_dirs.rs - 缓存目录配置表 CRUD
 *
 * 功能：
 * - get_all_cache_dirs: 获取所有缓存目录配置
 * - get_enabled_cache_dirs: 获取所有启用的缓存目录
 * - insert_cache_dir: 添加缓存目录
 * - update_cache_dir: 更新缓存目录
 * - delete_cache_dir: 删除缓存目录
 */
use crate::errors::AppResult;
use crate::models::CacheDir;
use super::Database;

impl Database {
    /// 获取所有缓存目录配置（按排序顺序）
    pub fn get_all_cache_dirs(&self) -> AppResult<Vec<CacheDir>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, is_enabled, sort_order FROM cache_dirs ORDER BY sort_order, id"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CacheDir {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                path: row.get(2)?,
                is_enabled: row.get::<_, i32>(3)? != 0,
                sort_order: row.get(4)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 获取所有启用的缓存目录
    pub fn get_enabled_cache_dirs(&self) -> AppResult<Vec<CacheDir>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, is_enabled, sort_order FROM cache_dirs WHERE is_enabled=1 ORDER BY sort_order, id"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CacheDir {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                path: row.get(2)?,
                is_enabled: row.get::<_, i32>(3)? != 0,
                sort_order: row.get(4)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 添加缓存目录
    pub fn insert_cache_dir(&self, cache_dir: &CacheDir) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO cache_dirs (name, path, is_enabled, sort_order) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                cache_dir.name,
                cache_dir.path,
                cache_dir.is_enabled as i32,
                cache_dir.sort_order,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 更新缓存目录
    pub fn update_cache_dir(&self, cache_dir: &CacheDir) -> AppResult<()> {
        self.conn.execute(
            "UPDATE cache_dirs SET name=?1, path=?2, is_enabled=?3, sort_order=?4 WHERE id=?5",
            rusqlite::params![
                cache_dir.name,
                cache_dir.path,
                cache_dir.is_enabled as i32,
                cache_dir.sort_order,
                cache_dir.id,
            ],
        )?;
        Ok(())
    }

    /// 删除缓存目录
    pub fn delete_cache_dir(&self, id: i64) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM cache_dirs WHERE id=?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }
}
