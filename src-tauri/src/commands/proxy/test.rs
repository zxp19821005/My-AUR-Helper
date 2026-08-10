/**
 * proxy/test.rs - 代理连通性测试命令与辅助
 *
 * 提供代理延迟测试的多个入口（单 URL / 批量 / 单代理），
 * 以及测试结果持久化与测试 URL 配置等辅助逻辑。
 */
use log::{debug, info};
use tauri::State;
use chrono::Utc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::errors::{AppError, AppResult};
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
            .map_err(|e| AppError::NetworkError(format!("并发控制失败: {}", e)))?;
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
        let result = joined.map_err(|e| AppError::NetworkError(format!("代理测试任务异常: {}", e)))?;
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
            .ok_or_else(|| AppError::PackageNotFound(format!("代理 {} 不存在", proxy_id)))?;
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
