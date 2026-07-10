/**
 * seed.rs - 默认数据初始化
 *
 * 数据库首次初始化时插入默认设置和编程语言
 */
use crate::errors::AppResult;

use super::Database;

impl Database {
    /// 插入默认设置和编程语言数据（仅在表为空时插入）
    pub fn seed_defaults(&self) -> AppResult<()> {
        let defaults = vec![
            ("aur_username", "zxp19821005", "AUR 维护者用户名", "aur"),
            (
                "aur_packages_dir",
                "/run/media/zxp/LocalBak/git/My_AUR_Files",
                "本地 AUR 包文件目录",
                "aur",
            ),
            (
                "aur_batch_size",
                "50",
                "AUR 批量查询每批数量上限（最大100）",
                "aur",
            ),
            (
                "aur_batch_interval",
                "5",
                "AUR 批量查询间隔时间（秒）",
                "aur",
            ),
            (
                "backup_dir",
                "/run/media/zxp/Backup/Linux/ZST",
                "默认备份目录",
                "backup",
            ),
            (
                "github_backup_repo",
                "https://github.com/zxp19821005/My_AUR_Files",
                "GitHub 备份仓库地址",
                "backup",
            ),
            ("show_tray_icon", "true", "是否显示系统托盘图标", "general"),
            (
                "close_action",
                "minimize_to_tray",
                "关闭窗口动作 (minimize_to_tray/exit)",
                "general",
            ),
            (
                "log_max_size",
                "10485760",
                "单个日志文件大小上限（字节），默认 10MB",
                "log",
            ),
            (
                "log_max_files",
                "7",
                "保留的日志文件最大数量，默认 7",
                "log",
            ),
            (
                "github_token",
                "",
                "GitHub Personal Access Token（用于提高 API 速率限制）",
                "checker",
            ),
            (
                "gitee_token",
                "",
                "Gitee 私人令牌（access_token，用于提高 API 速率限制）",
                "checker",
            ),
            (
                "gitlab_token",
                "",
                "GitLab Personal Access Token（用于提高 API 速率限制）",
                "checker",
            ),
            ("http_timeout", "30", "HTTP 请求超时时间（秒）", "checker"),
            ("http_retry_count", "2", "HTTP 请求失败重试次数", "checker"),
            // 列表设置
            (
                "list_page_size_software",
                "50",
                "软件管理列表每页显示行数",
                "list",
            ),
            (
                "list_page_size_backup",
                "50",
                "备份管理列表每页显示行数",
                "list",
            ),
            (
                "list_page_size_cache",
                "50",
                "缓存管理列表每页显示行数",
                "list",
            ),
            (
                "list_page_size_proxy",
                "50",
                "代理管理列表每页显示行数",
                "list",
            ),
            (
                "list_page_size_license",
                "50",
                "License管理列表每页显示行数",
                "list",
            ),
            (
                "list_page_size_language",
                "50",
                "编程语言管理列表每页显示行数",
                "list",
            ),
        ];
        for (key, value, description, category) in defaults {
            self.conn.execute(
                "INSERT OR IGNORE INTO settings (key, value, description, category) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![key, value, description, category],
            )?;
        }

        let langs = [
            ("Rust", "rs"),
            ("TypeScript", "ts"),
            ("Python", "py"),
            ("Go", "go"),
            ("C/C++", "c"),
            ("Java", "java"),
            ("Kotlin", "kt"),
            ("C#", "cs"),
            ("Ruby", "rb"),
            ("PHP", "php"),
        ];
        for (name, short_name) in langs {
            self.conn.execute(
                "INSERT OR IGNORE INTO enum_programming_languages (name, short_name) VALUES (?1, ?2)",
                rusqlite::params![name, short_name],
            )?;
        }

        let licenses = [
            ("MIT", "MIT License"),
            ("Apache-2.0", "Apache License 2.0"),
            ("BSD-3-Clause", "BSD 3-Clause License"),
            ("BSD-2-Clause", "BSD 2-Clause License"),
            ("ISC", "ISC License"),
            ("Unlicense", "The Unlicense"),
            ("MPL-2.0", "Mozilla Public License 2.0"),
            (
                "LGPL-3.0-only",
                "GNU Lesser General Public License v3.0 only",
            ),
            ("GPL-3.0-only", "GNU General Public License v3.0 only"),
            ("GPL-2.0-only", "GNU General Public License v2.0 only"),
            (
                "AGPL-3.0-only",
                "GNU Affero General Public License v3.0 only",
            ),
            ("CC0-1.0", "Creative Commons Zero v1.0 Universal"),
        ];
        for (spdx_id, full_name) in licenses {
            self.conn.execute(
                "INSERT OR IGNORE INTO enum_licenses (spdx_id, full_name) VALUES (?1, ?2)",
                rusqlite::params![spdx_id, full_name],
            )?;
        }
        Ok(())
    }
}