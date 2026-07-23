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
mod migration_enum;
mod migration_software;
mod migration_upstream;
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
    /// 标记 software_info 表的 FK 约束是否已检查并修复
    fk_checked: std::cell::Cell<bool>,
}

impl Database {
    /// 打开或创建数据库文件
    pub fn new(path: &Path) -> AppResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self {
            conn,
            fk_checked: std::cell::Cell::new(false),
        })
    }

    /// 初始化数据库：建表 → 迁移 → 填充默认数据 → 修复 FK 约束
    pub fn initialize(&self) -> AppResult<()> {
        self.create_tables()?;
        self.migrate_aur_info()?;
        self.migrate_software_info()?;
        self.migrate_upstream_info()?;
        self.migrate_enum_licenses()?;
        self.migrate_enum_programming_languages()?;
        self.seed_defaults()?;
        // 初始化时一次性检查并修复 FK 约束
        self.ensure_no_fk_constraints()?;
        Ok(())
    }

    /// 获取指定表的所有列名
    /// 通过白名单验证表名，防止 SQL 注入
    fn get_table_columns(&self, table_name: &str) -> AppResult<Vec<String>> {
        // 白名单：仅允许已知的表名
        const ALLOWED_TABLES: &[&str] = &[
            "software_info",
            "aur_info",
            "upstream_info",
            "proxies_info",
            "backup_software",
            "cache_software",
            "logs",
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

    /// 检查并确保 software_info 表没有意外的 FK 约束
    /// 仅在首次调用时执行检查，后续调用直接返回
    fn ensure_no_fk_constraints(&self) -> AppResult<()> {
        if self.fk_checked.get() {
            return Ok(());
        }
        let fk_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_foreign_key_list('software_info')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if fk_count > 0 {
            log::warn!(
                "[ensure_no_fk_constraints] software_info 表有 {} 个外键约束，正在移除...",
                fk_count
            );
            self.rebuild_software_info_remove_fk()?;
        }
        self.fk_checked.set(true);
        Ok(())
    }

    /// 重建 software_info 表以移除所有外键约束
    fn rebuild_software_info_remove_fk(&self) -> AppResult<()> {
        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
        self.conn
            .execute_batch("DROP TABLE IF EXISTS software_info_new;")?;
        self.conn.execute_batch(
            "CREATE TABLE software_info_new (
                software_id             INTEGER PRIMARY KEY AUTOINCREMENT,
                pkgname                 TEXT NOT NULL UNIQUE,
                upstream_url            TEXT,
                package_type_id         INTEGER NOT NULL DEFAULT 1,
                checker_type_id         INTEGER NOT NULL DEFAULT 7,
                is_outdated             INTEGER NOT NULL DEFAULT 0,
                check_test_versions     INTEGER NOT NULL DEFAULT 0,
                check_binary_files      INTEGER NOT NULL DEFAULT 0,
                auto_check_enabled      INTEGER NOT NULL DEFAULT 1,
                language_id             TEXT DEFAULT '[]',
                version_extract_regex   TEXT
            );",
        )?;
        self.conn.execute_batch(
            "INSERT INTO software_info_new
             SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id,
                    is_outdated, check_test_versions, check_binary_files, auto_check_enabled,
                    language_id, version_extract_regex
             FROM software_info;",
        )?;
        self.conn.execute_batch("DROP TABLE software_info;")?;
        self.conn
            .execute_batch("ALTER TABLE software_info_new RENAME TO software_info;")?;
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_software_pkgname ON software_info(pkgname);
             CREATE INDEX IF NOT EXISTS idx_software_outdated ON software_info(is_outdated);",
        )?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        log::info!("[rebuild_software_info_remove_fk] software_info 表已重建，FK 约束已移除");
        Ok(())
    }
}
