/**
 * migration_aur.rs - aur_info 表迁移逻辑
 *
 * 功能：
 * - 移除已废弃的列（provides, conflicts, replaces 等）
 * - 标准化 last_updated 列为 Unix 时间戳
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    pub fn migrate_aur_info(&self) -> AppResult<()> {
        let columns = self.get_table_columns("aur_info")?;

        let old_cols = [
            "provides", "conflicts", "replaces", "votes",
            "popularity", "submitted_by", "maintainers",
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
        Ok(())
    }
}