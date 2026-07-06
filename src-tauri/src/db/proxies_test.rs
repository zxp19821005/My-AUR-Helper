use crate::errors::AppResult; // 通用错误处理

use crate::models::*; // 数据模型

use super::Database;  // 数据库结构体

impl Database {
    /// 插入代理测试结果记录
    /// @param test - 代理测试信息
    /// @returns 新插入记录的 ID
    pub fn insert_proxy_test(&self, test: &ProxyTest) -> AppResult<i64> {
        self.conn.execute(
            "INSERT INTO proxies_test (proxy_id, test_time, avg_latency, success_count, fail_count) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![test.proxy_id, test.test_time, test.avg_latency, test.success_count, test.fail_count],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 获取指定代理的所有测试记录（按时间降序）
    /// @param proxy_id - 代理 ID
    /// @returns 该代理的测试记录列表
    pub fn get_proxy_tests(&self, proxy_id: i64) -> AppResult<Vec<ProxyTest>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, proxy_id, test_time, avg_latency, success_count, fail_count FROM proxies_test WHERE proxy_id=?1 ORDER BY test_time DESC"
        )?;
        let rows = stmt.query_map(rusqlite::params![proxy_id], |row| {
            Ok(ProxyTest {
                id: Some(row.get(0)?),
                proxy_id: row.get(1)?,
                test_time: row.get(2)?,
                avg_latency: row.get(3)?,
                success_count: row.get(4)?,
                fail_count: row.get(5)?,
            })
        })?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }

    /// 获取指定代理的最新测试记录
    /// @param proxy_id - 代理 ID
    /// @returns 可选的最近一次测试记录
    pub fn get_latest_proxy_test(&self, proxy_id: i64) -> AppResult<Option<ProxyTest>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, proxy_id, test_time, avg_latency, success_count, fail_count FROM proxies_test WHERE proxy_id=?1 ORDER BY test_time DESC LIMIT 1"
        )?;
        let mut rows = stmt.query_map(rusqlite::params![proxy_id], |row| {
            Ok(ProxyTest {
                id: Some(row.get(0)?),
                proxy_id: row.get(1)?,
                test_time: row.get(2)?,
                avg_latency: row.get(3)?,
                success_count: row.get(4)?,
                fail_count: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }
}
