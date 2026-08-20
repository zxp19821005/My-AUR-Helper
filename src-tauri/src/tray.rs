/**
 * tray.rs - 系统托盘创建
 *
 * 根据配置是否启用，创建系统托盘图标与菜单，
 * 并处理菜单点击与托盘图标点击事件。
 */
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

/// 创建系统托盘（若启用）
/// @param app - Tauri 应用引用
/// @param show_tray - 是否启用系统托盘
pub fn create_tray(app: &tauri::App, show_tray: bool) -> tauri::Result<()> {
    if show_tray {
        // 创建托盘菜单项
        let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
        let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

        // 创建托盘图标
        let _tray = TrayIconBuilder::new()
            .icon(
                app.default_window_icon().cloned()
                    .unwrap_or_else(|| {
                        log::warn!("默认图标加载失败，使用系统默认图标");
                        // 创建一个 1x1 的透明图标作为备用
                        tauri::image::Image::new(&[0u8, 0, 0, 0], 1, 1)
                    }),
            )
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
    Ok(())
}
