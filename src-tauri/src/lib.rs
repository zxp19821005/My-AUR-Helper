/**
 * lib.rs - Tauri 应用入口
 *
 * 功能：
 * - 初始化 Tauri 应用
 * - 配置日志系统
 * - 初始化数据库
 * - 创建系统托盘
 * - 注册所有 Tauri 命令
 * - 处理窗口关闭事件
 */
pub mod aur;
pub mod cache;
pub mod checkers;
pub mod commands;
pub mod db;
pub mod errors;
pub mod http_client;
pub mod logger;
pub mod models;
pub mod network;
pub mod proxy;
mod tray;
pub mod versions;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

/// 应用状态
pub struct AppState {
    pub db: Mutex<db::Database>,
    pub memory_cache: Mutex<cache::CacheManager>,
}

/// 窗口关闭动作配置
struct CloseAction(String);

fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.zxp19821005.aur-helper")
}

fn get_setting_string(db: &db::Database, key: &str, default: &str) -> String {
    db.get_setting(key)
        .ok()
        .flatten()
        .map(|s| s.value)
        .unwrap_or_else(|| default.to_string())
}

/// 运行 Tauri 应用
pub fn run() {
    let config_dir = get_config_dir();

    tauri::Builder::default()
        // 配置 Shell 插件，用于执行系统命令
        .plugin(tauri_plugin_shell::init())
        // 应用初始化回调
        .setup(move |app| {
            // 初始化数据库
            let app_dir = app.path().app_config_dir()?;
            std::fs::create_dir_all(&app_dir)
                .map_err(|e| errors::AppError::FileOperation(format!("创建配置目录失败: {}", e)))?;
            let db_path = app_dir.join("my_aur_helper.db"); // 数据库文件路径
            let database = db::Database::new(&db_path)
                .map_err(|e| errors::AppError::DatabaseError(format!("数据库初始化失败: {}", e)))?;
            database.initialize().map_err(|e| {
                errors::AppError::DatabaseError(format!("数据库表结构初始化失败: {}", e))
            })?;

            // 读取日志设置并初始化日志轮转系统
            let log_dir_raw = get_setting_string(&database, "log_dir", "");
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
            let log_prefix = get_setting_string(&database, "log_prefix", "applog");
            let log_max_size: u64 = get_setting_string(&database, "log_max_size", "10485760")
                .parse()
                .unwrap_or(10485760);
            let log_max_files: usize = get_setting_string(&database, "log_max_files", "7")
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

            // 读取系统托盘设置
            let show_tray = get_setting_string(&database, "show_tray_icon", "true") == "true";
            let close_action = get_setting_string(&database, "close_action", "minimize_to_tray");
            log::info!(
                "配置: show_tray_icon={}, close_action={}",
                show_tray,
                close_action
            );

            // 存储窗口关闭动作配置
            app.manage(CloseAction(close_action));

            // 如果启用，创建系统托盘
            tray::create_tray(app, show_tray)?;

            // 初始化内存缓存：读取配置 → 尝试从磁盘加载未过期条目
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

            // 注册定时写盘任务（写入周期 > 0 且缓存启用时）
            let write_interval = cache_config.write_interval_secs;
            if cache_config.enabled && write_interval > 0 {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let mut ticker =
                        tokio::time::interval(std::time::Duration::from_secs(write_interval));
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

            Ok(())
        })
        // 窗口关闭事件处理
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // 获取关闭动作配置
                let close_action = window
                    .try_state::<CloseAction>()
                    .map(|s| s.0.clone())
                    .unwrap_or_else(|| "minimize_to_tray".to_string());

                if close_action == "minimize_to_tray" {
                    // 隐藏窗口到系统托盘，而不是关闭应用
                    if let Err(e) = window.hide() {
                        log::warn!("窗口隐藏失败: {}", e);
                    }
                    api.prevent_close(); // 阻止窗口关闭
                }
                // 否则：默认行为，关闭窗口并退出应用
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::software::list_software,
            commands::software::list_software_view,
            commands::software::get_software_list_entry,
            commands::software::get_software,
            commands::software::get_software_detail,
            commands::software::get_prev_next_software,
            commands::software::search_software,
            commands::software::add_software,
            commands::software::update_software,
            commands::software::delete_software,
            commands::software::batch_delete_software,
            commands::software::set_software_license,
            commands::software::set_software_language,
            commands::sysops::software_sync::aur::sync_from_aur,
            commands::sysops::software_sync::aur::update_aur_info,
            commands::sysops::software_sync::import_aur::search_aur_packages,
            commands::sysops::software_sync::import_aur::import_aur_package,
            commands::sysops::software_sync::pkgbuild::sync_from_pkgbuild,
            commands::sysops::software_sync::upstream::check_all_upstream,
            commands::sysops::software_check_single::check_upstream_version,
            commands::sysops::software_check::selected::check_selected_upstream,
            commands::sysops::upstream_validate::validate_upstream_urls,
            commands::fileops::scan::scan_pkg_files_cmd,
            commands::fileops::cache_scan::list_cache_software,
            commands::fileops::cache_scan::scan_all_cache_dirs,
            commands::fileops::cache_scan::clear_cache_software,
            commands::fileops::cache_backup::backup_cache_to_existing,
            commands::fileops::cache_backup::backup_cache_to_subdirectory,
            commands::fileops::backup_scan::scan_backup_directory,
            commands::fileops::backup_scan::list_backup_subdirectories,
            commands::fileops::backup_dedup::deduplicate_backups,
            commands::sysops::backup_basic::list_backup_software,
            commands::sysops·backup_basic::clear_backup_software,
            commands::sysops::backup_basic::delete_backup,
            commands::sysops::backup_install::get_package_file_info,
            commands::sysops::backup_install::check_sudoers_config,
            commands::sysops::backup_install::get_sudoers_command,
            commands::sysops::backup_install::install_backup_package,
            commands::proxy::get_proxies,
            commands::proxy::fetch_proxy_sources,
            commands::proxy::download_proxy_file,
            commands::proxy::parse_proxy_file,
            commands::proxy::test_proxy,
            commands::proxy::test_proxies_batch,
            commands::proxy::test_proxy_single,
            commands::proxy::set_proxy_active,
            commands::proxy::update_proxy,
            commands::proxy::delete_proxy,
            commands::proxy::clear_proxy_tables,
            commands::sysops::sys_command::get_package_version,
            commands::sysops::sys_command::list_installed_packages,
            commands::sysops::cache_cleanup::clean_system_cache,
            commands::sysops::cache_cleanup::clean_custom_cache_dirs,
            commands::sysops::cache_cleanup::check_cache_cleanup_sudoers,
            commands::sysops::cache_cleanup::get_cache_cleanup_sudoers_command,
            commands::sysops::cache_cleanup::clear_github_tag_cache,
            commands::sysops::cache_cleanup::clear_expired_github_tag_cache,
            commands::sysops::cache_cleanup::get_github_tag_cache_stats,
            commands::sysops::cache_install::get_cache_package_info,
            commands::sysops::cache_install::install_cache_package,
            commands::sysops::cache_install::check_cache_install_sudoers,
            commands::sysops::cache_install::get_cache_install_sudoers_command,
            commands::logs::get_logs,
            commands::logs::get_new_logs,
            commands::logs::clear_logs,
            commands::settings::get_settings,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::apply_log_settings,
            commands::memory_cache::get_memory_cache_stats,
            commands::memory_cache::flush_memory_cache,
            commands::memory_cache::clear_memory_cache,
            commands::enums::get_licenses,
            commands::enums::sync_licenses_from_spdx,
            commands::enums::add_license,
            commands::enums::get_languages,
            commands::enums::upsert_language,
            commands::enums::delete_language,
            commands::dashboard::get_dashboard_stats,
            commands::fe_log::frontend_log,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // 退出前将内存缓存写盘（托盘「退出」/窗口关闭退出均触发 RunEvent::Exit）
            if let tauri::RunEvent::Exit = event {
                if let Some(state) = app_handle.try_state::<AppState>() {
                    if let Ok(mut cache) = state.memory_cache.lock() {
                        match cache.flush() {
                            Ok(n) => log::info!("退出前内存缓存已写盘: {} 个域", n),
                            Err(e) => log::warn!("退出前内存缓存写盘失败: {}", e),
                        }
                    }
                }
            }
        });
}
