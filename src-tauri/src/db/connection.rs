/**
 * connection.rs - 数据库连接和初始化
 *
 * 提供 Database 结构体定义、连接创建、表初始化和迁移功能
 */
use std::path::Path;
use crate::errors::AppResult;
use rusqlite::Connection;

/// 数据库结构体，包装 rusqlite 连接
pub struct Database {
    pub(crate) conn: Connection,
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
        self.migrate_backup_software()?;
        self.migrate_cache_software()?;
        self.seed_defaults()?;
        self.ensure_no_fk_constraints()?;
        Ok(())
    }

    /// 迁移 backup_software 表：移除 software_id、添加 pkgver/full_path 字段
    fn migrate_backup_software(&self) -> AppResult<()> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='backup_software'",
            [],
            |row| row.get(0),
        )?;
        if !exists {
            return Ok(());
        }

        let columns = self.get_table_columns("backup_software")?;
        let has_software_id = columns.contains(&"software_id".to_string());
        let has_pkgver = columns.contains(&"pkgver".to_string());
        let has_full_path = columns.contains(&"full_path".to_string());

        if !has_software_id && has_pkgver && has_full_path {
            return Ok(());
        }

        log::info!("[migrate_backup_software] 重建 backup_software 表");

        let new_schema = "CREATE TABLE backup_software_new (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            filename     TEXT NOT NULL,
            epoch        INTEGER NOT NULL DEFAULT 0,
            pkgver       TEXT NOT NULL DEFAULT '',
            pkgrel       TEXT NOT NULL DEFAULT '1',
            arch         TEXT NOT NULL DEFAULT 'x86_64',
            subdirectory TEXT,
            full_path    TEXT NOT NULL DEFAULT ''
        );";

        let has_subdir = columns.contains(&"subdirectory".to_string());
        let subdir_expr = if has_subdir { "subdirectory" } else { "''" };
        let full_path_expr = format!(
            "CASE WHEN {subdir} IS NOT NULL AND {subdir} != '' THEN {subdir} || '/' || filename ELSE filename END",
            subdir = subdir_expr
        );

        let select_cols = if has_pkgver {
            format!(
                "id, filename, epoch, pkgver, pkgrel, arch, {subdir}, {fp}",
                subdir = subdir_expr,
                fp = full_path_expr
            )
        } else {
            format!(
                "id, filename, epoch, '', pkgrel, arch, {subdir}, {fp}",
                subdir = subdir_expr,
                fp = full_path_expr
            )
        };

        let insert_sql = format!(
            "INSERT INTO backup_software_new (id, filename, epoch, pkgver, pkgrel, arch, subdirectory, full_path)
             SELECT {cols} FROM backup_software;",
            cols = select_cols
        );

        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
        self.conn
            .execute_batch("DROP TABLE IF EXISTS backup_software_new;")?;
        self.conn.execute_batch(new_schema)?;
        self.conn.execute_batch(&insert_sql)?;
        self.conn.execute_batch("DROP TABLE backup_software;")?;
        self.conn
            .execute_batch("ALTER TABLE backup_software_new RENAME TO backup_software;")?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        Ok(())
    }

    /// 迁移 cache_software 表：移除 software_id 外键，添加 name/version/size/source_dir 字段
    fn migrate_cache_software(&self) -> AppResult<()> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='cache_software'",
            [],
            |row| row.get(0),
        )?;
        if !exists {
            return Ok(());
        }

        let columns = self.get_table_columns("cache_software")?;
        let has_name = columns.contains(&"name".to_string());
        let has_version = columns.contains(&"version".to_string());
        let has_size = columns.contains(&"size".to_string());
        let has_source_dir = columns.contains(&"source_dir".to_string());

        // 如果所有新字段都已存在，跳过迁移
        if has_name && has_version && has_size && has_source_dir {
            return Ok(());
        }

        log::info!("[migrate_cache_software] 重建 cache_software 表");

        let new_schema = "CREATE TABLE cache_software_new (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            software_id     INTEGER NOT NULL DEFAULT 0,
            filename        TEXT NOT NULL,
            name            TEXT NOT NULL DEFAULT '',
            epoch           INTEGER NOT NULL DEFAULT 0,
            version         TEXT NOT NULL DEFAULT '',
            pkgrel          TEXT NOT NULL DEFAULT '1',
            arch            TEXT NOT NULL DEFAULT 'x86_64',
            size            INTEGER NOT NULL DEFAULT 0,
            source_dir      TEXT,
            cache_directory TEXT NOT NULL DEFAULT ''
        );";

        let name_expr = if has_name { "name" } else { "''" };
        let version_expr = if has_version { "version" } else { "''" };
        let size_expr = if has_size { "size" } else { "0" };
        let source_dir_expr = if has_source_dir { "source_dir" } else { "NULL" };

        let insert_sql = format!(
            "INSERT INTO cache_software_new (id, software_id, filename, name, epoch, version, pkgrel, arch, size, source_dir, cache_directory)
             SELECT id, software_id, filename, {name}, epoch, {version}, pkgrel, arch, {size}, {source_dir}, cache_directory
             FROM cache_software;",
            name = name_expr,
            version = version_expr,
            size = size_expr,
            source_dir = source_dir_expr
        );

        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
        self.conn
            .execute_batch("DROP TABLE IF EXISTS cache_software_new;")?;
        self.conn.execute_batch(new_schema)?;
        self.conn.execute_batch(&insert_sql)?;
        self.conn.execute_batch("DROP TABLE cache_software;")?;
        self.conn
            .execute_batch("ALTER TABLE cache_software_new RENAME TO cache_software;")?;
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_cache_software_pkg ON cache_software(software_id);
             CREATE INDEX IF NOT EXISTS idx_cache_software_name ON cache_software(name);",
        )?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        log::info!("[migrate_cache_software] cache_software 表已重建");
        Ok(())
    }

    /// 获取指定表的所有列名（通过白名单验证防止 SQL 注入）
    pub(crate) fn get_table_columns(&self, table_name: &str) -> AppResult<Vec<String>> {
        const ALLOWED_TABLES: &[&str] = &[
            "software_info", "aur_info", "upstream_info", "proxies_info",
            "backup_software", "cache_software", "logs", "settings",
            "enum_licenses", "enum_programming_languages", "proxies_test",
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

    /// 检查并确保 software_info 表没有意外的 FK 约束（仅首次执行）
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
