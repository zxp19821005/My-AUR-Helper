/**
 * proxy.rs - 代理管理命令
 *
 * 提供代理源的获取、测试和启用/禁用管理功能
 */
use log::{debug, info};
use tauri::State;

use crate::errors::AppResult;
use crate::models::*;
use crate::proxy;
use crate::AppState;

/// 获取所有代理列表
#[tauri::command]
pub async fn get_proxies(state: State<'_, AppState>) -> AppResult<Vec<ProxyInfo>> {
    debug!("正在获取所有代理");
    let db = state.db.lock()?;
    let result = db.get_all_proxies()?;
    info!("已获取 {} 个代理", result.len());
    Ok(result)
}

/// 从 Greasyfork 用户脚本获取代理源
#[tauri::command]
pub async fn fetch_proxy_sources(state: State<'_, AppState>) -> AppResult<usize> {
    info!("正在从用户脚本获取代理源");
    let client = reqwest::Client::new();
    let proxies = proxy::fetch_proxy_list_from_userscript(&client).await?;
    let db = state.db.lock()?;
    let mut count = 0;
    for p in proxies {
        let proxy_info = ProxyInfo {
            proxy_id: None,
            proxy_name: p.url.clone(),
            proxy_type: ProxyType::Download,
            url: p.url.clone(),
            is_active: true,
        };
        let _ = db.insert_proxy(&proxy_info);
        count += 1;
    }
    info!("已获取 {} 个代理源", count);
    Ok(count)
}

/// 测试代理延迟
/// 注意：不记录代理 URL，防止凭据泄露
#[tauri::command]
pub async fn test_proxy(_state: State<'_, AppState>, proxy_url: String) -> AppResult<i64> {
    debug!("正在测试代理延迟");
    let client = reqwest::Client::new();
    let latency = proxy::test_proxy_latency(&client, &proxy_url).await?;
    debug!("代理延迟: {}ms", latency);
    Ok(latency)
}

/// 设置代理启用状态
#[tauri::command]
pub async fn set_proxy_active(
    state: State<'_, AppState>,
    proxy_id: i64,
    is_active: bool,
) -> AppResult<()> {
    info!("正在设置代理 {} 启用状态={}", proxy_id, is_active);
    let db = state.db.lock()?;
    db.update_proxy_active(proxy_id, is_active)
}
