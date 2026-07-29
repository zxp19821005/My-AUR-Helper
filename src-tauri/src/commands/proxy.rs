/**
 * proxy.rs - 代理管理命令
 *
 * 提供代理源的获取、下载、解析、测试和启用/禁用管理功能
 */
use log::{debug, info};
use tauri::State;

use crate::errors::AppResult;
use crate::models::*;
use crate::proxy;
use crate::AppState;

/// 代理测试结果
#[derive(serde::Serialize, Clone)]
pub struct ProxyTestResult {
    pub proxy_id: i64,
    pub success: bool,
    pub latency: Option<i64>,
    pub error: Option<String>,
    pub test_url: String,
}

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

/// 批量测试代理
/// 测试指定的代理列表或所有代理
#[tauri::command]
pub async fn test_proxies_batch(
    state: State<'_, AppState>,
    proxy_ids: Option<Vec<i64>>,
) -> AppResult<Vec<ProxyTestResult>> {
    info!("开始批量测试代理");

    // 获取要测试的代理列表和测试 URL 设置
    let (proxies, test_urls) = {
        let db = state.db.lock()?;
        let proxies = if let Some(ids) = proxy_ids {
            // 测试指定的代理
            let all_proxies = db.get_all_proxies()?;
            all_proxies.into_iter()
                .filter(|p| ids.contains(&p.proxy_id.unwrap_or(0)))
                .collect()
        } else {
            // 测试所有代理
            db.get_all_proxies()?
        };
        let test_urls = get_test_urls(&db);
        (proxies, test_urls)
    };

    // 并发测试代理
    let client = reqwest::Client::new();
    let mut results = Vec::new();

    for proxy in proxies {
        let proxy_id = proxy.proxy_id.unwrap_or(0);
        let test_url = get_test_url_for_type(&proxy.proxy_type, &test_urls);

        let result = match proxy::test_proxy_by_type(
            &client,
            &proxy.url,
            &proxy.proxy_type,
            Some(&test_url),
        ).await {
            Ok(latency) => ProxyTestResult {
                proxy_id,
                success: true,
                latency: Some(latency),
                error: None,
                test_url,
            },
            Err(e) => ProxyTestResult {
                proxy_id,
                success: false,
                latency: None,
                error: Some(e.to_string()),
                test_url,
            },
        };

        results.push(result);
    }

    info!("批量测试完成，共 {} 个代理", results.len());
    Ok(results)
}

/// 单个测试代理
#[tauri::command]
pub async fn test_proxy_single(
    state: State<'_, AppState>,
    proxy_id: i64,
) -> AppResult<ProxyTestResult> {
    info!("测试代理 {}", proxy_id);

    // 获取代理信息和测试 URL 设置
    let (proxy, test_urls) = {
        let db = state.db.lock()?;
        let proxies = db.get_all_proxies()?;
        let proxy = proxies.into_iter()
            .find(|p| p.proxy_id == Some(proxy_id))
            .ok_or_else(|| crate::errors::AppError::PackageNotFound(format!("代理 {} 不存在", proxy_id)))?;
        let test_urls = get_test_urls(&db);
        (proxy, test_urls)
    };

    let test_url = get_test_url_for_type(&proxy.proxy_type, &test_urls);

    // 测试代理
    let client = reqwest::Client::new();
    let result = match proxy::test_proxy_by_type(
        &client,
        &proxy.url,
        &proxy.proxy_type,
        Some(&test_url),
    ).await {
        Ok(latency) => ProxyTestResult {
            proxy_id,
            success: true,
            latency: Some(latency),
            error: None,
            test_url,
        },
        Err(e) => ProxyTestResult {
            proxy_id,
            success: false,
            latency: None,
            error: Some(e.to_string()),
            test_url,
        },
    };

    info!("代理 {} 测试完成: {}", proxy_id, result.success);
    Ok(result)
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

/// 测试 URL 配置
struct TestUrls {
    download: String,
    clone: String,
    raw: String,
    ssh: String,
}

/// 获取测试 URL 配置
fn get_test_urls(db: &crate::db::Database) -> TestUrls {
    TestUrls {
        download: db.get_setting("proxy_test_download_url")
            .unwrap_or(None)
            .map(|s| s.value)
            .unwrap_or_else(|| "https://github.com/zxp19821005/My_AUR_Files/releases/latest/download/README.md".to_string()),
        clone: db.get_setting("proxy_test_clone_url")
            .unwrap_or(None)
            .map(|s| s.value)
            .unwrap_or_else(|| "https://github.com/zxp19821005/My_AUR_Files.git".to_string()),
        raw: db.get_setting("proxy_test_raw_url")
            .unwrap_or(None)
            .map(|s| s.value)
            .unwrap_or_else(|| "https://raw.githubusercontent.com/zxp19821005/My_AUR_Files/main/README.md".to_string()),
        ssh: db.get_setting("proxy_test_ssh_url")
            .unwrap_or(None)
            .map(|s| s.value)
            .unwrap_or_else(|| "ssh://git@ssh.github.com:443/zxp19821005/My_AUR_Files".to_string()),
    }
}

/// 根据代理类型获取测试 URL
fn get_test_url_for_type(proxy_type: &ProxyType, test_urls: &TestUrls) -> String {
    match proxy_type {
        ProxyType::Download => test_urls.download.clone(),
        ProxyType::Clone => test_urls.clone.clone(),
        ProxyType::Raw => test_urls.raw.clone(),
        ProxyType::Ssh => test_urls.ssh.clone(),
    }
}
