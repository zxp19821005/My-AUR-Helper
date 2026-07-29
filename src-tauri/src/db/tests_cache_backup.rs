/**
 * tests_cache_backup.rs - cache_software / backup_software 表功能测试
 *
 * 测试内容：
 * 1. 两表 schema 设计一致性（基础字段：唯一标识、版本信息、存储路径、创建/更新时间）
 * 2. 旧版本表结构迁移（补齐 full_path/created_at/updated_at 并回填数据）
 * 3. 缓存管理页面数据源（insert → get_all_cache_entries 全量读取）
 */
use std::path::Path;

use crate::models::CacheSoftware;

use super::Database;

/// 创建内存数据库并完成初始化
fn setup_db() -> Database {
    let db = Database::new(Path::new(":memory:")).expect("创建内存数据库失败");
    db.initialize().expect("数据库初始化失败");
    db
}

/// 构造一条测试用缓存记录
fn sample_cache(filename: &str, name: &str, version: &str, dir: &str) -> CacheSoftware {
    CacheSoftware {
        id: None,
        software_id: 0,
        filename: filename.to_string(),
        name: name.to_string(),
        epoch: 0,
        version: version.to_string(),
        pkgrel: "1".to_string(),
        arch: "x86_64".to_string(),
        size: 1024,
        source_dir: Some("系统缓存".to_string()),
        cache_directory: dir.to_string(),
        full_path: format!("{}/{}", dir, filename),
        created_at: None,
        updated_at: None,
    }
}

/// 验证 cache_software 与 backup_software 的基础字段设计一致
#[test]
fn test_schema_consistency_between_cache_and_backup() {
    let db = setup_db();
    let cache_cols = db.get_table_columns("cache_software").unwrap();
    let backup_cols = db.get_table_columns("backup_software").unwrap();

    // 两表共有的基础字段：唯一标识、文件名、版本信息、存储路径、创建/更新时间
    for col in [
        "id",
        "filename",
        "epoch",
        "pkgrel",
        "arch",
        "full_path",
        "created_at",
        "updated_at",
    ] {
        assert!(
            cache_cols.contains(&col.to_string()),
            "cache_software 缺少基础字段 {}",
            col
        );
        assert!(
            backup_cols.contains(&col.to_string()),
            "backup_software 缺少基础字段 {}",
            col
        );
    }

    // 缓存业务特有字段
    for col in [
        "software_id",
        "name",
        "version",
        "size",
        "source_dir",
        "cache_directory",
    ] {
        assert!(
            cache_cols.contains(&col.to_string()),
            "cache_software 缺少特有字段 {}",
            col
        );
    }

    // 备份业务特有字段
    for col in ["pkgver", "subdirectory"] {
        assert!(
            backup_cols.contains(&col.to_string()),
            "backup_software 缺少特有字段 {}",
            col
        );
    }
}

/// 验证旧版本 cache_software 表迁移后补齐新字段且数据保留、full_path 正确回填
#[test]
fn test_migrate_cache_software_from_old_schema() {
    let db = Database::new(Path::new(":memory:")).expect("创建内存数据库失败");
    // 模拟上一版本的 11 列表结构（无 full_path/created_at/updated_at）
    db.conn
        .execute_batch(
            "CREATE TABLE cache_software (
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
            );
            INSERT INTO cache_software (software_id, filename, name, epoch, version, pkgrel, arch, size, source_dir, cache_directory)
            VALUES (0, 'foo-bin-1.0.0-1-x86_64.pkg.tar.zst', 'foo-bin', 0, '1.0.0', '1', 'x86_64', 2048, '系统缓存', '/var/cache/pacman/pkg');",
        )
        .unwrap();

    db.initialize().expect("迁移旧表失败");

    let cols = db.get_table_columns("cache_software").unwrap();
    for col in ["full_path", "created_at", "updated_at"] {
        assert!(cols.contains(&col.to_string()), "迁移后缺少字段 {}", col);
    }

    // 旧数据保留且 full_path 回填为 cache_directory/filename
    let entries = db.get_all_cache_software().unwrap();
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(e.name, "foo-bin");
    assert_eq!(e.version, "1.0.0");
    assert_eq!(
        e.full_path,
        "/var/cache/pacman/pkg/foo-bin-1.0.0-1-x86_64.pkg.tar.zst"
    );
    assert!(e.created_at.is_some(), "created_at 未回填");
    assert!(e.updated_at.is_some(), "updated_at 未回填");
}

