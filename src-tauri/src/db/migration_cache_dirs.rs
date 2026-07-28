/**
 * migration_cache_dirs.rs - cache_dirs 表迁移
 *
 * 创建缓存目录配置表，用于存储 AUR 助手的缓存路径
 */
use crate::errors::AppResult;
use super::Database;

impl Database {
    /// 迁移 cache_dirs 表：如果不存在则创建
    pub fn migrate_cache_dirs(&self) -> AppResult<()> {
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='cache_dirs'",
            [],
            |row| row.get(0),
        )?;

        if !exists {
            self.conn.execute_batch(
                "CREATE TABLE cache_dirs (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    name        TEXT NOT NULL,
                    path        TEXT NOT NULL,
                    is_enabled  INTEGER NOT NULL DEFAULT 1,
                    sort_order  INTEGER NOT NULL DEFAULT 0,
                    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
                );

                -- 插入默认的缓存目录配置
                INSERT INTO cache_dirs (name, path, is_enabled, sort_order) VALUES
                    ('系统缓存', '/var/cache/pacman/pkg', 1, 0),
                    ('paru 缓存', '/home/' || trim(replace(hex(randomblob(8)), '00', '')) || '/.cache/paru/clone', 0, 1),
                    ('yay 缓存', '/home/' || trim(replace(hex(randomblob(8)), '00', '')) || '/.config/yay', 0, 2);
                "
            )?;

            // 修正默认路径为实际用户名
            if let Ok(username) = std::env::var("USER") {
                if username.is_empty() {
                    return Ok(());
                }
                let paru_path = format!("/home/{}/.cache/paru/clone", username);
                let yay_path = format!("/home/{}/.config/yay", username);
                self.conn.execute(
                    "UPDATE cache_dirs SET path=?1 WHERE name='paru 缓存'",
                    rusqlite::params![paru_path],
                )?;
                self.conn.execute(
                    "UPDATE cache_dirs SET path=?1 WHERE name='yay 缓存'",
                    rusqlite::params![yay_path],
                )?;
            }
        }

        Ok(())
    }
}
