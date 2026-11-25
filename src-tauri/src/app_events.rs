/**
 * app_events.rs - 窗口关闭事件与退出前写盘事件处理
 *
 * 功能：
 * - handle_window_event：CloseRequested 时按 CloseAction 决定隐藏到托盘还是退出
 * - handle_run_event：RunEvent::Exit 时把内存缓存最后写盘一次
 *
 * 工作流程：
 * 1. 窗口请求关闭 → 读取 CloseAction（默认 minimize_to_tray）
 * 2. minimize_to_tray：隐藏窗口并 prevent_close；否则按默认行为关闭并退出
 * 3. 事件循环退出 → 取 AppState.memory_cache 并 flush，记录写盘域数量
 *
 * 适用场景：Tauri 事件回调，从 lib.rs 迁出以降低入口文件行数。
 */
use tauri::Manager;

use crate::app_state::{AppState, CloseAction};

/// 处理窗口事件：仅拦截 CloseRequested，按配置隐藏到托盘或允许关闭。
///
/// # 参数
/// - `window`: 触发事件的窗口
/// - `event`: 窗口事件（仅 CloseRequested 会被处理）
pub(crate) fn handle_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
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
}

/// 处理应用级事件：退出前将内存缓存写盘。
///
/// # 参数
/// - `app_handle`: 应用句柄，用于取 AppState
/// - `event`: 应用事件（仅 RunEvent::Exit 会被处理）
pub(crate) fn handle_run_event(app_handle: &tauri::AppHandle, event: tauri::RunEvent) {
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
}
