/**
 * tests.rs - 内存缓存模块单元测试
 *
 * 覆盖缓存生命周期核心路径：miss 回源 → 命中复用 → 失效重载 →
 * 写盘 → 磁盘重建 → 过期丢弃 → 禁用直通 → Settings 不落盘。
 * 使用系统临时目录，测试结束清理。
 * 注意：本文件由 cache/mod.rs 的 `#[cfg(test)] mod tests;` 声明，顶层即模块体。
 */
use std::path::PathBuf;

use super::config::CacheConfig;
use super::domain::CacheDomain;
use super::manager::CacheManager;
use super::persistence;

/// 构造启用状态的测试配置（目录为指定临时目录）
fn test_config(dir: PathBuf) -> CacheConfig {
    CacheConfig {
        enabled: true,
        max_entries: 100,
        ttl_secs: 300,
        write_interval_secs: 0,
        dir,
    }
}

/// 创建唯一临时目录并清理旧数据
fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "my_aur_helper_cache_test_{}_{}",
        tag,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// 首次 miss 回源填充，第二次命中不重复调用 loader
#[test]
fn get_or_load_miss_then_hit() {
    let dir = temp_dir("miss_hit");
    let mut cm = CacheManager::new(test_config(dir.clone()));
    let mut loads = 0usize;
    let data: Vec<i64> = cm
        .get_or_load(CacheDomain::Languages, || {
            loads += 1;
            Ok(vec![1, 2, 3])
        })
        .unwrap();
    assert_eq!(data, vec![1, 2, 3]);
    assert_eq!(loads, 1);

    // 第二次命中，loader 不再执行
    let data2: Vec<i64> = cm
        .get_or_load(CacheDomain::Languages, || {
            loads += 1;
            Ok(vec![9])
        })
        .unwrap();
    assert_eq!(data2, vec![1, 2, 3]);
    assert_eq!(loads, 1);
    std::fs::remove_dir_all(&dir).ok();
}

/// 失效后强制回源重建
#[test]
fn invalidate_forces_reload() {
    let dir = temp_dir("invalidate");
    let mut cm = CacheManager::new(test_config(dir.clone()));
    cm.get_or_load(CacheDomain::Settings, || Ok(1i64)).unwrap();
    cm.invalidate(CacheDomain::Settings);
    let data: i64 = cm
        .get_or_load(CacheDomain::Settings, || Ok(2i64))
        .unwrap();
    assert_eq!(data, 2);
    std::fs::remove_dir_all(&dir).ok();
}

/// 写盘后可从磁盘重建缓存（重启恢复场景）
#[test]
fn flush_and_load_from_disk() {
    let dir = temp_dir("flush");
    let mut cm = CacheManager::new(test_config(dir.clone()));
    cm.get_or_load(CacheDomain::Licenses, || Ok(vec![10, 20]))
        .unwrap();
    let written = cm.flush().unwrap();
    assert_eq!(written, 1);
    assert!(dir.join("licenses.json").exists());

    // 新实例从磁盘加载，命中磁盘数据而非回源
    let mut cm2 = CacheManager::new(test_config(dir.clone()));
    cm2.load_from_disk().unwrap();
    let data: Vec<i64> = cm2
        .get_or_load(CacheDomain::Licenses, || Ok(vec![0]))
        .unwrap();
    assert_eq!(data, vec![10, 20]);
    std::fs::remove_dir_all(&dir).ok();
}

/// 磁盘缓存过期后加载时丢弃并回源
#[test]
fn expired_entry_dropped_on_disk_load() {
    let dir = temp_dir("expired");
    let mut cm = CacheManager::new(test_config(dir.clone()));
    cm.get_or_load(CacheDomain::Languages, || Ok(vec![1]))
        .unwrap();
    cm.flush().unwrap();
    // 手动把过期时间改为过去
    let path = dir.join("languages.json");
    let mut pc: persistence::PersistedCache =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    pc.meta.expires_at = 1;
    std::fs::write(&path, serde_json::to_vec(&pc).unwrap()).unwrap();

    let mut cm2 = CacheManager::new(test_config(dir.clone()));
    cm2.load_from_disk().unwrap();
    let data: Vec<i64> = cm2
        .get_or_load(CacheDomain::Languages, || Ok(vec![2]))
        .unwrap();
    assert_eq!(data, vec![2]); // 过期被丢弃，回源加载
    std::fs::remove_dir_all(&dir).ok();
}

/// 禁用缓存时每次读取都走 loader（行为与未引入缓存一致）
#[test]
fn disabled_cache_always_loads() {
    let dir = temp_dir("disabled");
    let mut cfg = test_config(dir.clone());
    cfg.enabled = false;
    let mut cm = CacheManager::new(cfg);
    let mut loads = 0usize;
    cm.get_or_load(CacheDomain::Settings, || {
        loads += 1;
        Ok(42i64)
    })
    .unwrap();
    cm.get_or_load(CacheDomain::Settings, || {
        loads += 1;
        Ok(42i64)
    })
    .unwrap();
    assert_eq!(loads, 2);
    std::fs::remove_dir_all(&dir).ok();
}

/// Settings 域不落盘（含敏感凭据，仅内存缓存）
#[test]
fn settings_domain_not_persisted() {
    let dir = temp_dir("settings_no_persist");
    let mut cm = CacheManager::new(test_config(dir.clone()));
    cm.get_or_load(CacheDomain::Settings, || Ok(1i64)).unwrap();
    let written = cm.flush().unwrap();
    assert_eq!(written, 0);
    assert!(!dir.join("settings.json").exists());
    std::fs::remove_dir_all(&dir).ok();
}
