/**
 * proxy/basic.rs - 代理基础管理命令
 *
 * 提供代理源的获取、下载、解析与增删改查命令，
 * 不含连通性测试逻辑（测试相关见 test.rs）。
 */
use log::{debug, info};
use tauri::State;
use reqwest;

use crate::errors::AppResult;
use crate::models::*;
use crate::proxy;
use crate::AppState;

/// 获取所有代理列表（附带最新测试统计）
#[tauri::command]
pub async fn get_proxies(state: State<'_, AppState>) -> AppResult<Vec<ProxyInfo>> {
    debug!("正在获取所有代理");
    let db = state.db.lock()?;
    let result = db.get_all_proxies_with_stats()?;
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
            proxy_name: proxy::extract_proxy_name(&p.url),
            proxy_type: p.proxy_type,
            url: p.url.clone(),
            is_active: true,
            success_count: 0,
            fail_count: 0,
            avg_latency: None,
            last_test_status: None,
            strip_target_protocol: p.strip_target_protocol,
        };
        let _ = db.insert_proxy(&proxy_info);
        count += 1;
    }
    info!("已获取 {} 个代理源", count);
    Ok(count)
}

/// 下载代理文件
/// 从配置的 URL 下载代理规则 JS 文件到本地
#[tauri::command]
pub async fn download_proxy_file(state: State<'_, AppState>) -> AppResult<usize> {
    info!("开始下载代理文件");

    // 获取下载 URL
    let download_url = {
        let db = state.db.lock()?;
        db.get_setting("proxy_download_url")
            .unwrap_or(None)
            .map(|s| s.value)
            .unwrap_or_else(|| "https://update.greasyfork.org/scripts/412245/Github%20%E5%A2%9E%E5%BC%BA%20-%20%E9%AB%98%E9%80%9F%E4%B8%8B%E8%BD%BD.user.js".to_string())
    };

    let client = reqwest::Client::new();
    let file_path = proxy::download_proxy_file(&client, &download_url).await?;

    info!("代理文件已下载到: {:?}", file_path);
    Ok(0) // 返回 0，实际数量在解析时计算
}

/// 解析代理文件
/// 读取已下载的代理规则 JS 文件，解析代理信息并写入数据库
#[tauri::command]
pub async fn parse_proxy_file(state: State<'_, AppState>) -> AppResult<usize> {
    info!("开始解析代理文件");
    let proxies = proxy::parse_proxy_file().await?;

    let db = state.db.lock()?;
    let mut count = 0;
    for p in proxies {
        if let Ok(_) = db.insert_proxy(&p) {
            count += 1;
        }
    }

    info!("成功解析并插入 {} 个代理", count);
    Ok(count)
}

/// 清空代理表
/// 删除 proxies_info 和 proxies_test 表中所有数据，并重置 proxy_id 自增计数器
#[tauri::command]
pub async fn clear_proxy_tables(state: State<'_, AppState>) -> AppResult<usize> {
    info!("正在清空代理表");
    let db = state.db.lock()?;
    let count = db.clear_all_proxies()?;
    info!("已清空 {} 个代理记录", count);
    Ok(count)
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

/// 删除代理
#[tauri::command]
pub async fn delete_proxy(state: State<'_, AppState>, proxy_id: i64) -> AppResult<()> {
    info!("正在删除代理 {}", proxy_id);
    let db = state.db.lock()?;
    db.delete_proxy(proxy_id)
}

/// 更新代理信息（支持手动编辑名称、URL、类型）
#[tauri::command]
pub async fn update_proxy(
    state: State<'_, AppState>,
    proxy_id: i64,
    proxy_name: String,
    url: String,
    proxy_type: String,
) -> AppResult<()> {
    info!(
        "正在更新代理 {}: name={}, url={}, type={}",
        proxy_id, proxy_name, url, proxy_type
    );
    let db = state.db.lock()?;
    db.update_proxy(proxy_id, &proxy_name, &url, &proxy_type)?;
    info!("代理 {} 更新完成", proxy_id);
    Ok(())
}
