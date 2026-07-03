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
pub mod aur;        // AUR RPC API 交互模块
pub mod backup;     // 备份管理模块
pub mod checkers;   // 版本检查器模块
pub mod commands;   // Tauri IPC 命令模块
pub mod db;         // 数据库操作模块
pub mod logger;     // 日志宏模块
pub mod models;     // 数据模型模块
pub mod proxy;      // 代理管理模块

use std::path::PathBuf;       // 路径缓冲区，用于构建文件路径
use std::sync::Mutex;         // 互斥锁，保证数据库连接的线程安全访问
use tauri::{
    menu::{Menu, MenuItem},   // Tauri 菜单组件
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, // 系统托盘相关
    Manager,                  // Tauri 应用管理器 trait
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
    db.get_setting(key)       // 从数据库查询设置
        .ok()                 // 将 Result 转为 Option
        .flatten()            // 展开 Option<Option<Setting>>
        .map(|s| s.value)     // 提取 Setting 的值字段
        .unwrap_or_else(|| default.to_string()) // 不存在则返回默认值
}

/// 运行 Tauri 应用
pub fn run() {
    // 初始化日志目录
    let config_dir = get_config_dir();
    let logs_dir = config_dir.join("logs");
    std::fs::create_dir_all(&logs_dir).ok();

    tauri::Builder::default()
        // 配置日志插件
        .plugin(
            tauri_plugin_log::Builder::new()
                .clear_targets()  // 清除默认日志目标
                .targets([
                    // 输出到标准输出
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    // 输出到日志文件
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                        path: logs_dir,
                        file_name: Some("my_aur_helper".to_string()),
                    }),
                ])
                .level(tauri_plugin_log::log::LevelFilter::Debug) // 设置日志级别为 Debug
                // 自定义日志格式：时间 - 级别: [模块] 消息
                .format(|out, message, record| {
                    let target = record.target(); // 获取日志来源模块名
                    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"); // 格式化当前时间
                    // 将日志级别映射为固定长度的字符串
                    let level = match record.level() {
                        log::Level::Error => "ERROR",
                        log::Level::Warn => "WARN",
                        log::Level::Info => "INFO",
                        log::Level::Debug => "DEBUG",
                        log::Level::Trace => "TRACE",
                    };
                    out.finish(format_args!("{} - {}: [{}] {}", now, level, target, message))
                })
                .build(),
        )
        // 配置 Shell 插件，用于执行系统命令
        .plugin(tauri_plugin_shell::init())
        // 应用初始化回调
        .setup(|app| {
            log::info!("Application starting");

            // 初始化数据库
            let app_dir = app.path().app_config_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("my_aur_helper.db"); // 数据库文件路径
            let database = db::Database::new(&db_path)?;    // 创建数据库连接
            database.initialize()?;                          // 初始化表结构和默认数据

            // 读取系统托盘设置
            let show_tray = get_setting_string(&database, "show_tray_icon", "true") == "true";
            let close_action = get_setting_string(&database, "close_action", "minimize_to_tray");
            log::info!("Settings: show_tray_icon={}, close_action={}", show_tray, close_action);

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
                    .icon(app.default_window_icon().unwrap().clone()) // 使用应用默认图标
                    .menu(&menu)                                      // 绑定菜单
                    .tooltip("My AUR Helper")                         // 鼠标悬停提示
                    // 菜单事件处理
                    .on_menu_event(move |app, event| {
                        match event.id.as_ref() {
                            "show" => {
                                // 显示主窗口并获取焦点
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
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
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;

                log::info!("System tray created");
            } else {
                log::info!("System tray disabled by settings");
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
                    window.hide().unwrap();
                    api.prevent_close(); // 阻止窗口关闭
                }
                // 否则：默认行为，关闭窗口并退出应用
            }
        })
        // 注册所有 Tauri 命令
        .invoke_handler(tauri::generate_handler![
            // 软件包管理
            commands::software::list_software,           // 获取所有软件包列表
            commands::software::get_software,            // 根据包名获取单个软件包
            commands::software::search_software,         // 搜索软件包
            commands::software::add_software,            // 添加新的软件包
            commands::software::delete_software,         // 删除软件包
            commands::software::set_software_license,    // 设置软件包的 License
            commands::software::set_software_language,   // 设置软件包的编程语言
            commands::software::check_upstream_version,  // 检查单个软件包的上游版本
            commands::software::check_all_upstream,      // 检查所有软件包的上游版本
            // 扫描
            commands::scan::scan_directory,              // 扫描指定目录（单层）
            commands::scan::scan_directory_recursive,    // 递归扫描目录树
            commands::scan::scan_pkg_files_cmd,          // 扫描 .pkg.tar.zst 包文件
            // 备份管理
            commands::backup::run_backup,                // 执行备份操作
            // 代理管理
            commands::proxy::get_proxies,                // 获取所有代理列表
            commands::proxy::fetch_proxy_sources,        // 从 Greasyfork 获取代理源
            commands::proxy::test_proxy,                 // 测试代理延迟
            commands::proxy::set_proxy_active,           // 设置代理启用状态
            // 文件操作
            commands::files::copy_file,                  // 复制文件或目录
            commands::files::move_file,                  // 移动文件或目录
            commands::files::delete_file,                // 删除文件或目录
            commands::files::delete_directory,           // 删除目录
            commands::files::create_directory,           // 创建目录
            commands::files::read_file,                  // 读取文件内容
            commands::files::list_directory,             // 列出目录内容
            commands::files::file_exists,                // 检查文件是否存在
            commands::files::file_metadata,              // 获取文件元信息
            commands::files::scan_pkg_files,             // 扫描 .pkg.tar 文件
            commands::files::batch_delete,               // 批量删除文件
            // 系统命令
            commands::sys_command::run_command,          // 执行任意系统命令
            commands::sys_command::install_package,      // 安装软件包
            commands::sys_command::remove_package,       // 卸载软件包
            commands::sys_command::clean_cache,          // 清理 pacman 缓存
            commands::sys_command::get_package_version,  // 获取已安装包的版本
            commands::sys_command::list_installed_packages, // 列出所有已安装包
            commands::sys_command::sync_database,        // 同步 pacman 数据库
            commands::sys_command::makepkg,              // 运行 makepkg 构建
            // 日志管理
            commands::logs::get_logs,                    // 获取日志列表
            commands::logs::clear_logs,                  // 清空日志
            // 设置管理
            commands::settings::get_settings,            // 获取所有设置
            commands::settings::get_setting,             // 获取单个设置
            commands::settings::set_setting,             // 设置配置值
            // 枚举值管理
            commands::enums::get_licenses,               // 获取所有 License
            commands::enums::sync_licenses_from_spdx,    // 从 SPDX 同步 License
            commands::enums::add_license,                // 添加 License
            commands::enums::get_languages,              // 获取所有编程语言
            commands::enums::upsert_language,            // 添加或更新编程语言
            commands::enums::delete_language,            // 删除编程语言
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
