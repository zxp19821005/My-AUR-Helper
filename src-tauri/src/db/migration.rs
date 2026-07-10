use crate::errors::AppResult;

use super::Database;

impl Database {
    pub fn migrate_aur_info(&self) -> AppResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(aur_info)")?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();

        let old_cols = [
            "provides",
            "conflicts",
            "replaces",
            "votes",
            "popularity",
            "submitted_by",
            "maintainers",
        ];
        for col in &old_cols {
            if columns.contains(&col.to_string()) {
                self.conn
                    .execute_batch(&format!("ALTER TABLE aur_info DROP COLUMN {col};"))?;
            }
        }

        if columns.contains(&"last_updated".to_string()) {
            self.conn.execute_batch(
                "UPDATE aur_info SET last_updated = CAST(last_updated AS INTEGER)
                 WHERE typeof(last_updated) = 'text' AND last_updated IS NOT NULL
                 AND last_updated NOT LIKE '%-%';",
            )?;
            self.conn.execute_batch(
                "UPDATE aur_info SET last_updated = CAST(strftime('%s', last_updated) AS INTEGER)
                 WHERE typeof(last_updated) = 'text' AND last_updated IS NOT NULL
                 AND last_updated LIKE '%-%';",
            )?;
        }
        Ok(())
    }

    pub fn migrate_software_info(&self) -> AppResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(software_info)")?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();

        if columns.contains(&"package_type".to_string())
            && !columns.contains(&"package_type_id".to_string())
        {
            self.conn.execute_batch(
                "ALTER TABLE software_info RENAME COLUMN package_type TO package_type_id;",
            )?;
        }
        if columns.contains(&"checker_type".to_string())
            && !columns.contains(&"checker_type_id".to_string())
        {
            self.conn.execute_batch(
                "ALTER TABLE software_info RENAME COLUMN checker_type TO checker_type_id;",
            )?;
        }
        if columns.contains(&"created_at".to_string()) {
            self.conn
                .execute_batch("ALTER TABLE software_info DROP COLUMN created_at;")?;
        }
        if !columns.contains(&"version_extract_regex".to_string()) {
            self.conn.execute_batch(
                "ALTER TABLE software_info ADD COLUMN version_extract_regex TEXT;",
            )?;
        }

        if columns.contains(&"license_id".to_string()) {
            let fk_has_license: bool = self.conn.query_row(
                "SELECT COUNT(*) > 0 FROM pragma_foreign_key_list('software_info') WHERE \"from\" = 'license_id'",
                [],
                |row| row.get(0),
            )?;
            if fk_has_license {
                self.rebuild_software_info_without_license_fk()?;
            } else {
                self.conn
                    .execute_batch("ALTER TABLE software_info DROP COLUMN license_id;")?;
            }
        }

        // 将 language_id 从 INTEGER 改为 TEXT 以存储 JSON 数组
        self.migrate_language_id_to_json()?;

        Ok(())
    }

    fn migrate_language_id_to_json(&self) -> AppResult<()> {
        // 检查 language_id 列的类型
        let col_type: Option<String> = self.conn.query_row(
            "SELECT type FROM pragma_table_info('software_info') WHERE name='language_id'",
            [],
            |row| row.get(0),
        ).ok();

        // 如果列不存在或已经是 TEXT 类型，则不需要迁移
        if col_type.is_none() || col_type.as_deref() == Some("TEXT") {
            return Ok(());
        }

        // 需要重建表以改变列类型，移除外键约束
        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
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

        // 将旧的 INTEGER language_id 转换为 JSON 数组格式
        self.conn.execute_batch(
            "INSERT INTO software_info_new
             SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id,
                    is_outdated, check_test_versions, check_binary_files, auto_check_enabled,
                    CASE WHEN language_id IS NULL THEN '[]'
                         ELSE '[' || CAST(language_id AS TEXT) || ']'
                    END,
                    version_extract_regex
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

        Ok(())
    }

    fn rebuild_software_info_without_license_fk(&self) -> AppResult<()> {
        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
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
                    CASE WHEN language_id IS NULL THEN '[]'
                         ELSE '[' || CAST(language_id AS TEXT) || ']'
                    END,
                    version_extract_regex
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
        Ok(())
    }

    /// 迁移 enum_programming_languages 表，简化为 id, name, short_name
    pub fn migrate_enum_programming_languages(&self) -> AppResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(enum_programming_languages)")?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();

        // 检查是否需要迁移（存在旧列）
        let needs_migration = columns.contains(&"description".to_string())
            || columns.contains(&"file_extensions".to_string())
            || columns.contains(&"build_system".to_string())
            || columns.contains(&"build_command".to_string());

        if !needs_migration {
            // 如果表结构已经是新的，确保 short_name 列存在
            if !columns.contains(&"short_name".to_string()) {
                self.conn.execute_batch(
                    "ALTER TABLE enum_programming_languages ADD COLUMN short_name TEXT;",
                )?;
            }
            return Ok(());
        }

        // 重建表
        self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
        self.conn.execute_batch(
            "CREATE TABLE enum_programming_languages_new (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,
                short_name  TEXT
            );",
        )?;

        // 迁移数据（如果 short_name 不存在，用 name 的首字母作为简称）
        let has_short_name = columns.contains(&"short_name".to_string());
        if has_short_name {
            self.conn.execute_batch(
                "INSERT INTO enum_programming_languages_new (id, name, short_name)
                 SELECT id, name, short_name FROM enum_programming_languages;",
            )?;
        } else {
            self.conn.execute_batch(
                "INSERT INTO enum_programming_languages_new (id, name, short_name)
                 SELECT id, name, SUBSTR(name, 1, 2) FROM enum_programming_languages;",
            )?;
        }

        self.conn.execute_batch("DROP TABLE enum_programming_languages;")?;
        self.conn.execute_batch(
            "ALTER TABLE enum_programming_languages_new RENAME TO enum_programming_languages;",
        )?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        Ok(())
    }

    pub fn migrate_upstream_info(&self) -> AppResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(upstream_info)")?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();

        let has_upstream_url = columns.contains(&"upstream_url".to_string());
        let has_upstream_license = columns.contains(&"upstream_license".to_string());
        let has_upstream_license_id = columns.contains(&"upstream_license_id".to_string());
        let has_last_checked = columns.contains(&"last_checked".to_string());

        if has_upstream_url || has_upstream_license || !has_upstream_license_id {
            let last_checked_type: Option<String> = self.conn.query_row(
                "SELECT COALESCE((SELECT typeof(\"last_checked\") FROM upstream_info LIMIT 1), 'null')",
                [],
                |row| row.get(0),
            ).unwrap_or_default();

            self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;

            let new_schema = "upstream_info_new";
            let drop_sql = format!("DROP TABLE IF EXISTS {new_schema};");
            self.conn.execute_batch(&drop_sql)?;

            self.conn.execute_batch(
                "CREATE TABLE upstream_info_new (
                    software_id        INTEGER PRIMARY KEY,
                    upstream_version   TEXT,
                    upstream_license_id INTEGER,
                    last_checked       INTEGER,
                    FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE,
                    FOREIGN KEY (upstream_license_id) REFERENCES enum_licenses(id)
                );"
            )?;

            if has_upstream_url {
                if last_checked_type.as_deref() == Some("text") {
                    self.conn.execute_batch(
                        "INSERT INTO upstream_info_new (software_id, upstream_version, upstream_license_id, last_checked)
                         SELECT software_id, upstream_version, NULL,
                                CAST(strftime('%s', last_checked) AS INTEGER)
                         FROM upstream_info;"
                    )?;
                } else {
                    self.conn.execute_batch(
                        "INSERT INTO upstream_info_new (software_id, upstream_version, upstream_license_id, last_checked)
                         SELECT software_id, upstream_version, NULL, last_checked
                         FROM upstream_info;"
                    )?;
                }
            } else if has_upstream_license || !has_upstream_license_id {
                if last_checked_type.as_deref() == Some("text") {
                    self.conn.execute_batch(
                        "INSERT INTO upstream_info_new (software_id, upstream_version, upstream_license_id, last_checked)
                         SELECT software_id, upstream_version,
                                CASE WHEN typeof(upstream_license) = 'integer' THEN CAST(upstream_license AS INTEGER) ELSE NULL END,
                                CAST(strftime('%s', last_checked) AS INTEGER)
                         FROM upstream_info;"
                    )?;
                } else {
                    self.conn.execute_batch(
                        "INSERT INTO upstream_info_new (software_id, upstream_version, upstream_license_id, last_checked)
                         SELECT software_id, upstream_version,
                                CASE WHEN typeof(upstream_license) = 'integer' THEN CAST(upstream_license AS INTEGER) ELSE NULL END,
                                last_checked
                         FROM upstream_info;"
                    )?;
                }
            } else if has_last_checked && last_checked_type.as_deref() == Some("text") {
                self.conn.execute_batch(
                    "INSERT INTO upstream_info_new (software_id, upstream_version, upstream_license_id, last_checked)
                     SELECT software_id, upstream_version, upstream_license_id,
                            CAST(strftime('%s', last_checked) AS INTEGER)
                     FROM upstream_info;"
                )?;
            } else {
                self.conn.execute_batch(
                    "INSERT INTO upstream_info_new (software_id, upstream_version, upstream_license_id, last_checked)
                     SELECT software_id, upstream_version, upstream_license_id, last_checked
                     FROM upstream_info;"
                )?;
            }

            self.conn.execute_batch("DROP TABLE upstream_info;")?;
            self.conn
                .execute_batch("ALTER TABLE upstream_info_new RENAME TO upstream_info;")?;
            self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        }

        Ok(())
    }

    pub fn migrate_enum_licenses(&self) -> AppResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(enum_licenses)")?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();

        if columns.len() > 3 {
            self.conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
            self.conn.execute_batch(
                "CREATE TABLE enum_licenses_new (
                    id        INTEGER PRIMARY KEY AUTOINCREMENT,
                    spdx_id   TEXT NOT NULL UNIQUE,
                    full_name TEXT NOT NULL
                );",
            )?;
            self.conn.execute_batch(
                "INSERT INTO enum_licenses_new (id, spdx_id, full_name)
                 SELECT id, spdx_id, full_name FROM enum_licenses;",
            )?;
            self.conn.execute_batch("DROP TABLE enum_licenses;")?;
            self.conn
                .execute_batch("ALTER TABLE enum_licenses_new RENAME TO enum_licenses;")?;
            self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        }
        Ok(())
    }
}