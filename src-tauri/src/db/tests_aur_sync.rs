/**
 * tests_aur_sync.rs - 按维护者 AUR 同步的 software_info 写入行为测试
 *
 * 测试内容：
 * 1. 新建包时 upstream_url 强制留空（即使传入非 NULL 也不写入）
 * 2. 已存在包更新时保留原 upstream_url，不被 AUR 数据覆盖
 * 3. 已存在包更新时同步 AUR 可推断字段（包类型/检查器类型/测试与二进制标记）
 * 4. 已存在包更新时 upsert 本身不直接写 is_outdated（避免全量覆盖）
 * 5. recompute_outdated_from_aur 依据 AUR/上游版本比较重算 is_outdated：
 *    AUR 落后置位、追上清除、无上游版本不动、版本相同 + OutOfDate 兜底
 */
use std::path::Path;

use crate::models::{CheckerType, PackageType, SoftwareInfo, UpstreamInfo};

use super::Database;

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

/// 新建包：即使传入上游地址，也必须留空
#[test]
fn test_new_package_upstream_url_is_empty() {
    let db = setup_db();
    let sw = sample_sw("foo-bin", Some("https://should-not-be-stored.example"));
    let (sid, created) = db.upsert_software_from_aur(&sw).unwrap();

    assert!(created, "首次导入应为新建");
    let stored = db.get_software_by_name("foo-bin").unwrap().unwrap();
    assert_eq!(stored.software_id, Some(sid));
    assert!(
        stored.upstream_url.is_none(),
        "新建包的上游地址必须留空，实际为 {:?}",
        stored.upstream_url
    );
}

/// 已存在包：更新时保留原上游地址，不被覆盖
#[test]
fn test_existing_package_keeps_upstream_url() {
    let db = setup_db();
    // 先建一条带上游地址的记录（模拟用户手动配置）
    let initial = sample_sw("bar", Some("https://user-configured.example"));
    db.insert_software(&initial).unwrap();

    // AUR 同步时传入不同的上游地址，应被忽略
    let from_aur = sample_sw("bar", Some("https://aur-homepage.example"));
    let (_, created) = db.upsert_software_from_aur(&from_aur).unwrap();

    assert!(!created, "已存在包应为更新");
    let stored = db.get_software_by_name("bar").unwrap().unwrap();
    assert_eq!(
        stored.upstream_url.as_deref(),
        Some("https://user-configured.example"),
        "更新时上游地址必须保留原值"
    );
}

/// 已存在包：更新 AUR 可推断字段（包类型/检查器类型/测试与二进制标记）
#[test]
fn test_existing_package_updates_inferred_fields() {
    let db = setup_db();
    db.insert_software(&sample_sw("baz-bin", None)).unwrap();

    let mut from_aur = sample_sw("baz-bin", None);
    from_aur.package_type_id = PackageType::Binary;
    from_aur.checker_type_id = CheckerType::GitHubAPI;
    from_aur.check_binary_files = true;
    db.upsert_software_from_aur(&from_aur).unwrap();

    let stored = db.get_software_by_name("baz-bin").unwrap().unwrap();
    assert_eq!(stored.package_type_id, PackageType::Binary);
    assert_eq!(stored.checker_type_id, CheckerType::GitHubAPI);
    assert!(stored.check_binary_files);
}

/// 构造已存在且带上游版本的软件包，返回软件 ID
fn setup_with_upstream(pkgname: &str, upstream_version: &str) -> (Database, i64) {
    let db = setup_db();
    db.insert_software(&sample_sw(pkgname, Some("https://upstream.example")))
        .unwrap();
    let sid = db
        .get_software_by_name(pkgname)
        .unwrap()
        .unwrap()
        .software_id
        .unwrap();
    let info = UpstreamInfo {
        software_id: sid,
        upstream_version: Some(upstream_version.to_string()),
        upstream_license_id: None,
        last_checked: None,
        upstream_url_status: None,
        last_check_error: None,
    };
    db.upsert_upstream_info(&info).unwrap();
    (db, sid)
}

/// AUR 版本落后于上游：重算后应标记需更新
#[test]
fn test_recompute_outdated_when_aur_behind() {
    let (db, sid) = setup_with_upstream("outdated-bin", "2.0.0");
    db.recompute_outdated_from_aur(sid, Some("1.0.0-1"), false, "outdated-bin")
        .unwrap();
    let stored = db.get_software_by_name("outdated-bin").unwrap().unwrap();
    assert!(stored.is_outdated, "AUR 版本落后应标记需更新");
}

/// AUR 版本追上上游：重算后应清除旧的待更新标记（修复 stale 标记）
#[test]
fn test_recompute_outdated_clears_when_aur_catches_up() {
    let (db, sid) = setup_with_upstream("caught-bin", "3.0.9");
    // 模拟旧数据：AUR 已发布但仍停留在"需更新"
    db.update_software_outdated(sid, true).unwrap();
    db.recompute_outdated_from_aur(sid, Some("3.0.9-1"), false, "caught-bin")
        .unwrap();
    let stored = db.get_software_by_name("caught-bin").unwrap().unwrap();
    assert!(!stored.is_outdated, "AUR 追上后应清除需更新标记");
}

/// 无上游版本（新建包场景）：不翻转已有标记
#[test]
fn test_recompute_outdated_skips_without_upstream() {
    let db = setup_db();
    db.insert_software(&sample_sw("no-upstream-bin", None))
        .unwrap();
    let sid = db
        .get_software_by_name("no-upstream-bin")
        .unwrap()
        .unwrap()
        .software_id
        .unwrap();
    db.update_software_outdated(sid, true).unwrap();
    db.recompute_outdated_from_aur(sid, Some("1.0.0-1"), false, "no-upstream-bin")
        .unwrap();
    let stored = db.get_software_by_name("no-upstream-bin").unwrap().unwrap();
    assert!(stored.is_outdated, "无上游版本时不应翻转已有标记");
}

/// 版本相同但 AUR 标记 OutOfDate：作为补充信号标记需更新
#[test]
fn test_recompute_outdated_with_aur_out_of_date_signal() {
    let (db, sid) = setup_with_upstream("flagged-bin", "1.2.3");
    db.recompute_outdated_from_aur(sid, Some("1.2.3"), true, "flagged-bin")
        .unwrap();
    let stored = db.get_software_by_name("flagged-bin").unwrap().unwrap();
    assert!(stored.is_outdated, "版本相同 + AUR OutOfDate 应提示更新");
}
