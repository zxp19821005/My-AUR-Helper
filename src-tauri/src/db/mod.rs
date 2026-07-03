/**
 * db/mod.rs - 数据库操作模块
 *
 * 使用 SQLite (rusqlite) 作为后端存储
 * 每个数据表对应一个独立的子模块文件
 * 所有子模块通过 impl Database 扩展 Database 结构体的方法
 */
mod aur_info;                       // AUR 包信息表操作
mod backup_software;                // 备份软件包表操作
mod cache_software;                 // 缓存软件包表操作
mod enum_licenses;                  // License 枚举表操作
mod enum_programming_languages;     // 编程语言枚举表操作
mod logs;                           // 日志表操作
mod proxies_info;                   // 代理信息表操作
mod proxies_test;                   // 代理测试结果表操作
mod settings;                       // 设置表操作
mod software_info;                  // 软件包信息表操作
mod upstream_info;                  // 上游版本信息表操作

use std::path::Path;                // 文件路径

use anyhow::Result;                 // 通用错误处理
use rusqlite::Connection;           // SQLite 连接

/// 数据库结构体，包装 rusqlite 连接
pub struct Database {
    conn: Connection,  // SQLite 数据库连接
}

impl Database {
    /// 打开或创建数据库文件
    /// @param path - 数据库文件路径
    /// @returns Database 实例
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;  // 打开 SQLite 数据库文件
        // 启用 WAL 模式提高并发性能，启用外键约束保证数据完整性
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn })
    }

    /// 初始化数据库表结构和默认数据
    /// 创建所有必要的表（如不存在），并插入默认设置和编程语言
    pub fn initialize(&self) -> Result<()> {
        // 执行批量 SQL 创建所有表
        self.conn.execute_batch(
            "
            -- 软件包信息表
            CREATE TABLE IF NOT EXISTS software_info (
                software_id         INTEGER PRIMARY KEY AUTOINCREMENT, -- 软件包唯一 ID
                pkgname             TEXT NOT NULL UNIQUE,              -- 包名（唯一）
                upstream_url        TEXT,                              -- 上游项目地址
                package_type        INTEGER NOT NULL DEFAULT 1,        -- 包类型（编译/二进制/Git/AppImage）
                checker_type        INTEGER NOT NULL DEFAULT 7,        -- 检查器类型（GitHub/Gitee/等，默认 Manual）
                is_outdated         INTEGER NOT NULL DEFAULT 0,        -- 是否有更新（0=否，1=是）
                check_test_versions INTEGER NOT NULL DEFAULT 0,        -- 是否检查测试版本
                check_binary_files  INTEGER NOT NULL DEFAULT 0,        -- 是否检查二进制文件
                auto_check_enabled  INTEGER NOT NULL DEFAULT 1,        -- 是否启用自动检查
                license_id          INTEGER,                           -- 关联的 License ID
                language_id         INTEGER,                           -- 关联的编程语言 ID
                created_at          INTEGER NOT NULL DEFAULT (unixepoch()), -- 创建时间戳
                FOREIGN KEY (license_id) REFERENCES enum_licenses(id),
                FOREIGN KEY (language_id) REFERENCES enum_programming_languages(id)
            );

            -- AUR 包详情信息表
            CREATE TABLE IF NOT EXISTS aur_info (
                software_id     INTEGER PRIMARY KEY,  -- 关联的软件包 ID
                pkgdesc         TEXT,                  -- 包描述
                aur_version     TEXT,                  -- AUR 中的版本号
                license_id      INTEGER,               -- License ID
                last_updated    INTEGER,               -- 最后更新时间（Unix 时间戳）
                depends         TEXT,                  -- 依赖（JSON 数组字符串）
                makedepends     TEXT,                  -- 构建依赖
                optdepends      TEXT,                  -- 可选依赖
                out_of_date     INTEGER,               -- 是否过期
                FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE,
                FOREIGN KEY (license_id) REFERENCES enum_licenses(id)
            );

            -- 上游版本信息表
            CREATE TABLE IF NOT EXISTS upstream_info (
                software_id      INTEGER PRIMARY KEY, -- 关联的软件包 ID
                upstream_url     TEXT,                 -- 上游项目 URL
                upstream_version TEXT,                 -- 检测到的上游版本号
                upstream_license TEXT,                 -- 上游项目 License
                last_checked     TEXT,                 -- 最后检查时间
                FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE
            );

            -- 备份软件包记录表
            CREATE TABLE IF NOT EXISTS backup_software (
                id           INTEGER PRIMARY KEY AUTOINCREMENT, -- 记录 ID
                software_id  INTEGER NOT NULL,                  -- 软件包 ID
                filename     TEXT NOT NULL,                     -- 备份文件名
                epoch        INTEGER NOT NULL DEFAULT 0,        -- epoch 版本
                pkgrel       TEXT NOT NULL DEFAULT '1',         -- 包发布号
                arch         TEXT NOT NULL DEFAULT 'x86_64',    -- 架构
                subdirectory TEXT,                              -- 子目录
                FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE
            );

            -- 缓存软件包记录表
            CREATE TABLE IF NOT EXISTS cache_software (
                id             INTEGER PRIMARY KEY AUTOINCREMENT, -- 记录 ID
                software_id    INTEGER NOT NULL,                  -- 软件包 ID
                filename       TEXT NOT NULL,                     -- 缓存文件名
                epoch          INTEGER NOT NULL DEFAULT 0,        -- epoch 版本
                pkgrel         TEXT NOT NULL DEFAULT '1',         -- 包发布号
                arch           TEXT NOT NULL DEFAULT 'x86_64',    -- 架构
                cache_directory TEXT NOT NULL,                    -- 缓存目录路径
                FOREIGN KEY (software_id) REFERENCES software_info(software_id) ON DELETE CASCADE
            );

            -- 代理信息表
            CREATE TABLE IF NOT EXISTS proxies_info (
                proxy_id    INTEGER PRIMARY KEY AUTOINCREMENT, -- 代理 ID
                proxy_name  TEXT NOT NULL,                      -- 代理名称
                proxy_type  TEXT NOT NULL DEFAULT 'download',   -- 代理类型（download/clone/raw/ssh）
                url         TEXT NOT NULL UNIQUE,               -- 代理 URL（唯一）
                is_active   INTEGER NOT NULL DEFAULT 1          -- 是否启用（0=禁用，1=启用）
            );

            -- 代理测试结果表
            CREATE TABLE IF NOT EXISTS proxies_test (
                id            INTEGER PRIMARY KEY AUTOINCREMENT, -- 测试记录 ID
                proxy_id      INTEGER NOT NULL,                  -- 关联的代理 ID
                test_time     TEXT,                              -- 测试时间
                avg_latency   INTEGER,                           -- 平均延迟（毫秒）
                success_count INTEGER NOT NULL DEFAULT 0,        -- 成功次数
                fail_count    INTEGER NOT NULL DEFAULT 0,        -- 失败次数
                FOREIGN KEY (proxy_id) REFERENCES proxies_info(proxy_id) ON DELETE CASCADE
            );

            -- License 枚举表
            CREATE TABLE IF NOT EXISTS enum_licenses (
                id              INTEGER PRIMARY KEY AUTOINCREMENT, -- License ID
                spdx_id         TEXT NOT NULL UNIQUE,              -- SPDX 标准 ID（唯一）
                full_name       TEXT NOT NULL,                     -- License 完整名称
                url             TEXT,                              -- License 参考 URL
                is_deprecated   INTEGER NOT NULL DEFAULT 0,        -- 是否已弃用
                is_osi_approved INTEGER NOT NULL DEFAULT 0,        -- 是否被 OSI 批准
                description     TEXT,                              -- 描述
                category        TEXT,                              -- 分类
                created_at      TEXT NOT NULL DEFAULT (datetime('now')) -- 创建时间
            );

            -- 编程语言枚举表
            CREATE TABLE IF NOT EXISTS enum_programming_languages (
                id              INTEGER PRIMARY KEY AUTOINCREMENT, -- 语言 ID
                name            TEXT NOT NULL UNIQUE,              -- 语言名称（唯一）
                description     TEXT,                              -- 描述
                file_extensions TEXT,                              -- 文件扩展名（逗号分隔）
                build_system    TEXT,                              -- 构建系统
                build_command   TEXT                               -- 构建命令
            );

            -- 日志表
            CREATE TABLE IF NOT EXISTS logs (
                id         INTEGER PRIMARY KEY AUTOINCREMENT, -- 日志 ID
                level      TEXT NOT NULL,                      -- 日志级别（INFO/WARN/ERROR/DEBUG）
                message    TEXT NOT NULL,                      -- 日志消息
                module     TEXT,                               -- 来源模块
                created_at TEXT NOT NULL DEFAULT (datetime('now')) -- 创建时间
            );

            -- 设置表
            CREATE TABLE IF NOT EXISTS settings (
                id          INTEGER PRIMARY KEY AUTOINCREMENT, -- 设置 ID
                key         TEXT NOT NULL UNIQUE,              -- 设置键（唯一）
                value       TEXT NOT NULL DEFAULT '',          -- 设置值
                description TEXT,                              -- 设置描述
                category    TEXT NOT NULL DEFAULT 'general',   -- 设置分类
                created_at  TEXT NOT NULL DEFAULT (datetime('now')) -- 创建时间
            );

            -- 索引：加速按包名查询
            CREATE INDEX IF NOT EXISTS idx_software_pkgname ON software_info(pkgname);
            -- 索引：加速过期包查询
            CREATE INDEX IF NOT EXISTS idx_software_outdated ON software_info(is_outdated);
            -- 索引：加速按软件包 ID 查询备份记录
            CREATE INDEX IF NOT EXISTS idx_backup_software_pkg ON backup_software(software_id);
            -- 索引：加速按软件包 ID 查询缓存记录
            CREATE INDEX IF NOT EXISTS idx_cache_software_pkg ON cache_software(software_id);
            -- 索引：加速按代理 ID 查询测试记录
            CREATE INDEX IF NOT EXISTS idx_proxies_test_proxy ON proxies_test(proxy_id);
            -- 索引：加速按时间查询日志
            CREATE INDEX IF NOT EXISTS idx_logs_created ON logs(created_at);
            -- 索引：加速按分类查询设置
            CREATE INDEX IF NOT EXISTS idx_settings_category ON settings(category);
            ",
        )?;
        self.seed_defaults()?; // 插入默认数据
        Ok(())
    }

    /// 插入默认设置和编程语言数据
    /// 仅在表为空时插入（INSERT OR IGNORE）
    fn seed_defaults(&self) -> Result<()> {
        // 默认设置项：[键，值，描述，分类]
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

        // 默认编程语言：[名称，描述，扩展名，构建系统，构建命令]
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
