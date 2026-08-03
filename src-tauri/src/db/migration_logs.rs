/**
 * migration_logs.rs - 删除 logs 表迁移
 *
 * 迁移内容：
 * - 删除多余的 logs 表（日志已改为文件存储）
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    /// 删除 logs 表（如果存在）
    pub(crate) fn migrate_drop_logs_table(&self) -> AppResult<()> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='logs'",
            [],
            |row| row.get(0),
        )?;
        if exists {
            self.conn.execute("DROP TABLE IF EXISTS logs", [])?;
            log::info!("[migrate_drop_logs_table] logs 表已删除");
        }
        Ok(())
    }
}
