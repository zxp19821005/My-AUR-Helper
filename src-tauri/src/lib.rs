/**
 * lib.rs - Tauri 应用入口
 *
 * 功能：
 * - 声明模块并编排 Tauri Builder：插件 → setup 初始化 → 事件回调 → 命令路由 → 运行
 * - 具体职责已拆出：ipc/registry（命令注册与 bindings）、app_state（状态）、
 *   app_setup（数据库/日志/缓存初始化）、app_events（窗口关闭与退出写盘）
 *
 * 工作流程（setup 初始化顺序不得改变）：
 * DB → 日志 → 托盘 → 图标 → 缓存
 *
 * 适用场景：应用启动入口，仅保留编排骨架，业务细节在各子模块。
 */
pub mod app_events;
pub mod app_setup;
pub mod app_state;
pub mod aur;
pub mod cache;
pub mod checkers;
pub mod commands;
pub mod db;
pub mod errors;
pub mod http_client;
pub mod ipc;
pub mod logger;
pub mod models;
pub mod network;
pub mod proxy;
mod tray;
pub mod versions;
mod window_icon;

use tauri::Manager;

use app_state::{get_config_dir, get_setting_string, CloseAction};
use ipc::registry::build_specta_router;

// 保持对外路径 crate::AppState 不变（全仓命令均 use crate::AppState），调用方零改动
pub use app_state::AppState;

#[cfg(any(debug_assertions, test))]
use ipc::registry::export_bindings;

/// 运行 Tauri 应用
pub fn run() {
    let config_dir = get_config_dir();

    let specta_router = build_specta_router();
    #[cfg(debug_assertions)]
    export_bindings();

    tauri::Builder::default()
        // 配置 Shell 插件，用于执行系统命令
        .plugin(tauri_plugin_shell::init())
        // 应用初始化回调
        .setup(move |app| {
            // 1. 初始化数据库
            let database = app_setup::init_database(app)?;

            // 2. 读取日志设置并初始化日志轮转系统
            app_setup::init_logger(&database, app, &config_dir);

            // 3. 读取系统托盘设置（顺序：托盘 → 图标 → 缓存）
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

            // 4. 显式设置主窗口图标，补齐 X11 _NET_WM_ICON 路径
            window_icon::apply_window_icon(app);

            // 5. 初始化内存缓存（内部读取配置 → 加载磁盘 → app.manage(AppState)）
            let cache_config = app_setup::init_memory_cache(database, app);

            // 注册定时写盘任务（写入周期 > 0 且缓存启用时）
            app_setup::spawn_cache_flush_task(app, &cache_config);

            Ok(())
        })
        // 窗口关闭事件处理
        .on_window_event(app_events::handle_window_event)
        .invoke_handler(specta_router.invoke_handler())
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        // 退出事件：写盘内存缓存
        .run(app_events::handle_run_event);
}

#[cfg(test)]
mod specta_bindings_tests {
    /// 手动刷新 bindings：cargo test --lib export_bindings
    /// debug 构建启动时会自动导出，此测试供 release 前或不启动应用时刷新使用。
    #[test]
    fn export_bindings() {
        super::export_bindings();
    }
}
