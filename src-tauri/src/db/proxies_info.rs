use crate::errors::AppResult; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database; // 数据库结构体

impl Database {
    /// 插入代理记录（忽略重复 URL）
    /// @param proxy - 代理信息
    /// @returns 新插入记录的 ID（如果已存在则返回 0）
    pub fn insert_proxy(&self, proxy: &ProxyInfo) -> AppResult<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO proxies_info (proxy_name, proxy_type, url, is_active) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![proxy.proxy_name, proxy.proxy_type.as_str(), proxy.url, proxy.is_active as i32],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 获取所有代理记录（按名称排序）
    /// @returns 所有代理信息列表
    pub fn get_all_proxies(&self) -> AppResult<Vec<ProxyInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT proxy_id, proxy_name, proxy_type, url, is_active FROM proxies_info ORDER BY proxy_name"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProxyInfo {
                proxy_id: Some(row.get(0)?),
                proxy_name: row.get(1)?,
                proxy_type: ProxyType::parse_from(&row.get::<_, String>(2)?), // 字符串转枚举
                url: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0, // 整数转布尔
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 获取指定类型的所有已启用代理
    /// @param proxy_type - 代理类型
    /// @returns 已启用且匹配类型的代理列表
    pub fn get_active_proxies(&self, proxy_type: &ProxyType) -> AppResult<Vec<ProxyInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT proxy_id, proxy_name, proxy_type, url, is_active FROM proxies_info WHERE is_active=1 AND proxy_type=?1 ORDER BY proxy_name"
        )?;
        let rows = stmt.query_map(rusqlite::params![proxy_type.as_str()], |row| {
            Ok(ProxyInfo {
                proxy_id: Some(row.get(0)?),
                proxy_name: row.get(1)?,
                proxy_type: ProxyType::parse_from(&row.get::<_, String>(2)?),
                url: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 更新代理的启用状态
    /// @param proxy_id - 代理 ID
    /// @param is_active - 是否启用
    pub fn update_proxy_active(&self, proxy_id: i64, is_active: bool) -> AppResult<()> {
        self.conn.execute(
            "UPDATE proxies_info SET is_active=?1 WHERE proxy_id=?2",
            rusqlite::params![is_active as i32, proxy_id],
        )?;
        Ok(())
    }

    /// 删除代理记录
    /// @param proxy_id - 代理 ID
    pub fn delete_proxy(&self, proxy_id: i64) -> AppResult<()> {
        self.conn.execute(
            "DELETE FROM proxies_info WHERE proxy_id=?1",
            rusqlite::params![proxy_id],
        )?;
        Ok(())
    }
}
