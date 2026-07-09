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
            ("log_max_size", "10485760", "单个日志文件大小上限（字节），默认 10MB", "log"),
            ("log_max_files", "7", "保留的日志文件最大数量，默认 7", "log"),
            ("github_token", "", "GitHub Personal Access Token（用于提高 API 速率限制）", "checker"),
            ("gitee_token", "", "Gitee 私人令牌（access_token，用于提高 API 速率限制）", "checker"),
            ("gitlab_token", "", "GitLab Personal Access Token（用于提高 API 速率限制）", "checker"),
            ("http_timeout", "30", "HTTP 请求超时时间（秒）", "checker"),
            ("http_retry_count", "2", "HTTP 请求失败重试次数", "checker"),
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

        // 常见开源 License 默认数据（首次启动可用，完整列表可通过 SPDX 同步获取）
        let licenses = [
            ("MIT", "MIT License", "https://opensource.org/licenses/MIT", false, true, "宽松许可，允许几乎任何用途", "permissive"),
            ("Apache-2.0", "Apache License 2.0", "https://opensource.org/licenses/Apache-2.0", false, true, "宽松许可，含专利授权条款", "permissive"),
            ("BSD-3-Clause", "BSD 3-Clause License", "https://opensource.org/licenses/BSD-3-Clause", false, true, "宽松许可，禁止使用作者名宣传", "permissive"),
            ("BSD-2-Clause", "BSD 2-Clause License", "https://opensource.org/licenses/BSD-2-Clause", false, true, "简化版 BSD 许可", "permissive"),
            ("ISC", "ISC License", "https://opensource.org/licenses/ISC", false, true, "类 MIT 的宽松许可", "permissive"),
            ("Unlicense", "The Unlicense", "https://unlicense.org", false, true, "放弃版权，完全自由使用", "permissive"),
            ("MPL-2.0", "Mozilla Public License 2.0", "https://opensource.org/licenses/MPL-2.0", false, true, "弱 copyleft，文件级授权", "weak_copyleft"),
            ("LGPL-3.0-only", "GNU Lesser General Public License v3.0 only", "https://opensource.org/licenses/LGPL-3.0", false, true, "弱 copyleft，库可被链接", "weak_copyleft"),
            ("GPL-3.0-only", "GNU General Public License v3.0 only", "https://opensource.org/licenses/GPL-3.0", false, true, "强 copyleft，衍生作品须开源", "copyleft"),
            ("GPL-2.0-only", "GNU General Public License v2.0 only", "https://opensource.org/licenses/GPL-2.0", false, true, "强 copyleft，Linux 内核采用", "copyleft"),
            ("AGPL-3.0-only", "GNU Affero General Public License v3.0 only", "https://opensource.org/licenses/AGPL-3.0", false, true, "强 copyleft，含网络服务条款", "copyleft"),
            ("CC0-1.0", "Creative Commons Zero v1.0 Universal", "https://creativecommons.org/publicdomain/zero/1.0", false, true, "公共领域，完全放弃版权", "public_domain"),
        ];
        for (spdx_id, full_name, url, is_deprecated, is_osi_approved, description, category) in licenses {
            self.conn.execute(
                "INSERT OR IGNORE INTO enum_licenses (spdx_id, full_name, url, is_deprecated, is_osi_approved, description, category) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![spdx_id, full_name, url, is_deprecated as i32, is_osi_approved as i32, description, category],
            )?;
        }
        Ok(())
    }
}
