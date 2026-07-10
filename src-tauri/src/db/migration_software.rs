/**
 * migration_software.rs - software_info 表迁移逻辑
 *
 * 功能：
 * - 重命名 package_type/checker_type 为 _id 后缀
 * - 移除 created_at 列
 * - 添加 version_extract_regex 列
 * - 移除 license_id 外键约束
 * - 将 language_id 从 INTEGER 改为 TEXT (JSON 数组)
 */
use crate::errors::AppResult;

use super::Database;

const SOFTWARE_NEW_SCHEMA: &str = "
    CREATE TABLE software_info_new (
        software_id             INTEGER PRIMARY KEY AUTOINCREMENT,
        pkgname                 TEXT NOT NULL UNIQUE,
        upstream_url            TEXT,
        package_type_id         INTEGER NOT NULL DEFAULT 1,
        checker_type_id         INTEGER NOT NULL DEFAULT 7,
        is_outdated             INTEGER NOT NULL DEFAULT 0,
        check_test_versions     INTEGER NOT NULL DEFAULT 0,
        check_binary_files      INTEGER NOT NULL DEFAULT 0,
        auto_check_enabled      INTEGER NOT NULL DEFAULT 1,
        language_id             TEXT DEFAULT '[]',
        version_extract_regex   TEXT
    );";

const SOFTWARE_INSERT_SQL: &str = "
    INSERT INTO software_info_new
    SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id,
           is_outdated, check_test_versions, check_binary_files, auto_check_enabled,
           CASE WHEN language_id IS NULL THEN '[]'
                ELSE '[' || CAST(language_id AS TEXT) || ']'
           END,
           version_extract_regex
    FROM software_info;";

const SOFTWARE_INDEX_SQL: &str = "
    CREATE INDEX IF NOT EXISTS idx_software_pkgname ON software_info(pkgname);
    CREATE INDEX IF NOT EXISTS idx_software_outdated ON software_info(is_outdated);";

impl Database {
    pub fn migrate_software_info(&self) -> AppResult<()> {
        let columns = self.get_table_columns("software_info")?;

        if columns.contains(&"package_type".to_string())
            && !columns.contains(&"package_type_id".to_string())
        {
            self.conn.execute_batch(
                "ALTER TABLE software_info RENAME COLUMN package_type TO package_type_id;",
            )?;
        }
        if columns.contains(&"checker_type".to_string())
            && !columns.contains(&"checker_type_id".to_string())
        {
            self.conn.execute_batch(
                "ALTER TABLE software_info RENAME COLUMN checker_type TO checker_type_id;",
            )?;
        }
        if columns.contains(&"created_at".to_string()) {
            self.conn
                .execute_batch("ALTER TABLE software_info DROP COLUMN created_at;")?;
        }
        if !columns.contains(&"version_extract_regex".to_string()) {
            self.conn.execute_batch(
                "ALTER TABLE software_info ADD COLUMN version_extract_regex TEXT;",
            )?;
        }

        if columns.contains(&"license_id".to_string()) {
            let fk_has_license: bool = self.conn.query_row(
                "SELECT COUNT(*) > 0 FROM pragma_foreign_key_list('software_info') WHERE \"from\" = 'license_id'",
                [],
                |row| row.get(0),
            )?;
            if fk_has_license {
                self.rebuild_software_info_without_license_fk()?;
            } else {
                self.conn
                    .execute_batch("ALTER TABLE software_info DROP COLUMN license_id;")?;
            }
        }

        self.migrate_language_id_to_json()?;

        Ok(())
    }

    fn migrate_language_id_to_json(&self) -> AppResult<()> {
        let col_type: Option<String> = self.conn.query_row(
            "SELECT type FROM pragma_table_info('software_info') WHERE name='language_id'",
            [],
            |row| row.get(0),
        ).ok();

        if col_type.is_none() || col_type.as_deref() == Some("TEXT") {
            return Ok(());
        }

        self.rebuild_table("software_info", SOFTWARE_NEW_SCHEMA, SOFTWARE_INSERT_SQL, SOFTWARE_INDEX_SQL)
    }

    fn rebuild_software_info_without_license_fk(&self) -> AppResult<()> {
        self.rebuild_table("software_info", SOFTWARE_NEW_SCHEMA, SOFTWARE_INSERT_SQL, SOFTWARE_INDEX_SQL)
    }

    /// 通用表重建工具方法
    fn rebuild_table(
        &self,
        table_name: &str,
        create_sql: &str,
        insert_sql: &str,
        index_sql: &str,
    ) -> AppResult<()> {
        let new_table = format!("{table_name}_new");
        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
        self.conn.execute_batch(&format!("DROP TABLE IF EXISTS {new_table};"))?;
        self.conn.execute_batch(create_sql)?;
        self.conn.execute_batch(insert_sql)?;
        self.conn.execute_batch(&format!("DROP TABLE {table_name};"))?;
        self.conn.execute_batch(&format!("ALTER TABLE {new_table} RENAME TO {table_name};"))?;
        self.conn.execute_batch(index_sql)?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        Ok(())
    }
}