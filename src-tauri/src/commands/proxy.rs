/**
 * proxy.rs - 代理管理命令
 *
 * 提供代理源的获取、测试和启用/禁用管理功能
 */
use log::{info, debug};     // 日志记录
use tauri::State;           // Tauri 状态管理

use crate::models::*;       // 数据模型
use crate::proxy;           // 代理模块
use crate::AppState;        // 应用状态

/// 获取所有代理列表
/// @param state - Tauri 应用状态
/// @returns 所有代理信息列表
#[tauri::command]
pub async fn get_proxies(state: State<'_, AppState>) -> Result<Vec<ProxyInfo>, String> {
    debug!("正在获取所有代理");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result = db.get_all_proxies().map_err(|e| e.to_string())?;
    info!("已获取 {} 个代理", result.len());
    Ok(result)
}

/// 从 Greasyfork 用户脚本获取代理源
/// 解析 userscript 中的代理数组，存入数据库
/// @param state - Tauri 应用状态
/// @returns 获取并存入的代理数量
#[tauri::command]
pub async fn fetch_proxy_sources(state: State<'_, AppState>) -> Result<usize, String> {
    info!("正在从用户脚本获取代理源");
    let client = reqwest::Client::new();
    // 从 userscript 中提取代理列表
    let proxies = proxy::fetch_proxy_list_from_userscript(&client)
        .await
        .map_err(|e| format!("Fetch failed: {}", e))?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut count = 0;
    for p in proxies {
        let proxy_info = ProxyInfo {
            proxy_id: None,
            proxy_name: p.url.clone(),
            proxy_type: ProxyType::Download, // 默认设为下载代理
            url: p.url.clone(),
            is_active: true,                  // 默认为启用
        };
        let _ = db.insert_proxy(&proxy_info); // 忽略重复插入错误
        count += 1;
    }
    info!("已获取 {} 个代理源", count);
    Ok(count)
}

/// 测试代理延迟
/// @param _state - Tauri 应用状态（当前未使用）
/// @param proxy_url - 要测试的代理 URL
/// @returns 延迟毫秒数
#[tauri::command]
pub async fn test_proxy(_state: State<'_, AppState>, proxy_url: String) -> Result<i64, String> {
    info!("正在测试代理: {}", proxy_url);
    let client = reqwest::Client::new();
    let latency = proxy::test_proxy_latency(&client, &proxy_url)
        .await
        .map_err(|e| format!("测试失败: {}", e))?;
    info!("代理 {} 延迟: {}ms", proxy_url, latency);
    Ok(latency)
}

/// 设置代理启用状态
/// @param state - Tauri 应用状态
/// @param proxy_id - 代理 ID
/// @param is_active - 是否启用
#[tauri::command]
pub async fn set_proxy_active(
    state: State<'_, AppState>,
    proxy_id: i64,
    is_active: bool,
) -> Result<(), String> {
    info!("正在设置代理 {} 启用状态={}", proxy_id, is_active);
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_proxy_active(proxy_id, is_active)
        .map_err(|e| e.to_string())
}
