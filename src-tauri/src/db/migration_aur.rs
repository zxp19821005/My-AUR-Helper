/**
 * migration_aur.rs - aur_info 表迁移逻辑
 *
 * 功能：
 * - 移除已废弃的列（provides, conflicts, replaces 等）
 * - 标准化 last_updated 列为 Unix 时间戳
 * - 将 license_id 从 INTEGER 改为 TEXT（存储 JSON 数组）
 * - 补齐 last_sync_error 列（AUR 同步失败原因，供列表「AUR更新失败」筛选使用）
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    pub fn migrate_aur_info(&self) -> AppResult<()> {
        let columns = self.get_table_columns("aur_info")?;

        let old_cols = [
            "provides",
            "conflicts",
            "replaces",
            "votes",
            "popularity",
            "submitted_by",
            "maintainers",
        ];
        for col in &old_cols {
            if columns.contains(&col.to_string()) {
                self.conn
                    .execute_batch(&format!("ALTER TABLE aur_info DROP COLUMN {col};"))?;
            }
        }

        if columns.contains(&"last_updated".to_string()) {
            self.conn.execute_batch(
                "UPDATE aur_info SET last_updated = CAST(last_updated AS INTEGER)
                 WHERE typeof(last_updated) = 'text' AND last_updated IS NOT NULL
                 AND last_updated NOT LIKE '%-%';",
            )?;
            self.conn.execute_batch(
                "UPDATE aur_info SET last_updated = CAST(strftime('%s', last_updated) AS INTEGER)
                 WHERE typeof(last_updated) = 'text' AND last_updated IS NOT NULL
                 AND last_updated LIKE '%-%';",
            )?;
        }

        // 检查 license_id 是否为 INTEGER 类型（需要迁移）
        let license_col_type: String = self
            .conn
            .query_row(
                "SELECT COALESCE((SELECT typeof(license_id) FROM aur_info LIMIT 1), 'null')",
                [],
                |row| row.get(0),
            )
            .unwrap_or_default();

        if license_col_type == "integer" || license_col_type == "null" {
            // 需要迁移 license_id 从 INTEGER 到 TEXT
            self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
            self.conn
                .execute_batch("DROP TABLE IF EXISTS aur_info_new;")?;

            self.conn.execute_batch(
                "CREATE TABLE aur_info_new (
                    software_id     INTEGER PRIMARY KEY,
                    pkgdesc         TEXT,
                    aur_version     TEXT,
                    license_id      TEXT,
                    last_updated    INTEGER,
                    depends         TEXT,
                    makedepends     TEXT,
                    optdepends      TEXT,
                    out_of_date     INTEGER,
                    FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE
                );"
            )?;

            // 将旧的整数 license_id 转换为 JSON 数组
            self.conn.execute_batch(
                "INSERT INTO aur_info_new (software_id, pkgdesc, aur_version, license_id, last_updated, depends, makedepends, optdepends, out_of_date)
                 SELECT software_id, pkgdesc, aur_version,
                        CASE WHEN license_id IS NOT NULL THEN 
                            (SELECT '[' || json_quote(spdx_id) || ']' FROM enum_licenses WHERE id = aur_info.license_id)
                        ELSE NULL END,
                        last_updated, depends, makedepends, optdepends, out_of_date
                 FROM aur_info;"
            )?;

            self.conn.execute_batch("DROP TABLE aur_info;")?;
            self.conn
                .execute_batch("ALTER TABLE aur_info_new RENAME TO aur_info;")?;
            self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        }

        self.migrate_aur_error_column()?;

        Ok(())
    }

    /// 补齐 last_sync_error 列（幂等）
    ///
    /// 老库建表时没有该列，`CREATE TABLE IF NOT EXISTS` 不会补列，
    /// 故在此用 ALTER TABLE 增量添加；列已存在时直接跳过。
    pub fn migrate_aur_error_column(&self) -> AppResult<()> {
        let columns = self.get_table_columns("aur_info")?;
        if !columns.contains(&"last_sync_error".to_string()) {
            self.conn.execute_batch(
                "ALTER TABLE aur_info ADD COLUMN last_sync_error TEXT DEFAULT NULL;",
            )?;
        }
        Ok(())
    }
}
