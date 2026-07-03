/**
 * migration.rs - 数据库迁移逻辑
 *
 * 管理已有数据库的表结构变更
 */
use anyhow::Result;

use super::Database;

impl Database {
    /// 迁移 aur_info 表（删除无用列，转换 last_updated 类型）
    pub fn migrate_aur_info(&self) -> Result<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(aur_info)")?;
        let columns: Vec<String> = stmt.query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();

        let old_cols = ["provides", "conflicts", "replaces", "votes", "popularity", "submitted_by", "maintainers"];
        for col in &old_cols {
            if columns.contains(&col.to_string()) {
                self.conn.execute_batch(&format!("ALTER TABLE aur_info DROP COLUMN {col};"))?;
            }
        }

        if columns.contains(&"last_updated".to_string()) {
            self.conn.execute_batch(
                "UPDATE aur_info SET last_updated = CAST(strftime('%s', last_updated) AS INTEGER)
                 WHERE typeof(last_updated) = 'text' AND last_updated IS NOT NULL;"
            )?;
        }
        Ok(())
    }
}
