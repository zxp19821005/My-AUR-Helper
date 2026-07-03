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
mod migration;
mod proxies_info;
mod proxies_test;
mod schema;
mod seed;
mod settings;
mod software_info;
mod upstream_info;

use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

/// 数据库结构体，包装 rusqlite 连接
pub struct Database {
    conn: Connection,
}

impl Database {
    /// 打开或创建数据库文件
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn })
    }

    /// 初始化数据库：建表 → 迁移 → 填充默认数据
    pub fn initialize(&self) -> Result<()> {
        self.create_tables()?;
        self.migrate_aur_info()?;
        self.seed_defaults()?;
        Ok(())
    }
}
