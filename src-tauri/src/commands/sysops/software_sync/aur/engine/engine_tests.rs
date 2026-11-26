// @LINE_LIMIT_EXEMPT: 测试文件
/**
 * engine_tests.rs - AUR 同步引擎（包名解析 + 单包写入）单元测试
 *
 * 覆盖点：
 * 1. upsert_one 新建本地没有的包：upstream_url 留空、按包名推断类型/检查器、aur_info 落库
 * 2. upsert_one 更新已存在的包：保留用户配置的 upstream_url
 * 3. merge_pkgnames：本地与维护者包名取并集并去重
 * 4. resolve_pkgnames(include_maintained=false)：只返回本地已有包名，不发网络请求
 */
use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;

use reqwest::Client;

use crate::db::Database;
use crate::models::{CheckerType, PackageType, SoftwareInfo};

use super::{merge_pkgnames, resolve_pkgnames, upsert_one};

/// 创建内存数据库并完成初始化
fn setup_db() -> Database {
    let db = Database::new(Path::new(":memory:")).expect("创建内存数据库失败");
    db.initialize().expect("数据库初始化失败");
    db
}

/// 构造测试用软件包记录
fn sample_sw(pkgname: &str, upstream: Option<&str>) -> SoftwareInfo {
    SoftwareInfo {
        software_id: None,
        pkgname: pkgname.to_string(),
        upstream_url: upstream.map(|s| s.to_string()),
        package_type_id: PackageType::Compiled,
        checker_type_id: CheckerType::GitHubTags,
        is_outdated: false,
        check_test_versions: false,
        check_binary_files: false,
        auto_check_enabled: true,
        skip_check_upstream: false,
        language_ids: vec![],
        version_extract_regex: None,
    }
}

/// 本地没有的包：应新建，且 upstream_url 留空（AUR URL 是项目主页）
#[test]
fn test_upsert_one_creates_new_with_empty_upstream_url() {
    let db = setup_db();
    let data = serde_json::json!({
        "Name": "foo-bin",
        "Description": "测试二进制包",
        "Version": "1.2.3-1",
        "URL": "https://aur.example/foo",
        "License": ["MIT"],
        "LastModified": 1700000000,
    });

    let created = upsert_one(&db, "foo-bin", &data).unwrap();
    assert!(created, "本地不存在的包应为新建");

    let stored = db.get_software_by_name("foo-bin").unwrap().unwrap();
    assert!(stored.upstream_url.is_none());
    assert_eq!(stored.package_type_id, PackageType::Binary);
    assert_eq!(stored.checker_type_id, CheckerType::GitHubAPI);
    assert!(stored.check_binary_files);

    let sid = stored.software_id.unwrap();
    let aur = db.get_aur_info(sid).unwrap().unwrap();
    assert_eq!(aur.aur_version.as_deref(), Some("1.2.3-1"));
    assert_eq!(aur.pkgdesc.as_deref(), Some("测试二进制包"));
    assert!(aur.last_sync_error.is_none());
}

/// 已存在的包：更新时保留用户配置的 upstream_url，不被 AUR 数据覆盖
#[test]
fn test_upsert_one_keeps_existing_upstream_url() {
    let db = setup_db();
    db.insert_software(&sample_sw("bar", Some("https://user-configured.example")))
        .unwrap();
    let data = serde_json::json!({
        "Name": "bar",
        "Description": "已存在的包",
        "Version": "2.0.0-1",
        "URL": "https://aur.example/bar",
    });

    let created = upsert_one(&db, "bar", &data).unwrap();
    assert!(!created, "已存在的包应为更新");

    let stored = db.get_software_by_name("bar").unwrap().unwrap();
    assert_eq!(
        stored.upstream_url.as_deref(),
        Some("https://user-configured.example"),
        "更新时必须保留用户配置的上游地址"
    );
}

/// merge_pkgnames：本地与维护者包名取并集并去重
#[test]
fn test_merge_pkgnames_dedups_and_unions() {
    let local: HashSet<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
    let merged = merge_pkgnames(local, vec!["b".to_string(), "c".to_string()]);
    let merged_set: HashSet<String> = merged.into_iter().collect();
    assert_eq!(merged_set.len(), 3, "并集去重后应为 3 个");
    for name in ["a", "b", "c"] {
        assert!(merged_set.contains(name), "并集应包含 {}", name);
    }
}

/// resolve_pkgnames（不含维护者）：只返回本地已有包名
#[tokio::test]
async fn test_resolve_pkgnames_local_only() {
    let db = setup_db();
    db.insert_software(&sample_sw("local-a", None)).unwrap();
    db.insert_software(&sample_sw("local-b", None)).unwrap();
    let db = Mutex::new(db);
    let client = Client::new();

    let names = resolve_pkgnames(&client, &db, false).await.unwrap();
    assert_eq!(names.len(), 2, "未启用维护者并入时应只有本地 2 个包");
    assert!(names.contains(&"local-a".to_string()));
    assert!(names.contains(&"local-b".to_string()));
}

/// 契约 1 的降级路径：include_maintained=true 但未配置 aur_username
/// → 仅返回本地集合、不报错、且确实不发起任何网络请求。
///
/// 手法：把 AUR 域名强制解析到不可达的 127.0.0.1:1，一旦真的发请求会立刻连接失败，
/// 因此「返回 Ok(本地集合)」即可反证没有触网。
#[tokio::test]
async fn test_resolve_pkgnames_degrades_to_local_without_username() {
    let db = setup_db();
    // 种子数据默认写入 aur_username=zxp19821005，这里清空以模拟未配置
    db.set_setting("aur_username", "").unwrap();
    db.insert_software(&sample_sw("only-local-bin", None))
        .unwrap();
    let db = Mutex::new(db);

    let unreachable = std::net::SocketAddr::from(([127, 0, 0, 1], 1));
    let client = Client::builder()
        .resolve("aur.archlinux.org", unreachable)
        .build()
        .expect("构造探测用 client 失败");

    let names = resolve_pkgnames(&client, &db, true)
        .await
        .expect("用户名为空应降级为仅本地、不得报错");
    assert_eq!(names.len(), 1, "降级后应只有本地 1 个包");
    assert!(names.contains(&"only-local-bin".to_string()));
}
