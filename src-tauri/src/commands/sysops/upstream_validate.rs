/**
 * upstream_validate.rs - 上游 URL 验证命令
 *
 * 功能：
 * - 批量验证软件包的上游 URL 可达性
 * - 支持并发验证（并发数: 10）
 * - 通过 Tauri 窗口事件报告验证进度
 * - 更新 upstream_info 表中的 upstream_url_status 字段
 */
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::Semaphore;

use crate::errors::AppResult;
use crate::models::UpstreamUrlStatus;
use crate::AppState;

/// 验证结果结构
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidateResult {
    pub software_id: i64,
    pub pkgname: String,
    pub upstream_url: Option<String>,
    pub status: UpstreamUrlStatus,
}

/// 验证单个软件包的上游 URL
/// @param client - HTTP 客户端
/// @param software_id - 软件 ID
/// @param pkgname - 包名
/// @param upstream_url - 上游 URL
/// @returns 验证结果
async fn validate_single_url(
    client: &reqwest::Client,
    software_id: i64,
    pkgname: &str,
    upstream_url: &str,
) -> ValidateResult {
    let status = match client.get(upstream_url).send().await {
        Ok(resp) => {
            let status_code = resp.status().as_u16();
            if status_code == 200 {
                UpstreamUrlStatus::Ok
            } else if status_code == 404 {
                UpstreamUrlStatus::NotFound
            } else if status_code == 403 || status_code == 401 {
                UpstreamUrlStatus::Forbidden
            } else if status_code >= 300 && status_code < 400 {
                UpstreamUrlStatus::Redirected
            } else if status_code >= 500 {
                UpstreamUrlStatus::ServerError
            } else {
                UpstreamUrlStatus::OtherError
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("timeout") {
                UpstreamUrlStatus::Timeout
            } else if error_str.contains("connection")
                || error_str.contains("connect")
                || error_str.contains("dns")
            {
                UpstreamUrlStatus::ConnectionError
            } else {
                UpstreamUrlStatus::OtherError
            }
        }
    };

    ValidateResult {
        software_id,
        pkgname: pkgname.to_string(),
        upstream_url: Some(upstream_url.to_string()),
        status,
    }
}

/// 批量验证上游 URL
///
/// 验证软件包的上游 URL 可达性，支持以下参数：
/// - pkgname_list: 指定要验证的包名列表；为空时验证所有有上游 URL 的包
/// - 通过窗口事件 "validate-upstream-progress" 报告进度
/// - 并发数: 10，超时时间: 10 秒
#[tauri::command]
pub async fn validate_upstream_urls(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    pkgname_list: Option<Vec<String>>,
) -> AppResult<Vec<ValidateResult>> {
    log::info!("[上游URL验证] 开始批量验证");

    // 获取需要验证的条目列表（在锁内完成查询，立即释放锁）
    let entries_to_validate = {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;

        let all_entries = db.get_software_list_entries()?;
        if let Some(ref pkgname_list) = pkgname_list {
            all_entries
                .into_iter()
                .filter(|e| pkgname_list.contains(&e.pkgname))
                .collect::<Vec<_>>()
        } else {
            all_entries
                .into_iter()
                .filter(|e| e.upstream_url.is_some())
                .collect::<Vec<_>>()
        }
    };

    let total = entries_to_validate.len();
    log::info!("[上游URL验证] 共 {} 个软件包需要验证", total);

    if total == 0 {
        return Ok(vec![]);
    }

    // 创建 HTTP 客户端（10 秒超时）
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| {
            crate::errors::AppError::NetworkError(format!("创建 HTTP 客户端失败: {}", e))
        })?;

    // 并发控制：最大 10 个并发请求
    let semaphore = Arc::new(Semaphore::new(10));
    let mut handles = Vec::new();

    for (idx, entry) in entries_to_validate.into_iter().enumerate() {
        if let Some(ref url) = entry.upstream_url {
            let client = client.clone();
            let sem = semaphore.clone();
            let app = app.clone();
            let url = url.clone();
            let software_id = entry.software_id;
            let pkgname = entry.pkgname.clone();
            let current_idx = idx + 1;

            let handle = tokio::spawn(async move {
                // 获取信号量许可（控制并发数）
                let _permit = sem.acquire().await.unwrap();

                // 报告进度
                let _ = app.emit(
                    "validate-upstream-progress",
                    serde_json::json!({
                        "current": current_idx,
                        "total": total,
                        "pkgname": pkgname,
                    }),
                );

                // 执行验证
                validate_single_url(&client, software_id, &pkgname, &url).await
            });

            handles.push(handle);
        }
    }

    // 等待所有验证完成
    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    // 批量更新数据库（在新锁内完成更新）
    {
        let db = state.db.lock().map_err(|e| {
            crate::errors::AppError::DatabaseError(format!("获取数据库锁失败: {}", e))
        })?;

        for result in &results {
            db.update_upstream_url_status(result.software_id, &result.status)?;
        }
    }

    log::info!("[上游URL验证] 完成，共验证 {} 个软件包", results.len());

    Ok(results)
}
