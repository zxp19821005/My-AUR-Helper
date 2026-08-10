/**
 * proxy.rs - 代理管理命令
 *
 * 提供代理源的获取、下载、解析、测试和启用/禁用管理功能
 */
use log::{debug, info};
use tauri::State;
use chrono::Utc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::errors::AppResult;
use crate::models::*;
use crate::proxy;
use crate::AppState;

/// 批量代理测试的最大并发数
/// 配合 test.rs 中 10s 请求超时：既避免 42 个请求瞬时打满连接池，
/// 又防止个别慢代理拖垮整批（最坏耗时从「42×超时」降到「42/8×超时」）。
const MAX_PROXY_TEST_CONCURRENCY: usize = 8;

/// 代理测试结果
#[derive(serde::Serialize, Clone)]
pub struct ProxyTestResult {
    pub proxy_id: i64,
    pub success: bool,
    pub latency: Option<i64>,
    pub error: Option<String>,
    pub test_url: String,
}

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

/// 测试代理延迟（按下载代理类型，使用真实的下载地址拼接规则）
/// 注意：不记录代理 URL，防止凭据泄露
    #[tauri::command]
pub async fn test_proxy(_state: State<'_, AppState>, proxy_url: String) -> AppResult<i64> {
    debug!("正在测试代理延迟");
    let latency = proxy::test_proxy_by_type(
        0,
        &proxy::extract_proxy_name(&proxy_url),
        &proxy_url,
        &ProxyType::Download,
        None,
        false, // 单 URL 测试为遗留入口，默认保留目标协议头
    )
    .await?;
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
            all_proxies
                .into_iter()
                .filter(|p| ids.contains(&p.proxy_id.unwrap_or(0)))
                .collect()
        } else {
            // 测试所有代理
            db.get_all_proxies()?
        };
        let test_urls = get_test_urls(&db);
        (proxies, test_urls)
    };

    // 有界并发测试代理：用信号量限制同时运行的任务数为 MAX_PROXY_TEST_CONCURRENCY。
    // 信号量 permit 在任务内持有，任务结束（完成或失败）才释放，从而保证并发上限。
    let sem = std::sync::Arc::new(Semaphore::new(MAX_PROXY_TEST_CONCURRENCY));
    let mut set = JoinSet::new();

    for proxy in proxies {
        // 拿到 permit 才 spawn，等价于「最多 8 个在飞」，避免一次性 spawn 42 个任务
        let permit = sem
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| crate::errors::AppError::NetworkError(format!("并发控制失败: {}", e)))?;
        let test_urls = test_urls.clone();
        set.spawn(async move {
            let _permit = permit; // 持有至任务结束，维持并发上限
            let proxy_id = proxy.proxy_id.unwrap_or(0);
            let test_url = get_test_url_for_type(&proxy.proxy_type, &test_urls);

            match proxy::test_proxy_by_type(
                proxy_id,
                &proxy.proxy_name,
                &proxy.url,
                &proxy.proxy_type,
                Some(&test_url),
                proxy.strip_target_protocol,
            )
            .await
            {
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
            }
        });
    }

    // 按完成顺序收集结果（完成顺序 ≠ 发起顺序，但前端按 proxy_id 映射，无影响）
    let mut results = Vec::new();
    while let Some(joined) = set.join_next().await {
        let result = joined.map_err(|e| {
            crate::errors::AppError::NetworkError(format!("代理测试任务异常: {}", e))
        })?;
        results.push(result);
    }

    info!("批量测试完成，共 {} 个代理", results.len());

    // 持久化测试结果到数据库
    {
        let db = state.db.lock()?;
        for result in &results {
            persist_test_result(&db, result)?;
        }
    }

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
        let proxy = proxies
            .into_iter()
            .find(|p| p.proxy_id == Some(proxy_id))
            .ok_or_else(|| {
                crate::errors::AppError::PackageNotFound(format!("代理 {} 不存在", proxy_id))
            })?;
        let test_urls = get_test_urls(&db);
        (proxy, test_urls)
    };

    let test_url = get_test_url_for_type(&proxy.proxy_type, &test_urls);

    // 测试代理
    let result = match proxy::test_proxy_by_type(
        proxy_id,
        &proxy.proxy_name,
        &proxy.url,
        &proxy.proxy_type,
        Some(&test_url),
        proxy.strip_target_protocol,
    )
    .await
    {
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

    // 持久化测试结果
    {
        let db = state.db.lock()?;
        persist_test_result(&db, &result)?;
    }

    Ok(result)
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
    info!("正在更新代理 {}: name={}, url={}, type={}", proxy_id, proxy_name, url, proxy_type);
    let db = state.db.lock()?;
    db.update_proxy(proxy_id, &proxy_name, &url, &proxy_type)?;
    info!("代理 {} 更新完成", proxy_id);
    Ok(())
}

/// 持久化单个测试结果到 proxies_test 表
fn persist_test_result(db: &crate::db::Database, result: &ProxyTestResult) -> AppResult<()> {
    let success_count = if result.success { 1 } else { 0 };
    let fail_count = if result.success { 0 } else { 1 };
    let test_record = ProxyTest {
        id: None,
        proxy_id: result.proxy_id,
        test_time: Some(Utc::now().naive_utc()),
        avg_latency: result.latency,
        success_count,
        fail_count,
        last_test_status: Some(if result.success {
            "success".to_string()
        } else {
            "fail".to_string()
        }),
    };
    db.insert_or_update_proxy_test(&test_record)?;
    Ok(())
}

/// 测试 URL 配置
#[derive(Clone)]
struct TestUrls {
    download: String,
    clone: String,
    raw: String,
    ssh: String,
}

/// 获取测试 URL 配置
fn get_test_urls(db: &crate::db::Database) -> TestUrls {
    TestUrls {
        download: db
            .get_setting("proxy_test_download_url")
            .unwrap_or(None)
            .map(|s| s.value)
            .unwrap_or_else(|| {
                "https://github.com/zxp19821005/My_AUR_Files/releases/latest/download/README.md"
                    .to_string()
            }),
        clone: db
            .get_setting("proxy_test_clone_url")
            .unwrap_or(None)
            .map(|s| s.value)
            .unwrap_or_else(|| "https://github.com/zxp19821005/My_AUR_Files.git".to_string()),
        raw: db
            .get_setting("proxy_test_raw_url")
            .unwrap_or(None)
            .map(|s| s.value)
            .unwrap_or_else(|| {
                "https://raw.githubusercontent.com/zxp19821005/My_AUR_Files/main/README.md"
                    .to_string()
            }),
        ssh: db
            .get_setting("proxy_test_ssh_url")
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
