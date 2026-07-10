use crate::errors::AppResult; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database; // 数据库结构体

impl Database {
    /// 获取所有编程语言记录（按名称排序）
    /// @returns 所有编程语言列表
    pub fn get_all_languages(&self) -> AppResult<Vec<EnumProgrammingLanguage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, short_name FROM enum_programming_languages ORDER BY name"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(EnumProgrammingLanguage {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                short_name: row.get(2)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 插入或更新编程语言记录（按 name 去重）
    /// @param lang - 编程语言信息
    /// @returns 新插入或更新的记录 ID
    pub fn upsert_language(&self, lang: &EnumProgrammingLanguage) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO enum_programming_languages (name, short_name)
             VALUES (?1, ?2)
             ON CONFLICT(name) DO UPDATE SET
                short_name=excluded.short_name",
            rusqlite::params![lang.name, lang.short_name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 删除编程语言记录
    /// @param name - 要删除的编程语言名称
    pub fn delete_language(&self, name: &str) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM enum_programming_languages WHERE name=?1",
            rusqlite::params![name],
        )?;
        Ok(())
    }
}