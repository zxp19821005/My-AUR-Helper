use crate::errors::AppResult;
use rusqlite::Connection;
/**
 * connection.rs - 数据库连接和初始化
 *
 * 提供 Database 结构体定义、连接创建、表初始化和迁移功能
 */
use std::path::Path;

/// 数据库结构体，包装 rusqlite 连接
pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    /// 打开或创建数据库文件
    pub fn new(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn })
    }

    /// 初始化数据库：建表 → 迁移 → 填充默认数据
    pub fn initialize(&self) -> AppResult<()> {
        self.create_tables()?;
        self.migrate_aur_info()?;
        self.migrate_software_info()?;
        self.migrate_upstream_info()?;
        self.migrate_enum_licenses()?;
        self.migrate_enum_programming_languages()?;
        self.migrate_backup_software()?;
        self.migrate_cache_software()?;
        self.migrate_proxies()?;
        self.migrate_drop_logs_table()?;
        self.seed_defaults()?;
        Ok(())
    }

    /// 获取指定表的所有列名（通过白名单验证防止 SQL 注入）
    pub(crate) fn get_table_columns(&self, table_name: &str) -> AppResult<Vec<String>> {
        const ALLOWED_TABLES: &[&str] = &[
            "software_info",
            "aur_info",
            "upstream_info",
            "proxies_info",
            "backup_software",
            "cache_software",
            "settings",
            "enum_licenses",
            "enum_programming_languages",
            "proxies_test",
        ];
        if !ALLOWED_TABLES.contains(&table_name) {
            return Err(crate::errors::AppError::DatabaseError(format!(
                "不允许查询表 '{}' 的列信息",
                table_name
            )));
        }
        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA table_info({table_name})"))?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(columns)
    }
}
