/**
 * migration_github_tag_cache.rs - github_tag_cache 表迁移
 *
 * 功能：
 * - 创建 github_tag_cache 表（幂等：IF NOT EXISTS）
 * - 仅在新建数据库时由 create_tables 覆盖；已有数据库通过此迁移补建
 */
use crate::errors::AppResult;

use super::Database;

/// github_tag_cache 表 DDL
const TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS github_tag_cache (
        owner            TEXT NOT NULL,
        repo             TEXT NOT NULL,
        last_synced_at   INTEGER NOT NULL,
        tag_count        INTEGER NOT NULL DEFAULT 0,
        data_json        TEXT NOT NULL DEFAULT '',
        cached_version   TEXT DEFAULT NULL,
        PRIMARY KEY (owner, repo)
    );";

/// 已有数据库补加 cached_version 列（幂等：COLUMN EXISTS 时忽略）
const ADD_COLUMN_SQL: &str = "
    ALTER TABLE github_tag_cache
    ADD COLUMN cached_version TEXT DEFAULT NULL;";

impl Database {
    /// 迁移：创建 github_tag_cache 表
    pub fn migrate_github_tag_cache(&self) -> AppResult<()> {
        self.conn.execute_batch(TABLE_SQL)?;
        // 兼容旧表：补加 cached_version 列（IF NOT EXISTS 在 SQLite 3.24+ 支持）
        let _ = self.conn.execute_batch(ADD_COLUMN_SQL);
        Ok(())
    }
}
