/**
 * migration_enum.rs - 枚举表迁移逻辑
 *
 * 功能：
 * - enum_programming_languages 表简化（id, name, short_name）
 * - enum_licenses 表简化（id, spdx_id, full_name）
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    /// 迁移 enum_programming_languages 表，简化为 id, name, short_name
    pub fn migrate_enum_programming_languages(&self) -> AppResult<()> {
        let columns = self.get_table_columns("enum_programming_languages")?;

        let needs_migration = columns.contains(&"description".to_string())
            || columns.contains(&"file_extensions".to_string())
            || columns.contains(&"build_system".to_string())
            || columns.contains(&"build_command".to_string());

        if !needs_migration {
            if !columns.contains(&"short_name".to_string()) {
                self.conn.execute_batch(
                    "ALTER TABLE enum_programming_languages ADD COLUMN short_name TEXT;",
                )?;
            }
            return Ok(());
        }

        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
        self.conn.execute_batch(
            "CREATE TABLE enum_programming_languages_new (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,
                short_name  TEXT
            );",
        )?;

        let has_short_name = columns.contains(&"short_name".to_string());
        if has_short_name {
            self.conn.execute_batch(
                "INSERT INTO enum_programming_languages_new (id, name, short_name)
                 SELECT id, name, short_name FROM enum_programming_languages;",
            )?;
        } else {
            self.conn.execute_batch(
                "INSERT INTO enum_programming_languages_new (id, name, short_name)
                 SELECT id, name, SUBSTR(name, 1, 2) FROM enum_programming_languages;",
            )?;
        }

        self.conn.execute_batch("DROP TABLE enum_programming_languages;")?;
        self.conn.execute_batch(
            "ALTER TABLE enum_programming_languages_new RENAME TO enum_programming_languages;",
        )?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        Ok(())
    }

    /// 迁移 enum_licenses 表，简化为 id, spdx_id, full_name
    pub fn migrate_enum_licenses(&self) -> AppResult<()> {
        let columns = self.get_table_columns("enum_licenses")?;

        if columns.len() > 3 {
            self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
            self.conn.execute_batch(
                "CREATE TABLE enum_licenses_new (
                    id        INTEGER PRIMARY KEY AUTOINCREMENT,
                    spdx_id   TEXT NOT NULL UNIQUE,
                    full_name TEXT NOT NULL
                );",
            )?;
            self.conn.execute_batch(
                "INSERT INTO enum_licenses_new (id, spdx_id, full_name)
                 SELECT id, spdx_id, full_name FROM enum_licenses;",
            )?;
            self.conn.execute_batch("DROP TABLE enum_licenses;")?;
            self.conn
                .execute_batch("ALTER TABLE enum_licenses_new RENAME TO enum_licenses;")?;
            self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        }
        Ok(())
    }
}