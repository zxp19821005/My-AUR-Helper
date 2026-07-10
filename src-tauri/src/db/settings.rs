use crate::errors::AppResult; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database; // 数据库结构体

impl Database {
    /// 获取所有设置项（按分类和键名排序）
    /// @returns 所有设置项列表
    pub fn get_all_settings(&self) -> AppResult<Vec<Setting>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, value, description, category, created_at FROM settings ORDER BY category, key"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Setting {
                id: Some(row.get(0)?),
                key: row.get(1)?,
                value: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        let mut settings = Vec::new();
        for row in rows {
            settings.push(row?);
        }
        Ok(settings)
    }

    /// 根据键名获取单个设置
    /// @param key - 设置键名
    /// @returns 可选的设置项
    pub fn get_setting(&self, key: &str) -> AppResult<Option<Setting>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, value, description, category, created_at FROM settings WHERE key=?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![key], |row| {
            Ok(Setting {
                id: Some(row.get(0)?),
                key: row.get(1)?,
                value: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    /// 设置配置值（不存在则创建，存在则更新）
    /// @param key - 设置键名
    /// @param value - 设置值
    pub fn set_setting(&self, key: &str, value: &str) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }
}
