/**
 * cache_software.rs - 缓存记录表 CRUD
 *
 * 功能：
 * - insert_cache_software: 插入缓存记录
 * - get_all_cache_software: 查询所有缓存记录（原始）
 * - get_all_cache_entries: 查询所有缓存记录（含解析出的包名，用于列表展示）
 * - clear_cache_software: 清空缓存表（重置自增 ID）
 * - delete_cache_software_batch: 批量删除缓存记录
 * - delete_cache_software: 删除单条缓存记录
 * - get_cache_software_by_filename: 按文件名查找缓存记录
 * - get_cache_source_dirs: 获取所有不重复的来源缓存目录
 */
use crate::errors::AppResult;

use crate::models::{CacheSoftware, CacheSoftwareEntry};

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
    /// 插入缓存软件记录
    /// @param cs - 缓存软件信息
    /// @returns 新插入记录的 ID
    pub fn insert_cache_software(&self, cs: &CacheSoftware) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO cache_software (software_id, filename, name, epoch, version, pkgrel, arch, size, source_dir, cache_directory, full_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                cs.cache_directory,
                cs.full_path
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 获取所有缓存记录（原始）
    /// @returns 所有缓存记录列表
    pub fn get_all_cache_software(&self) -> AppResult<Vec<CacheSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, software_id, filename, name, epoch, version, pkgrel, arch, size, source_dir, cache_directory, full_path, created_at, updated_at
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
                full_path: row.get(11)?,
                created_at: row.get(12).ok(),
                updated_at: row.get(13).ok(),
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 获取所有缓存记录（含解析出的包名，用于列表展示）
    /// 若 name 字段为空则从文件名解析包名
    /// @returns 缓存记录列表
    pub fn get_all_cache_entries(&self) -> AppResult<Vec<CacheSoftwareEntry>> {
        let entries = self.get_all_cache_software()?;
        Ok(entries
            .into_iter()
            .map(|e| {
                let pkgname = if e.name.is_empty() {
                    extract_pkgname(&e.filename)
                } else {
                    e.name.clone()
                };
                CacheSoftwareEntry {
                    id: e.id.unwrap_or(0),
                    pkgname,
                    filename: e.filename,
                    epoch: e.epoch,
                    version: e.version,
                    pkgrel: e.pkgrel,
                    arch: e.arch,
                    size: e.size,
                    source_dir: e.source_dir,
                    cache_directory: e.cache_directory,
                    software_id: e.software_id,
                    full_path: e.full_path,
                    created_at: e.created_at,
                    updated_at: e.updated_at,
                }
            })
            .collect())
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

    /// 批量删除缓存记录
    /// @param ids - 要删除的记录 ID 列表
    /// @returns 删除的记录数
    pub fn delete_cache_software_batch(&self, ids: &[i64]) -> AppResult<usize> {
        let mut count = 0;
        for id in ids {
            self.conn.execute(
                "DELETE FROM cache_software WHERE id=?1",
                rusqlite::params![id],
            )?;
            count += 1;
        }
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

    /// 根据文件名查找缓存记录
    /// @param filename - 文件名
    /// @returns 缓存记录
    pub fn get_cache_software_by_filename(
        &self,
        filename: &str,
    ) -> AppResult<Option<CacheSoftware>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, software_id, filename, name, epoch, version, pkgrel, arch, size, source_dir, cache_directory, full_path, created_at, updated_at
             FROM cache_software WHERE filename=?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![filename], |row| {
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
                full_path: row.get(11)?,
                created_at: row.get(12).ok(),
                updated_at: row.get(13).ok(),
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    /// 获取所有不重复的来源缓存目录名称列表
    /// @returns 来源目录名称列表（不含空值）
    pub fn get_cache_source_dirs(&self) -> AppResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT source_dir FROM cache_software
             WHERE source_dir IS NOT NULL AND source_dir != ''
             ORDER BY source_dir",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 按包名查询缓存记录（模糊匹配文件名开头或 name 字段匹配）
    /// @param pkgname - 软件包名称
    /// @returns 匹配的缓存记录列表
    pub fn get_cache_entries_by_pkgname(
        &self,
        pkgname: &str,
    ) -> AppResult<Vec<CacheSoftwareEntry>> {
        let like_pattern = format!("{}%-", pkgname);
        let mut stmt = self.conn.prepare(
            "SELECT id, software_id, filename, name, epoch, version, pkgrel, arch, size, source_dir, cache_directory, full_path, created_at, updated_at
             FROM cache_software WHERE name = ?1 OR filename LIKE ?2 ORDER BY filename",
        )?;
        let rows = stmt.query_map(rusqlite::params![pkgname, like_pattern], |row| {
            let filename: String = row.get(2)?;
            let name: String = row.get(3)?;
            let pkgname = if name.is_empty() {
                extract_pkgname(&filename)
            } else {
                name
            };
            Ok(CacheSoftwareEntry {
                id: row.get(0)?,
                software_id: row.get(1)?,
                pkgname,
                filename,
                epoch: row.get(4)?,
                version: row.get(5)?,
                pkgrel: row.get(6)?,
                arch: row.get(7)?,
                size: row.get(8)?,
                source_dir: row.get(9).ok(),
                cache_directory: row.get(10)?,
                full_path: row.get(11)?,
                created_at: row.get(12).ok(),
                updated_at: row.get(13).ok(),
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }
}
