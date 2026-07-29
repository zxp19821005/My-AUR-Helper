/**
 * migration_backup.rs - backup_software 表结构迁移
 *
 * 迁移内容：
 * - 移除历史版本中的 software_id 外键字段
 * - 补齐 pkgver/full_path 字段
 * - 补齐 created_at/updated_at 时间戳字段（与 cache_software 保持 schema 设计一致）
 *
 * 迁移方式：检测列缺失后整表重建（SQLite 不支持带表达式默认值的 ADD COLUMN）
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    /// 迁移 backup_software 表：移除 software_id、补齐 pkgver/full_path/created_at/updated_at 字段
    /// @returns 迁移结果（表不存在或无需迁移时直接返回 Ok）
    pub(crate) fn migrate_backup_software(&self) -> AppResult<()> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='backup_software'",
            [],
            |row| row.get(0),
        )?;
        if !exists {
            return Ok(());
        }

        let columns = self.get_table_columns("backup_software")?;
        let has_software_id = columns.contains(&"software_id".to_string());
        let has_pkgver = columns.contains(&"pkgver".to_string());
        let has_full_path = columns.contains(&"full_path".to_string());
        let has_created_at = columns.contains(&"created_at".to_string());
        let has_updated_at = columns.contains(&"updated_at".to_string());

        // 所有目标字段齐全且无历史遗留字段时跳过迁移
        if !has_software_id && has_pkgver && has_full_path && has_created_at && has_updated_at {
            return Ok(());
        }

        log::info!("[migrate_backup_software] 重建 backup_software 表");

        let new_schema = "CREATE TABLE backup_software_new (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            filename     TEXT NOT NULL,
            epoch        INTEGER NOT NULL DEFAULT 0,
            pkgver       TEXT NOT NULL DEFAULT '',
            pkgrel       TEXT NOT NULL DEFAULT '1',
            arch         TEXT NOT NULL DEFAULT 'x86_64',
            subdirectory TEXT,
            full_path    TEXT NOT NULL DEFAULT '',
            created_at   TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
        );";

        let has_subdir = columns.contains(&"subdirectory".to_string());
        let subdir_expr = if has_subdir { "subdirectory" } else { "''" };
        // full_path 缺失时用 subdirectory/filename 拼接回填
        let full_path_expr = if has_full_path {
            "full_path".to_string()
        } else {
            format!(
                "CASE WHEN {subdir} IS NOT NULL AND {subdir} != '' THEN {subdir} || '/' || filename ELSE filename END",
                subdir = subdir_expr
            )
        };
        let pkgver_expr = if has_pkgver { "pkgver" } else { "''" };
        // 时间戳缺失时用当前时间回填
        let created_expr = if has_created_at {
            "created_at"
        } else {
            "datetime('now')"
        };
        let updated_expr = if has_updated_at {
            "updated_at"
        } else {
            "datetime('now')"
        };

        let insert_sql = format!(
            "INSERT INTO backup_software_new (id, filename, epoch, pkgver, pkgrel, arch, subdirectory, full_path, created_at, updated_at)
             SELECT id, filename, epoch, {pkgver}, pkgrel, arch, {subdir}, {fp}, {created}, {updated} FROM backup_software;",
            pkgver = pkgver_expr,
            subdir = subdir_expr,
            fp = full_path_expr,
            created = created_expr,
            updated = updated_expr
        );

        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
        self.conn
            .execute_batch("DROP TABLE IF EXISTS backup_software_new;")?;
        self.conn.execute_batch(new_schema)?;
        self.conn.execute_batch(&insert_sql)?;
        self.conn.execute_batch("DROP TABLE backup_software;")?;
        self.conn
            .execute_batch("ALTER TABLE backup_software_new RENAME TO backup_software;")?;
        // 重建随旧表一起删除的索引
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_backup_software_filename ON backup_software(filename);",
        )?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        log::info!("[migrate_backup_software] backup_software 表已重建");
        Ok(())
    }
}
