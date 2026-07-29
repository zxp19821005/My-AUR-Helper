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
pub mod aur; // AUR RPC API 交互模块
pub mod checkers; // 版本检查器模块
pub mod commands; // Tauri IPC 命令模块
pub mod db; // 数据库操作模块
pub mod errors; // 统一错误处理模块
pub mod logger; // 日志轮转与输出模块
pub mod models; // 数据模型模块
pub mod proxy; // 代理管理模块
pub mod versions; // 版本处理模块

use std::path::PathBuf; // 路径缓冲区，用于构建文件路径
use std::sync::Mutex; // 互斥锁，保证数据库连接的线程安全访问
use tauri::{
    menu::{Menu, MenuItem}, // Tauri 菜单组件
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, // 系统托盘相关
    Manager,                // Tauri 应用管理器 trait
};

/// 应用状态，包含数据库连接
pub struct AppState {
    /// 数据库连接（线程安全）
    pub db: Mutex<db::Database>,
}

/// 窗口关闭动作配置
struct CloseAction(String);

/// 获取配置目录路径
/// 优先使用系统配置目录，失败时使用当前目录
/// @returns 配置目录的 PathBuf
fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.zxp19821005.aur-helper")
}

/// 从数据库获取设置值
/// @param db - 数据库连接引用
/// @param key - 设置键名
/// @param default - 默认值，当数据库中不存在该键时返回
/// @returns 设置值的字符串
fn get_setting_string(db: &db::Database, key: &str, default: &str) -> String {
    db.get_setting(key) // 从数据库查询设置
        .ok() // 将 Result 转为 Option
        .flatten() // 展开 Option<Option<Setting>>
        .map(|s| s.value) // 提取 Setting 的值字段
        .unwrap_or_else(|| default.to_string()) // 不存在则返回默认值
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
            let database = db::Database::new(&db_path).map_err(|e| {
                errors::AppError::DatabaseError(format!("数据库初始化失败: {}", e))
            })?;
            database.initialize().map_err(|e| {
                errors::AppError::DatabaseError(format!("数据库表结构初始化失败: {}", e))
            })?;

            // 读取日志设置并初始化日志轮转系统
            let logs_dir = config_dir.join("logs");
            let log_max_size: u64 = get_setting_string(&database, "log_max_size", "10485760")
                .parse()
                .unwrap_or(10485760);
            let log_max_files: usize = get_setting_string(&database, "log_max_files", "7")
                .parse()
                .unwrap_or(7);
            logger::update_log_settings(log_max_size, log_max_files);
            let rotating_logger = logger::RotatingLogger::new(logs_dir, "applog".to_string());
            rotating_logger.init().expect("初始化日志记录器失败");
            log::info!(
                "日志系统已初始化，最大大小: {}KB, 最大文件数: {}",
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
            if show_tray {
                // 创建托盘菜单项
                let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
                let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

                // 创建托盘图标
                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon()
                        .map(|icon| icon.clone())
                        .unwrap_or_else(|| {
                            log::warn!("默认图标加载失败，使用系统默认图标");
                            // 创建一个 1x1 的透明图标作为备用
                            tauri::image::Image::new(&[0u8, 0, 0, 0], 1, 1)
                        }))
                    .menu(&menu) // 绑定菜单
                    .tooltip("My AUR Helper") // 鼠标悬停提示
                    // 菜单事件处理
                    .on_menu_event(move |app, event| {
                        match event.id.as_ref() {
                            "show" => {
                                // 显示主窗口并获取焦点
                                if let Some(window) = app.get_webview_window("main") {
                                    if let Err(e) = window.show() {
                                        log::warn!("窗口显示失败: {}", e);
                                    }
                                    if let Err(e) = window.set_focus() {
                                        log::warn!("窗口聚焦失败: {}", e);
                                    }
                                }
                            }
                            "quit" => {
                                // 退出应用
                                app.exit(0);
                            }
                            _ => {}
                        }
                    })
                    // 托盘图标点击事件（左键点击显示窗口）
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                if let Err(e) = window.show() {
                                    log::warn!("窗口显示失败: {}", e);
                                }
                                if let Err(e) = window.set_focus() {
                                    log::warn!("窗口聚焦失败: {}", e);
                                }
                            }
                        }
                    })
                    .build(app)?;

                log::info!("系统托盘已创建");
            } else {
                log::info!("系统托盘已被设置禁用");
            }

            // 将数据库存储到应用状态，供命令使用
            app.manage(AppState {
                db: Mutex::new(database),
            });

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
        // 注册所有 Tauri 命令
        .invoke_handler(tauri::generate_handler![
            // 软件包管理
            commands::software::list_software, // 获取所有软件包列表
            commands::software::list_software_view, // 获取软件包列表展示数据
            commands::software::get_software,  // 根据包名获取单个软件包
            commands::software::get_software_detail, // 获取软件包完整详情
            commands::software::get_prev_next_software, // 获取上一个/下一个软件包（导航用）
            commands::software::search_software, // 搜索软件包
            commands::software::add_software,  // 添加新的软件包
            commands::software::update_software, // 更新软件包信息
            commands::software::delete_software, // 删除软件包
            commands::software::batch_delete_software, // 批量删除软件包
            commands::software::set_software_license, // 设置软件包的 License
            commands::software::set_software_language, // 设置软件包的编程语言
            // 软件包同步（sysops 模块）
            commands::sysops::software_sync::aur::sync_from_aur, // 从 AUR 同步软件包
            commands::sysops::software_sync::aur::update_aur_info, // 更新 AUR 信息
            commands::sysops::software_sync::pkgbuild::sync_from_pkgbuild, // 从 PKGBUILD 文件同步
            commands::sysops::software_sync::upstream::check_all_upstream, // 并行检查所有软件包的上游版本
            // 版本检查（sysops 模块）
            commands::sysops::software_check::check_upstream_version, // 检查单个软件包的上游版本
            commands::sysops::software_check::check_selected_upstream, // 检查选中的软件包上游版本
            // 上游 URL 验证（sysops 模块）
            commands::sysops::upstream_validate::validate_upstream_urls, // 批量验证上游 URL
            // 扫描和缓存管理（fileops 模块）
            commands::fileops::scan::scan_pkg_files_cmd, // 扫描 .pkg.tar.zst 包文件
            commands::fileops::cache_scan::list_cache_software, // 直接读取 cache_software 表（页面打开时）
            commands::fileops::cache_scan::scan_all_cache_dirs, // 扫描所有启用的缓存目录
            commands::fileops::cache_scan::clear_cache_software, // 清空 cache_software 表
            commands::fileops::cache_backup::backup_cache_to_existing, // 备份缓存到已有备份位置
            commands::fileops::cache_backup::backup_cache_to_subdirectory, // 备份缓存到指定子目录
            // 备份管理（fileops 模块）
            commands::fileops::backup_scan::scan_backup_directory, // 扫描备份目录
            commands::fileops::backup_scan::list_backup_subdirectories, // 获取子目录列表
            commands::fileops::backup_dedup::deduplicate_backups,  // 软件去重
            // 备份管理（sysops 模块 - 查询和安装）
            commands::sysops::backup_basic::list_backup_software, // 列出所有备份记录
            commands::sysops::backup_basic::clear_backup_software, // 清空备份表
            commands::sysops::backup_basic::delete_backup,        // 删除单个备份
            commands::sysops::backup_install::get_package_file_info, // 获取包文件信息
            commands::sysops::backup_install::check_sudoers_config, // 检测 sudoers 配置
            commands::sysops::backup_install::get_sudoers_command,  // 获取 sudoers 配置命令
            commands::sysops::backup_install::install_backup_package, // 安装备份包
            // 代理管理
            commands::proxy::get_proxies,         // 获取所有代理列表
            commands::proxy::fetch_proxy_sources, // 从 Greasyfork 获取代理源
            commands::proxy::download_proxy_file, // 下载代理文件
            commands::proxy::parse_proxy_file,    // 解析代理文件
            commands::proxy::test_proxy,          // 测试代理延迟
            commands::proxy::test_proxies_batch,  // 批量测试代理
            commands::proxy::test_proxy_single,   // 单个测试代理
            commands::proxy::set_proxy_active,    // 设置代理启用状态
            commands::proxy::delete_proxy,        // 删除代理
            // 系统命令（sysops 模块）
            commands::sysops::sys_command::get_package_version, // 获取已安装包的版本
            commands::sysops::sys_command::list_installed_packages, // 列出所有已安装包
            // 日志管理
            commands::logs::get_logs,   // 获取日志列表
            commands::logs::clear_logs, // 清空日志
            // 设置管理
            commands::settings::get_settings,       // 获取所有设置
            commands::settings::get_setting,        // 获取单个设置
            commands::settings::set_setting,        // 设置配置值
            commands::settings::apply_log_settings, // 应用日志轮转设置
            // 枚举值管理
            commands::enums::get_licenses, // 获取所有 License
            commands::enums::sync_licenses_from_spdx, // 从 SPDX 同步 License
            commands::enums::add_license,  // 添加 License
            commands::enums::get_languages, // 获取所有编程语言
            commands::enums::upsert_language, // 添加或更新编程语言
            commands::enums::delete_language, // 删除编程语言
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
