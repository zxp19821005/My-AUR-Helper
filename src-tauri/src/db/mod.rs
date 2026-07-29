/**
 * db/mod.rs - 数据库操作模块入口
 *
 * 模块结构：
 * - connection.rs — Database 结构体定义、连接创建、表初始化和迁移
 * - schema.rs     — 表结构定义（CREATE TABLE）
 * - migration_backup.rs — backup_software 表结构迁移
 * - migration_cache.rs  — cache_software 表结构迁移
 * - migration_proxy.rs  — proxies_info 和 proxies_test 表结构迁移
 * - seed.rs       — 默认数据初始化
 * - aur_info.rs   — AUR 包信息表 CRUD
 * - backup_software.rs — 备份记录表 CRUD
 * - cache_software.rs  — 缓存记录表 CRUD
 * - enum_licenses.rs   — License 枚举表 CRUD
 * - enum_programming_languages.rs — 编程语言枚举表 CRUD
 * - logs.rs       — 日志表 CRUD
 * - proxies_info.rs   — 代理信息表 CRUD
 * - proxies_test.rs   — 代理测试表 CRUD
 * - settings.rs   — 设置表 CRUD
 * - software_info.rs  — 软件包信息表 CRUD
 * - upstream_info.rs  — 上游版本信息表 CRUD
 */
mod aur_info;
mod backup_software;
mod cache_software;
mod connection;
mod enum_licenses;
mod enum_programming_languages;
mod logs;
mod migration_aur;
mod migration_backup;
mod migration_cache;
mod migration_enum;
mod migration_proxy;
mod migration_software;
mod migration_upstream;
mod proxies_info;
mod proxies_test;
mod schema;
mod seed;
mod settings;
mod software_info;
mod upstream_info;

#[cfg(test)]
mod tests_cache_backup;

pub use connection::Database;
