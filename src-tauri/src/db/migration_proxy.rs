/**
 * migration_proxy.rs - proxies_info 和 proxies_test 表结构迁移
 *
 * 迁移内容：
 * - 确保 proxies_info 表包含所有必需字段
 * - 确保 proxies_test 表包含所有必需字段
 * - 添加必要的索引
 *
 * 迁移方式：检测列缺失后整表重建（SQLite 不支持 DROP CONSTRAINT）
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    /// 迁移 proxies_info 和 proxies_test 表
    /// @returns 迁移结果（表不存在或无需迁移时直接返回 Ok）
    pub(crate) fn migrate_proxies(&self) -> AppResult<()> {
        self.migrate_proxies_info()?;
        self.migrate_proxies_test()?;
        Ok(())
    }

    /// 迁移 proxies_info 表
    fn migrate_proxies_info(&self) -> AppResult<()> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='proxies_info'",
            [],
            |row| row.get(0),
        )?;
        if !exists {
            return Ok(());
        }

        let columns = self.get_table_columns("proxies_info")?;
        let has_proxy_name = columns.contains(&"proxy_name".to_string());
        let has_proxy_type = columns.contains(&"proxy_type".to_string());
        let has_url = columns.contains(&"url".to_string());
        let has_is_active = columns.contains(&"is_active".to_string());
        let has_strip = columns.contains(&"strip_target_protocol".to_string());

        // 如果所有字段都已存在，跳过迁移
        if has_proxy_name && has_proxy_type && has_url && has_is_active && has_strip {
            return Ok(());
        }

        log::info!(
            "[migrate_proxies_info] 重建 proxies_info 表（字段齐全={}）",
            has_proxy_name && has_proxy_type && has_url && has_is_active && has_strip
        );

        let new_schema = "CREATE TABLE proxies_info_new (
            proxy_id    INTEGER PRIMARY KEY AUTOINCREMENT,
            proxy_name  TEXT NOT NULL,
            proxy_type  TEXT NOT NULL DEFAULT 'download',
            url         TEXT NOT NULL UNIQUE,
            is_active   INTEGER NOT NULL DEFAULT 1,
            strip_target_protocol INTEGER NOT NULL DEFAULT 0
        );";

        let proxy_name_expr = if has_proxy_name { "proxy_name" } else { "''" };
        let proxy_type_expr = if has_proxy_type {
            "proxy_type"
        } else {
            "'download'"
        };
        let url_expr = if has_url { "url" } else { "''" };
        let is_active_expr = if has_is_active { "is_active" } else { "1" };
        let strip_expr = if has_strip {
            "strip_target_protocol"
        } else {
            "0"
        };

        let insert_sql = format!(
            "INSERT INTO proxies_info_new (proxy_id, proxy_name, proxy_type, url, is_active, strip_target_protocol)
             SELECT proxy_id, {proxy_name}, {proxy_type}, {url}, {is_active}, {strip}
             FROM proxies_info;",
            proxy_name = proxy_name_expr,
            proxy_type = proxy_type_expr,
            url = url_expr,
            is_active = is_active_expr,
            strip = strip_expr
        );

        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
        self.conn
            .execute_batch("DROP TABLE IF EXISTS proxies_info_new;")?;
        self.conn.execute_batch(new_schema)?;
        self.conn.execute_batch(&insert_sql)?;
        self.conn.execute_batch("DROP TABLE proxies_info;")?;
        self.conn
            .execute_batch("ALTER TABLE proxies_info_new RENAME TO proxies_info;")?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        log::info!("[migrate_proxies_info] proxies_info 表已重建");
        Ok(())
    }

    /// 迁移 proxies_test 表
    fn migrate_proxies_test(&self) -> AppResult<()> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='proxies_test'",
            [],
            |row| row.get(0),
        )?;
        if !exists {
            return Ok(());
        }

        let columns = self.get_table_columns("proxies_test")?;
        let has_proxy_id = columns.contains(&"proxy_id".to_string());
        let has_test_time = columns.contains(&"test_time".to_string());
        let has_avg_latency = columns.contains(&"avg_latency".to_string());
        let has_success_count = columns.contains(&"success_count".to_string());
        let has_fail_count = columns.contains(&"fail_count".to_string());
        let has_last_test_status = columns.contains(&"last_test_status".to_string());

        // 如果所有字段都已存在，跳过迁移
        if has_proxy_id
            && has_test_time
            && has_avg_latency
            && has_success_count
            && has_fail_count
            && has_last_test_status
        {
            return Ok(());
        }

        log::info!(
            "[migrate_proxies_test] 重建 proxies_test 表（字段齐全={}）",
            has_proxy_id
                && has_test_time
                && has_avg_latency
                && has_success_count
                && has_fail_count
                && has_last_test_status
        );

        let new_schema = "CREATE TABLE proxies_test_new (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            proxy_id      INTEGER NOT NULL,
            test_time     TEXT,
            avg_latency   INTEGER,
            success_count INTEGER NOT NULL DEFAULT 0,
            fail_count    INTEGER NOT NULL DEFAULT 0,
            last_test_status TEXT,
            FOREIGN KEY (proxy_id) REFERENCES proxies_info(proxy_id) ON DELETE CASCADE
        );";

        let proxy_id_expr = if has_proxy_id { "proxy_id" } else { "0" };
        let test_time_expr = if has_test_time {
            "test_time"
        } else {
            "datetime('now')"
        };
        let avg_latency_expr = if has_avg_latency {
            "avg_latency"
        } else {
            "NULL"
        };
        let success_count_expr = if has_success_count {
            "success_count"
        } else {
            "0"
        };
        let fail_count_expr = if has_fail_count { "fail_count" } else { "0" };
        // 已有数据回填：有延迟即成功，无延迟即失败；新表该列默认 NULL（未测试）
        let last_test_status_expr = if has_last_test_status {
            "last_test_status"
        } else {
            "CASE WHEN avg_latency IS NOT NULL THEN 'success' ELSE 'fail' END"
        };

        let insert_sql = format!(
            "INSERT INTO proxies_test_new (id, proxy_id, test_time, avg_latency, success_count, fail_count, last_test_status)
             SELECT id, {proxy_id}, {test_time}, {avg_latency}, {success_count}, {fail_count}, {last_test_status}
             FROM proxies_test;",
            proxy_id = proxy_id_expr,
            test_time = test_time_expr,
            avg_latency = avg_latency_expr,
            success_count = success_count_expr,
            fail_count = fail_count_expr,
            last_test_status = last_test_status_expr
        );

        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
        self.conn
            .execute_batch("DROP TABLE IF EXISTS proxies_test_new;")?;
        self.conn.execute_batch(new_schema)?;
        self.conn.execute_batch(&insert_sql)?;
        self.conn.execute_batch("DROP TABLE proxies_test;")?;
        self.conn
            .execute_batch("ALTER TABLE proxies_test_new RENAME TO proxies_test;")?;
        // 重建索引
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_proxies_test_proxy ON proxies_test(proxy_id);",
        )?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        log::info!("[migrate_proxies_test] proxies_test 表已重建");
        Ok(())
    }
}
