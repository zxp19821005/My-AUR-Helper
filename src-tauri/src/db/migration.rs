/**
 * migration.rs - 数据库迁移逻辑
 *
 * 管理已有数据库的表结构变更
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    /// 迁移 aur_info 表（删除无用列，转换 last_updated 类型）
    pub fn migrate_aur_info(&self) -> AppResult<()> {
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
            // 先尝试将纯数字文本直接转为整数
            self.conn.execute_batch(
                "UPDATE aur_info SET last_updated = CAST(last_updated AS INTEGER)
                 WHERE typeof(last_updated) = 'text' AND last_updated IS NOT NULL
                 AND last_updated NOT LIKE '%-%';"
            )?;
            // 再尝试将日期格式文本转为时间戳
            self.conn.execute_batch(
                "UPDATE aur_info SET last_updated = CAST(strftime('%s', last_updated) AS INTEGER)
                 WHERE typeof(last_updated) = 'text' AND last_updated IS NOT NULL
                 AND last_updated LIKE '%-%';"
            )?;
        }
        Ok(())
    }

    /// 迁移 software_info 表（重命名列，删除 created_at）
    pub fn migrate_software_info(&self) -> AppResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(software_info)")?;
        let columns: Vec<String> = stmt.query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();

        if columns.contains(&"package_type".to_string()) && !columns.contains(&"package_type_id".to_string()) {
            self.conn.execute_batch("ALTER TABLE software_info RENAME COLUMN package_type TO package_type_id;")?;
        }
        if columns.contains(&"checker_type".to_string()) && !columns.contains(&"checker_type_id".to_string()) {
            self.conn.execute_batch("ALTER TABLE software_info RENAME COLUMN checker_type TO checker_type_id;")?;
        }
        if columns.contains(&"created_at".to_string()) {
            self.conn.execute_batch("ALTER TABLE software_info DROP COLUMN created_at;")?;
        }
        Ok(())
    }
}
