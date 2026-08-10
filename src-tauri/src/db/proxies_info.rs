use crate::errors::AppResult; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database; // 数据库结构体

impl Database {
    /// 插入代理记录（忽略重复 URL）
    /// @param proxy - 代理信息
    /// @returns 新插入记录的 ID（如果已存在则返回 0）
    pub fn insert_proxy(&self, proxy: &ProxyInfo) -> AppResult<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO proxies_info (proxy_name, proxy_type, url, is_active, strip_target_protocol) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![proxy.proxy_name, proxy.proxy_type.as_str(), proxy.url, proxy.is_active as i32, proxy.strip_target_protocol as i32],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 获取所有代理记录并附带最新测试统计（按名称排序）
    /// 通过 LEFT JOIN 关联 proxies_test 获取最新测试记录
    pub fn get_all_proxies_with_stats(&self) -> AppResult<Vec<ProxyInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.proxy_id, p.proxy_name, p.proxy_type, p.url, p.is_active, \
             COALESCE(t.success_count, 0) as success_count, \
             COALESCE(t.fail_count, 0) as fail_count, \
             t.avg_latency, \
             t.last_test_status, \
             p.strip_target_protocol \
             FROM proxies_info p \
             LEFT JOIN ( \
                 SELECT proxy_id, success_count, fail_count, avg_latency, last_test_status \
                 FROM proxies_test t1 \
                 WHERE test_time = (SELECT MAX(test_time) FROM proxies_test t2 WHERE t2.proxy_id = t1.proxy_id) \
             ) t ON p.proxy_id = t.proxy_id \
             ORDER BY p.proxy_name"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProxyInfo {
                proxy_id: Some(row.get(0)?),
                proxy_name: row.get(1)?,
                proxy_type: ProxyType::parse_from(&row.get::<_, String>(2)?), // 字符串转枚举
                url: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0, // 整数转布尔
                success_count: row.get(5)?,
                fail_count: row.get(6)?,
                avg_latency: row.get(7)?,
                last_test_status: row.get(8)?,
                strip_target_protocol: row.get(9)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 获取所有代理记录（按名称排序，兼容旧接口，无测试统计）
    pub fn get_all_proxies(&self) -> AppResult<Vec<ProxyInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT proxy_id, proxy_name, proxy_type, url, is_active, strip_target_protocol FROM proxies_info ORDER BY proxy_name"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProxyInfo {
                proxy_id: Some(row.get(0)?),
                proxy_name: row.get(1)?,
                proxy_type: ProxyType::parse_from(&row.get::<_, String>(2)?), // 字符串转枚举
                url: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0, // 整数转布尔
                success_count: 0,
                fail_count: 0,
                avg_latency: None,
                last_test_status: None,
                strip_target_protocol: row.get(5)?,
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
            "SELECT proxy_id, proxy_name, proxy_type, url, is_active, strip_target_protocol FROM proxies_info WHERE is_active=1 AND proxy_type=?1 ORDER BY proxy_name"
        )?;
        let rows = stmt.query_map(rusqlite::params![proxy_type.as_str()], |row| {
            Ok(ProxyInfo {
                proxy_id: Some(row.get(0)?),
                proxy_name: row.get(1)?,
                proxy_type: ProxyType::parse_from(&row.get::<_, String>(2)?),
                url: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                success_count: 0,
                fail_count: 0,
                avg_latency: None,
                last_test_status: None,
                strip_target_protocol: row.get(5)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 更新代理信息（名称、URL、类型）
    /// @param proxy_id - 代理 ID
    /// @param proxy_name - 新名称
    /// @param url - 新 URL
    /// @param proxy_type - 新类型
    pub fn update_proxy(&self, proxy_id: i64, proxy_name: &str, url: &str, proxy_type: &str) -> AppResult<()> {
        self.conn.execute(
            "UPDATE proxies_info SET proxy_name=?1, url=?2, proxy_type=?3 WHERE proxy_id=?4",
            rusqlite::params![proxy_name, url, proxy_type, proxy_id],
        )?;
        Ok(())
    }

    /// 更新代理名称（支持手动编辑覆盖）
    /// @param proxy_id - 代理 ID
    /// @param proxy_name - 新名称
    pub fn update_proxy_name(&self, proxy_id: i64, proxy_name: &str) -> AppResult<()> {
        self.conn.execute(
            "UPDATE proxies_info SET proxy_name=?1 WHERE proxy_id=?2",
            rusqlite::params![proxy_name, proxy_id],
        )?;
        Ok(())
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

    /// 清空所有代理记录并重置自增 ID
    /// 同时级联清空 proxies_test 表（通过外键 ON DELETE CASCADE）
    /// @returns 删除的记录数
    pub fn clear_all_proxies(&self) -> AppResult<usize> {
        // 先删除测试记录（避免外键约束问题，虽然设置了 CASCADE）
        self.conn.execute("DELETE FROM proxies_test", [])?;
        // 删除代理记录（会级联删除关联测试记录）
        let deleted = self.conn.execute("DELETE FROM proxies_info", [])?;
        // 重置自增 ID
        self.conn.execute("DELETE FROM sqlite_sequence WHERE name = 'proxies_info'", [])?;
        Ok(deleted)
    }
}
