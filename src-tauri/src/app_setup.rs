/**
 * app_setup.rs - Tauri setup 阶段的数据库 / 日志 / 缓存初始化
 *
 * 功能：把 lib.rs 的 setup 闭包中三段较重初始化拆为独立函数：
 * - init_database：创建配置目录、打开并初始化数据库表结构
 * - init_logger：读取日志设置并初始化日志轮转系统、设置 AppHandle
 * - init_memory_cache：读取缓存配置、加载磁盘未过期条目、装入 AppState
 * - spawn_cache_flush_task：注册内存缓存定时写盘任务
 *
 * 工作流程（顺序不得改变）：
 * DB → 日志 → 托盘 → 图标 → 缓存；本文件承载 DB / 日志 / 缓存三段，
 * 托盘与图标由 lib.rs 编排，内存缓存管理在 init_memory_cache 内 app.manage。
 *
 * 适用场景：应用启动初始化，保持与原 setup 闭包逐语句等价。
 */
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::Manager;

use crate::app_state::{get_setting_string, AppState};
use crate::errors::{AppError, AppResult};
use crate::{cache, db, logger};

/// 初始化数据库：创建配置目录并返回已建表的 Database。
///
/// # 参数
/// - `app`: Tauri 应用引用，用于定位 app_config_dir
///
/// # 返回
/// - `Ok(Database)`: 初始化完成的数据库
/// - `Err(AppError)`: 创建目录或建表失败
pub(crate) fn init_database(app: &tauri::App) -> AppResult<db::Database> {
    let app_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::FileOperation(format!("获取配置目录失败: {}", e)))?;
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| AppError::FileOperation(format!("创建配置目录失败: {}", e)))?;
    let db_path = app_dir.join("my_aur_helper.db"); // 数据库文件路径
    let database = db::Database::new(&db_path)
        .map_err(|e| AppError::DatabaseError(format!("数据库初始化失败: {}", e)))?;
    database
        .initialize()
        .map_err(|e| AppError::DatabaseError(format!("数据库表结构初始化失败: {}", e)))?;
    Ok(database)
}

/// 读取日志设置并初始化日志轮转系统，同时注册全局 AppHandle。
///
/// # 参数
/// - `database`: 已初始化的数据库引用
/// - `app`: Tauri 应用引用，用于设置日志事件推送的 AppHandle
/// - `config_dir`: 应用配置目录，日志目录未配置时在其下建 logs
pub(crate) fn init_logger(database: &db::Database, app: &tauri::App, config_dir: &Path) {
    let log_dir_raw = get_setting_string(database, "log_dir", "");
    let log_dir = if log_dir_raw.is_empty() {
        config_dir.join("logs")
    } else {
        // 展开 ~ 为主目录路径
        if log_dir_raw == "~" || log_dir_raw.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(&log_dir_raw[2..])
            } else {
                PathBuf::from(&log_dir_raw)
            }
        } else {
            PathBuf::from(&log_dir_raw)
        }
    };
    let log_prefix = get_setting_string(database, "log_prefix", "applog");
    let log_max_size: u64 = get_setting_string(database, "log_max_size", "10485760")
        .parse()
        .unwrap_or(10485760);
    let log_max_files: usize = get_setting_string(database, "log_max_files", "7")
        .parse()
        .unwrap_or(7);
    logger::update_log_settings(log_max_size, log_max_files);
    let rotating_logger = logger::RotatingLogger::new(log_dir.clone(), log_prefix.clone());
    rotating_logger.init().expect("初始化日志记录器失败");
    // 设置全局 AppHandle 用于日志事件推送
    logger::set_app_handle(app.handle().clone());
    log::info!(
        "日志系统已初始化，目录: {}, 前缀: {}, 最大大小: {}KB, 最大文件数: {}",
        log_dir.display(),
        log_prefix,
        log_max_size / 1024,
        log_max_files
    );
}

/// 初始化内存缓存：读取配置、加载磁盘未过期条目，并 app.manage(AppState)。
///
/// # 参数
/// - `database`: 已初始化的数据库（移入 AppState）
/// - `app`: Tauri 应用引用，用于注册 AppState
///
/// # 返回
/// - 缓存配置（供 spawn_cache_flush_task 使用写盘周期与目录）
pub(crate) fn init_memory_cache(database: db::Database, app: &tauri::App) -> cache::CacheConfig {
    let cache_config = cache::CacheConfig::from_db(&database);
    let mut memory_cache = cache::CacheManager::new(cache_config.clone());
    if let Err(e) = memory_cache.load_from_disk() {
        log::warn!("内存缓存磁盘加载失败（将回源数据库）: {}", e);
    }

    // 将数据库与内存缓存存储到应用状态，供命令使用
    app.manage(AppState {
        db: Mutex::new(database),
        memory_cache: Mutex::new(memory_cache),
    });
    cache_config
}

/// 注册内存缓存定时写盘任务（写入周期 > 0 且缓存启用时）。
///
/// # 参数
/// - `app`: Tauri 应用引用，用于取得 AppHandle
/// - `cache_config`: 缓存配置，提供启用状态、写盘周期与目录
pub(crate) fn spawn_cache_flush_task(app: &tauri::App, cache_config: &cache::CacheConfig) {
    let write_interval = cache_config.write_interval_secs;
    if cache_config.enabled && write_interval > 0 {
        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(write_interval));
            ticker.tick().await; // 跳过首个立即触发的 tick
            loop {
                ticker.tick().await;
                if let Some(state) = handle.try_state::<AppState>() {
                    if let Ok(mut cache) = state.memory_cache.lock() {
                        if let Err(e) = cache.flush() {
                            log::warn!("内存缓存定时写盘失败: {}", e);
                        }
                    }
                }
            }
        });
        log::info!(
            "内存缓存定时写盘已启动: 周期={}秒, 目录={}",
            write_interval,
            cache_config.dir.display()
        );
    }
}
