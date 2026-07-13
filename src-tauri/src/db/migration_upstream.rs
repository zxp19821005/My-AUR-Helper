/**
 * migration_upstream.rs - upstream_info 表迁移逻辑
 *
 * 功能：
 * - 移除 upstream_url 列（已移至 software_info）
 * - 重命名 upstream_license 为 upstream_license_id
 * - 标准化 last_checked 列为 Unix 时间戳
 * - 将 upstream_license_id 从 INTEGER 改为 TEXT（存储 JSON 数组）
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    pub fn migrate_upstream_info(&self) -> AppResult<()> {
        let columns = self.get_table_columns("upstream_info")?;

        let has_upstream_url = columns.contains(&"upstream_url".to_string());
        let has_upstream_license = columns.contains(&"upstream_license".to_string());
        let has_upstream_license_id = columns.contains(&"upstream_license_id".to_string());
        let has_last_checked = columns.contains(&"last_checked".to_string());

        // 检查 license_id 是否为 INTEGER 类型（需要迁移）
        let license_col_type: String = self.conn.query_row(
            "SELECT COALESCE((SELECT typeof(upstream_license_id) FROM upstream_info LIMIT 1), 'null')",
            [],
            |row| row.get(0),
        ).unwrap_or_default();
        let needs_license_migration = license_col_type == "integer" || license_col_type == "null";

        if has_upstream_url || has_upstream_license || !has_upstream_license_id || needs_license_migration {
            let last_checked_type: String = self.conn.query_row(
                "SELECT COALESCE((SELECT typeof(\"last_checked\") FROM upstream_info LIMIT 1), 'null')",
                [],
                |row| row.get(0),
            ).unwrap_or_default();

            self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
            self.conn.execute_batch("DROP TABLE IF EXISTS upstream_info_new;")?;

            self.conn.execute_batch(
                "CREATE TABLE upstream_info_new (
                    software_id        INTEGER PRIMARY KEY,
                    upstream_version   TEXT,
                    upstream_license_id TEXT,
                    last_checked       INTEGER,
                    FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE
                );"
            )?;

            self.copy_upstream_data(
                has_upstream_url, has_upstream_license, has_upstream_license_id,
                has_last_checked, &last_checked_type, needs_license_migration,
            )?;

            self.conn.execute_batch("DROP TABLE upstream_info;")?;
            self.conn
                .execute_batch("ALTER TABLE upstream_info_new RENAME TO upstream_info;")?;
            self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        }

        Ok(())
    }

    fn copy_upstream_data(
        &self,
        _has_upstream_url: bool,
        has_upstream_license: bool,
        has_upstream_license_id: bool,
        has_last_checked: bool,
        last_checked_type: &str,
        needs_license_migration: bool,
    ) -> AppResult<()> {
        let is_text_timestamp = last_checked_type == "text";

        let license_sql = if needs_license_migration {
            // 将旧的整数 license_id 转换为 JSON 数组
            // 首先从 enum_licenses 获取 spdx_id
            "CASE WHEN upstream_license_id IS NOT NULL THEN 
                (SELECT '[' || json_quote(spdx_id) || ']' FROM enum_licenses WHERE id = upstream_license_id)
             ELSE NULL END"
        } else if has_upstream_license || !has_upstream_license_id {
            "CASE WHEN typeof(upstream_license) = 'integer' THEN CAST(upstream_license AS INTEGER) ELSE NULL END"
        } else {
            "upstream_license_id"
        };

        let time_sql = if is_text_timestamp {
            "CAST(strftime('%s', last_checked) AS INTEGER)"
        } else if has_last_checked {
            "last_checked"
        } else {
            "NULL"
        };

        let insert_sql = format!(
            "INSERT INTO upstream_info_new (software_id, upstream_version, upstream_license_id, last_checked)
             SELECT software_id, upstream_version, {license_sql}, {time_sql}
             FROM upstream_info;"
        );

        self.conn.execute_batch(&insert_sql)?;
        Ok(())
    }
}