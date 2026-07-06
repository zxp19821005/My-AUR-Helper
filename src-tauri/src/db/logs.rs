use crate::errors::AppResult; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database;  // 数据库结构体

impl Database {
    /// 插入日志记录
    /// @param level - 日志级别（INFO/WARN/ERROR/DEBUG）
    /// @param message - 日志消息内容
    /// @param module - 来源模块名称
    pub fn insert_log(&self, level: &str, message: &str, module: Option<&str>) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO logs (level, message, module) VALUES (?1, ?2, ?3)",
            rusqlite::params![level, message, module],
        )?;
        Ok(())
    }

    /// 获取最近的日志记录
    /// @param limit - 返回的最大条数
    /// @returns 日志条目列表（按时间降序，最新的在前）
    pub fn get_logs(&self, limit: i64) -> AppResult<Vec<LogEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, level, message, module, created_at FROM logs ORDER BY created_at DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(rusqlite::params![limit], |row| {
            Ok(LogEntry {
                id: Some(row.get(0)?),
                level: row.get(1)?,
                message: row.get(2)?,
                module: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    /// 清空所有日志记录
    pub fn clear_logs(&self) -> AppResult<()> {
        self.conn.execute("DELETE FROM logs", [])?;
        Ok(())
    }
}
