/**
 * migration_cache.rs - cache_software 表结构迁移
 *
 * 迁移内容：
 * - 移除历史版本中 software_id 上的外键约束（外键导致 software_id=0 时 INSERT 失败）
 * - 移除 software_id/size/source_dir 字段
 * - 补齐 name/pkgver/full_path 字段
 * - 移除 created_at/updated_at 时间戳字段
 * - 将 version 字段重命名为 pkgver（与 backup_software 表一致）
 *
 * 迁移方式：检测列缺失或外键残留后整表重建（SQLite 不支持 DROP CONSTRAINT
 * 和带表达式默认值的 ADD COLUMN）
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    /// 迁移 cache_software 表：移除 software_id 外键，补齐 name/pkgver/size/source_dir/full_path/created_at/updated_at 字段，将 version 重命名为 pkgver
    /// @returns 迁移结果（表不存在或无需迁移时直接返回 Ok）
    pub(crate) fn migrate_cache_software(&self) -> AppResult<()> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='cache_software'",
            [],
            |row| row.get(0),
        )?;
        if !exists {
            return Ok(());
        }

        let columns = self.get_table_columns("cache_software")?;
        let has_name = columns.contains(&"name".to_string());
        let has_pkgver = columns.contains(&"pkgver".to_string());
        let has_version = columns.contains(&"version".to_string());
        let has_full_path = columns.contains(&"full_path".to_string());
        let has_software_id = columns.contains(&"software_id".to_string());
        let has_size = columns.contains(&"size".to_string());
        let has_source_dir = columns.contains(&"source_dir".to_string());
        let has_created_at = columns.contains(&"created_at".to_string());
        let has_updated_at = columns.contains(&"updated_at".to_string());
        let all_columns_ok = has_name
            && has_pkgver
            && has_full_path
            && !has_software_id
            && !has_size
            && !has_source_dir
            && !has_created_at
            && !has_updated_at
            && !has_version;

        // 检测是否还有外键约束（老版本 CREATE TABLE 带了 FOREIGN KEY）
        let fk_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_foreign_key_list('cache_software')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let has_foreign_key = fk_count > 0;

        // 如果所有新字段都已存在且没有外键约束，跳过迁移
        if all_columns_ok && !has_foreign_key {
            return Ok(());
        }

        log::info!(
            "[migrate_cache_software] 重建 cache_software 表（字段齐全={}，外键约束={}）",
            all_columns_ok,
            has_foreign_key
        );

        let new_schema = "CREATE TABLE cache_software_new (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            name            TEXT NOT NULL DEFAULT '',
            filename        TEXT NOT NULL,
            epoch           INTEGER NOT NULL DEFAULT 0,
            pkgver          TEXT NOT NULL DEFAULT '',
            pkgrel          TEXT NOT NULL DEFAULT '1',
            arch            TEXT NOT NULL DEFAULT 'x86_64',
            cache_directory TEXT NOT NULL DEFAULT '',
            full_path       TEXT NOT NULL DEFAULT ''
        );";

        let name_expr = if has_name { "name" } else { "''" };
        // version 字段需要迁移到 pkgver，如果旧表有 version 则读取，否则用 pkgver 或空字符串
        let pkgver_expr = if has_pkgver {
            "pkgver"
        } else if has_version {
            "version"
        } else {
            "''"
        };
        // full_path 缺失时用 cache_directory/filename 拼接回填
        let full_path_expr = if has_full_path {
            "full_path"
        } else {
            "CASE WHEN cache_directory != '' THEN cache_directory || '/' || filename ELSE filename END"
        };

        let insert_sql = format!(
            "INSERT INTO cache_software_new (id, name, filename, epoch, pkgver, pkgrel, arch, cache_directory, full_path)
             SELECT id, {name}, filename, epoch, {pkgver}, pkgrel, arch, cache_directory, {fp}
             FROM cache_software;",
            name = name_expr,
            pkgver = pkgver_expr,
            fp = full_path_expr
        );

        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
        self.conn
            .execute_batch("DROP TABLE IF EXISTS cache_software_new;")?;
        self.conn.execute_batch(new_schema)?;
        self.conn.execute_batch(&insert_sql)?;
        self.conn.execute_batch("DROP TABLE cache_software;")?;
        self.conn
            .execute_batch("ALTER TABLE cache_software_new RENAME TO cache_software;")?;
        // 重建随旧表一起删除的索引
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_cache_software_name ON cache_software(name);",
        )?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        log::info!("[migrate_cache_software] cache_software 表已重建");
        Ok(())
    }
}