/// 验证旧版本 backup_software 表迁移后补齐时间戳字段且数据保留
#[test]
fn test_migrate_backup_software_from_old_schema() {
    let db = Database::new(Path::new(":memory:")).expect("创建内存数据库失败");
    // 模拟上一版本的 8 列表结构（无 created_at/updated_at）
    db.conn
        .execute_batch(
            "CREATE TABLE backup_software (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                filename     TEXT NOT NULL,
                epoch        INTEGER NOT NULL DEFAULT 0,
                pkgver       TEXT NOT NULL DEFAULT '',
                pkgrel       TEXT NOT NULL DEFAULT '1',
                arch         TEXT NOT NULL DEFAULT 'x86_64',
                subdirectory TEXT,
                full_path    TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO backup_software (filename, epoch, pkgver, pkgrel, arch, subdirectory, full_path)
            VALUES ('foo-1.0.0-1-x86_64.pkg.tar.zst', 0, '1.0.0', '1', 'x86_64', 'sub', '/backup/sub/foo-1.0.0-1-x86_64.pkg.tar.zst');",
        )
        .unwrap();

    db.initialize().expect("迁移旧表失败");

    let cols = db.get_table_columns("backup_software").unwrap();
    for col in ["created_at", "updated_at"] {
        assert!(cols.contains(&col.to_string()), "迁移后缺少字段 {}", col);
    }

    // 旧数据保留且 full_path 不被覆盖
    let entries = db.get_all_backup_software().unwrap();
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(e.pkgver, "1.0.0");
    assert_eq!(e.full_path, "/backup/sub/foo-1.0.0-1-x86_64.pkg.tar.zst");
    assert!(e.created_at.is_some(), "created_at 未回填");
    assert!(e.updated_at.is_some(), "updated_at 未回填");
}

/// 验证缓存管理页面数据源：插入后 get_all_cache_entries 可读取全部存量数据
#[test]
fn test_cache_entries_page_load_data_source() {
    let db = setup_db();
    db.insert_cache_software(&sample_cache(
        "bar-2.0.0-1-x86_64.pkg.tar.zst",
        "bar",
        "2.0.0",
        "/var/cache/pacman/pkg",
    ))
    .unwrap();
    // name 为空时应从文件名解析包名
    db.insert_cache_software(&sample_cache(
        "baz-git-r100.abcdef-1-any.pkg.tar.zst",
        "",
        "r100.abcdef",
        "/var/cache/pacman/pkg",
    ))
    .unwrap();

    let entries = db.get_all_cache_entries().unwrap();
    assert_eq!(entries.len(), 2, "页面初始加载应返回全部存量数据");

    let bar = entries
        .iter()
        .find(|e| e.pkgname == "bar")
        .expect("缺少 bar 记录");
    assert_eq!(bar.version, "2.0.0");
    assert_eq!(
        bar.full_path,
        "/var/cache/pacman/pkg/bar-2.0.0-1-x86_64.pkg.tar.zst"
    );
    assert!(bar.created_at.is_some(), "created_at 应由数据库默认值填充");
    assert!(bar.updated_at.is_some(), "updated_at 应由数据库默认值填充");

    // name 为空的记录从文件名解析出包名
    assert!(
        entries.iter().any(|e| e.pkgname == "baz-git"),
        "应从文件名解析出包名 baz-git"
    );
}

/// 验证清空缓存表后 get_all_cache_entries 返回空列表（页面清空后刷新场景）
#[test]
fn test_clear_cache_software_then_reload_empty() {
    let db = setup_db();
    db.insert_cache_software(&sample_cache(
        "qux-1.0-1-x86_64.pkg.tar.zst",
        "qux",
        "1.0",
        "/tmp/cache",
    ))
    .unwrap();
    assert_eq!(db.get_all_cache_entries().unwrap().len(), 1);

    let removed = db.clear_cache_software().unwrap();
    assert_eq!(removed, 1);
    assert!(db.get_all_cache_entries().unwrap().is_empty());
}
