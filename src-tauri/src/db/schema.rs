/**
 * schema.rs - 数据库表结构定义
 *
 * 集中管理所有 CREATE TABLE 语句
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    /// 创建所有数据库表（如不存在）
    pub fn create_tables(&self) -> AppResult<()> {
        self.conn.execute_batch(
            "
            -- 软件包信息表
            CREATE TABLE IF NOT EXISTS software_info (
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
            );

            -- AUR 包详情信息表
            CREATE TABLE IF NOT EXISTS aur_info (
                software_id     INTEGER PRIMARY KEY,
                pkgdesc         TEXT,
                aur_version     TEXT,
                license_id      TEXT,
                last_updated    INTEGER,
                depends         TEXT,
                makedepends     TEXT,
                optdepends      TEXT,
                out_of_date     INTEGER,
                FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE
            );

            -- 上游版本信息表
            CREATE TABLE IF NOT EXISTS upstream_info (
                software_id        INTEGER PRIMARY KEY,
                upstream_version   TEXT,
                upstream_license_id TEXT,
                last_checked       INTEGER,
                FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE
            );

            -- 备份软件包记录表
            CREATE TABLE IF NOT EXISTS backup_software (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                software_id  INTEGER NOT NULL,
                filename     TEXT NOT NULL,
                epoch        INTEGER NOT NULL DEFAULT 0,
                pkgrel       TEXT NOT NULL DEFAULT '1',
                arch         TEXT NOT NULL DEFAULT 'x86_64',
                subdirectory TEXT,
                FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE
            );

            -- 缓存软件包记录表
            CREATE TABLE IF NOT EXISTS cache_software (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                software_id    INTEGER NOT NULL,
                filename       TEXT NOT NULL,
                epoch          INTEGER NOT NULL DEFAULT 0,
                pkgrel         TEXT NOT NULL DEFAULT '1',
                arch           TEXT NOT NULL DEFAULT 'x86_64',
                cache_directory TEXT NOT NULL,
                FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE
            );

            -- 代理信息表
            CREATE TABLE IF NOT EXISTS proxies_info (
                proxy_id    INTEGER PRIMARY KEY AUTOINCREMENT,
                proxy_name  TEXT NOT NULL,
                proxy_type  TEXT NOT NULL DEFAULT 'download',
                url         TEXT NOT NULL UNIQUE,
                is_active   INTEGER NOT NULL DEFAULT 1
            );

            -- 代理测试结果表
            CREATE TABLE IF NOT EXISTS proxies_test (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                proxy_id      INTEGER NOT NULL,
                test_time     TEXT,
                avg_latency   INTEGER,
                success_count INTEGER NOT NULL DEFAULT 0,
                fail_count    INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (proxy_id) REFERENCES proxies_info(proxy_id) ON DELETE CASCADE
            );

            -- License 枚举表
            CREATE TABLE IF NOT EXISTS enum_licenses (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                spdx_id   TEXT NOT NULL UNIQUE,
                full_name TEXT NOT NULL
            );

            -- 编程语言枚举表
            CREATE TABLE IF NOT EXISTS enum_programming_languages (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,
                short_name  TEXT
            );

            -- 日志表
            CREATE TABLE IF NOT EXISTS logs (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                level      TEXT NOT NULL,
                message    TEXT NOT NULL,
                module     TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- 设置表
            CREATE TABLE IF NOT EXISTS settings (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                key         TEXT NOT NULL UNIQUE,
                value       TEXT NOT NULL DEFAULT '',
                description TEXT,
                category    TEXT NOT NULL DEFAULT 'general',
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- 索引
            CREATE INDEX IF NOT EXISTS idx_software_pkgname ON software_info(pkgname);
            CREATE INDEX IF NOT EXISTS idx_software_outdated ON software_info(is_outdated);
            CREATE INDEX IF NOT EXISTS idx_backup_software_pkg ON backup_software(software_id);
            CREATE INDEX IF NOT EXISTS idx_cache_software_pkg ON cache_software(software_id);
            CREATE INDEX IF NOT EXISTS idx_proxies_test_proxy ON proxies_test(proxy_id);
            CREATE INDEX IF NOT EXISTS idx_logs_created ON logs(created_at);
            CREATE INDEX IF NOT EXISTS idx_settings_category ON settings(category);
            ",
        )?;
        Ok(())
    }
}