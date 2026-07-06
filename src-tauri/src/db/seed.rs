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
            ("aur_packages_dir", "/run/media/zxp/LocalBak/git/My_AUR_Files", "本地 AUR 包文件目录", "aur"),
            ("backup_dir", "/run/media/zxp/Backup/Linux/ZST", "默认备份目录", "backup"),
            ("github_backup_repo", "https://github.com/zxp19821005/My_AUR_Files", "GitHub 备份仓库地址", "backup"),
            ("show_tray_icon", "true", "是否显示系统托盘图标", "general"),
            ("close_action", "minimize_to_tray", "关闭窗口动作 (minimize_to_tray/exit)", "general"),
        ];
        for (key, value, description, category) in defaults {
            self.conn.execute(
                "INSERT OR IGNORE INTO settings (key, value, description, category) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![key, value, description, category],
            )?;
        }

        let langs = vec![
            ("Rust", "Rust 编程语言", ".rs,.toml", "cargo", "cargo build"),
            ("TypeScript", "TypeScript 编程语言", ".ts,.tsx,.js,.jsx,.mjs", "npm/pnpm/yarn", "npm run build"),
            ("Python", "Python 编程语言", ".py,.pyw", "pip/poetry", "python setup.py"),
            ("Go", "Go 编程语言", ".go", "go", "go build"),
            ("C/C++", "C/C++ 编程语言", ".c,.cpp,.h,.hpp", "make/cmake", "make"),
            ("Java", "Java 编程语言", ".java,.jar", "maven/gradle", "mvn package"),
            ("Kotlin", "Kotlin 编程语言", ".kt,.kts", "gradle", "gradle build"),
            ("C#", "C# 编程语言", ".cs,.csproj", "dotnet", "dotnet build"),
            ("Ruby", "Ruby 编程语言", ".rb", "gem/bundler", "bundle exec rake"),
            ("PHP", "PHP 编程语言", ".php", "composer", "composer install"),
        ];
        for (name, desc, exts, build_sys, build_cmd) in langs {
            self.conn.execute(
                "INSERT OR IGNORE INTO enum_programming_languages (name, description, file_extensions, build_system, build_command) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![name, desc, exts, build_sys, build_cmd],
            )?;
        }
        Ok(())
    }
}
