use crate::errors::AppResult;

use super::Database;

impl Database {
    pub fn migrate_aur_info(&self) -> AppResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(aur_info)")?;
        let columns: Vec<String> = stmt.query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();

        let old_cols = ["provides", "conflicts", "replaces", "votes", "popularity", "submitted_by", "maintainers"];
        for col in &old_cols {
            if columns.contains(&col.to_string()) {
                self.conn.execute_batch(&format!("ALTER TABLE aur_info DROP COLUMN {col};"))?;
            }
        }

        if columns.contains(&"last_updated".to_string()) {
            self.conn.execute_batch(
                "UPDATE aur_info SET last_updated = CAST(last_updated AS INTEGER)
                 WHERE typeof(last_updated) = 'text' AND last_updated IS NOT NULL
                 AND last_updated NOT LIKE '%-%';"
            )?;
            self.conn.execute_batch(
                "UPDATE aur_info SET last_updated = CAST(strftime('%s', last_updated) AS INTEGER)
                 WHERE typeof(last_updated) = 'text' AND last_updated IS NOT NULL
                 AND last_updated LIKE '%-%';"
            )?;
        }
        Ok(())
    }

    pub fn migrate_software_info(&self) -> AppResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(software_info)")?;
        let columns: Vec<String> = stmt.query_map([], |row| row.get(1))?
            .filter_map(|r| r.ok())
            .collect();

        if columns.contains(&"package_type".to_string()) && !columns.contains(&"package_type_id".to_string()) {
            self.conn.execute_batch("ALTER TABLE software_info RENAME COLUMN package_type TO package_type_id;")?;
        }
        if columns.contains(&"checker_type".to_string()) && !columns.contains(&"checker_type_id".to_string()) {
            self.conn.execute_batch("ALTER TABLE software_info RENAME COLUMN checker_type TO checker_type_id;")?;
        }
        if columns.contains(&"created_at".to_string()) {
            self.conn.execute_batch("ALTER TABLE software_info DROP COLUMN created_at;")?;
        }
        if !columns.contains(&"version_extract_regex".to_string()) {
            self.conn.execute_batch("ALTER TABLE software_info ADD COLUMN version_extract_regex TEXT;")?;
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
                self.conn.execute_batch("ALTER TABLE software_info DROP COLUMN license_id;")?;
            }
        }
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
                language_id             INTEGER,
                version_extract_regex   TEXT,
                FOREIGN KEY (language_id) REFERENCES enum_programming_languages(id)
            );"
        )?;
        self.conn.execute_batch(
            "INSERT INTO software_info_new
             SELECT software_id, pkgname, upstream_url, package_type_id, checker_type_id,
                    is_outdated, check_test_versions, check_binary_files, auto_check_enabled,
                    language_id, version_extract_regex
             FROM software_info;"
        )?;
        self.conn.execute_batch("DROP TABLE software_info;")?;
        self.conn.execute_batch("ALTER TABLE software_info_new RENAME TO software_info;")?;
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_software_pkgname ON software_info(pkgname);
             CREATE INDEX IF NOT EXISTS idx_software_outdated ON software_info(is_outdated);"
        )?;
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        Ok(())
    }

    pub fn migrate_upstream_info(&self) -> AppResult<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(upstream_info)")?;
        let columns: Vec<String> = stmt.query_map([], |row| row.get(1))?
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
            self.conn.execute_batch("ALTER TABLE upstream_info_new RENAME TO upstream_info;")?;
            self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        }

        Ok(())
    }
}
