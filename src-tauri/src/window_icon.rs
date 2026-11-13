/**
 * window_icon.rs - 运行时窗口图标设置
 *
 * Linux/X11 环境下，面板的任务栏按钮优先通过 .desktop 的 Icon= 或窗口的
 * _NET_WM_ICON 属性取图标。Tauri 会在创建窗口时注入 default_window_icon，
 * 但部分 GTK/WebKitGTK 组合下只落地为旧式 WM_HINTS 位图图标（单色）。
 * 此处在 setup 阶段对主窗口显式设置一次，补齐 ARGB 图标路径。
 */
use tauri::Manager;

/// 将应用默认图标显式写入主窗口
/// @param app - Tauri 应用引用
pub fn apply_window_icon(app: &tauri::App) {
    // 主窗口由 app.windows 配置创建，label 固定为 "main"
    let Some(window) = app.get_webview_window("main") else {
        log::warn!("未找到主窗口 main，跳过窗口图标设置");
        return;
    };

    // 图标由 tauri-codegen 在编译期从 bundle.icon 首个 PNG 解码为 RGBA 内嵌
    let Some(icon) = app.default_window_icon().cloned() else {
        log::warn!("默认窗口图标未加载，跳过窗口图标设置");
        return;
    };

    match window.set_icon(icon) {
        Ok(()) => log::info!("主窗口图标已设置（_NET_WM_ICON）"),
        Err(e) => log::warn!("主窗口图标设置失败: {}", e),
    }
}
