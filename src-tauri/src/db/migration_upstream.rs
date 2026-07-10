/**
 * migration_upstream.rs - upstream_info 表迁移逻辑
 *
 * 功能：
 * - 移除 upstream_url 列（已移至 software_info）
 * - 重命名 upstream_license 为 upstream_license_id
 * - 标准化 last_checked 列为 Unix 时间戳
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

        if has_upstream_url || has_upstream_license || !has_upstream_license_id {
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
                    upstream_license_id INTEGER,
                    last_checked       INTEGER,
                    FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE,
                    FOREIGN KEY (upstream_license_id) REFERENCES enum_licenses(id)
                );"
            )?;

            self.copy_upstream_data(
                has_upstream_url, has_upstream_license, has_upstream_license_id,
                has_last_checked, &last_checked_type,
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
    ) -> AppResult<()> {
        let is_text_timestamp = last_checked_type == "text";

        let license_sql = if has_upstream_license || !has_upstream_license_id {
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