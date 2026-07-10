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

    /// 根据语言名称获取或创建语言记录
    /// 如果语言不存在，则自动创建（简称取名称的前两个字符）
    /// @param name - 编程语言名称
    /// @returns 语言 ID
    pub fn get_or_create_language_id(&self, name: &str) -> AppResult<i64> {
        // 先尝试查询
        let mut stmt = self.conn.prepare(
            "SELECT id FROM enum_programming_languages WHERE name=?1"
        )?;
        let existing: Option<i64> = stmt
            .query_map(rusqlite::params![name], |row| row.get(0))?
            .next()
            .transpose()?;

        if let Some(id) = existing {
            return Ok(id);
        }

        // 不存在则创建
        let short_name: String = name.chars().take(2).collect();
        self.conn.execute(
            "INSERT INTO enum_programming_languages (name, short_name) VALUES (?1, ?2)",
            rusqlite::params![name, short_name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 批量处理语言名称列表，返回对应的语言 ID 列表
    /// 对于不存在的语言，会自动创建
    /// @param names - 编程语言名称列表
    /// @returns 语言 ID 列表
    pub fn resolve_language_ids(&self, names: &[String]) -> AppResult<Vec<i64>> {
        let mut ids = Vec::new();
        for name in names {
            match self.get_or_create_language_id(name) {
                Ok(id) => ids.push(id),
                Err(e) => {
                    log::warn!("[语言ID解析] 无法为 '{}' 创建语言记录: {}", name, e);
                }
            }
        }
        Ok(ids)
    }
}