/**
 * db/mod.rs - 数据库操作模块入口
 *
 * 模块结构：
 * - schema.rs    — 表结构定义（CREATE TABLE）
 * - migration.rs — 数据迁移逻辑（ALTER TABLE）
 * - seed.rs      — 默认数据初始化
 * - aur_info.rs  — AUR 包信息表 CRUD
 * - backup_software.rs — 备份记录表 CRUD
 * - cache_software.rs  — 缓存记录表 CRUD
 * - enum_licenses.rs   — License 枚举表 CRUD
 * - enum_programming_languages.rs — 编程语言枚举表 CRUD
 * - logs.rs      — 日志表 CRUD
 * - proxies_info.rs   — 代理信息表 CRUD
 * - proxies_test.rs   — 代理测试表 CRUD
 * - settings.rs  — 设置表 CRUD
 * - software_info.rs  — 软件包信息表 CRUD
 * - upstream_info.rs  — 上游版本信息表 CRUD
 */
mod aur_info;
mod backup_software;
mod cache_software;
mod enum_licenses;
mod enum_programming_languages;
mod logs;
mod migration_aur;
mod migration_software;
mod migration_upstream;
mod migration_enum;
mod proxies_info;
mod proxies_test;
mod schema;
mod seed;
mod settings;
mod software_info;
mod upstream_info;

use std::path::Path;

use crate::errors::AppResult;
use rusqlite::Connection;

/// 数据库结构体，包装 rusqlite 连接
pub struct Database {
    conn: Connection,
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
        self.seed_defaults()?;
        Ok(())
    }

    /// 获取指定表的所有列名
    /// 通过白名单验证表名，防止 SQL 注入
    fn get_table_columns(&self, table_name: &str) -> AppResult<Vec<String>> {
        // 白名单：仅允许已知的表名
        const ALLOWED_TABLES: &[&str] = &[
            "software_info", "aur_info", "upstream_info", "proxies_info",
            "backup_software", "cache_software", "logs", "settings",
            "enum_licenses", "enum_programming_languages", "proxies_test",
        ];
        if !ALLOWED_TABLES.contains(&table_name) {
            return Err(crate::errors::AppError::DatabaseError(
                format!("不允许查询表 '{}' 的列信息", table_name)
            ));
        }
        let mut stmt = self.conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(columns)
    }
}