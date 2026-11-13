//! rpc_batch/mod.rs - AUR RPC 批量 info 查询子模块
//!
//! 职责：将包名分批通过 AUR RPC `info` 接口查询，识别反爬中间件拦截，
//! 汇总各批次成功结果与失败原因，供上游版本同步使用。
use crate::aur::rpc::AUR_RPC_URL;
use crate::errors::AppResult;
use log::{debug, info, warn};
use reqwest::Client;

/// 判定一次响应是否被反爬中间件（如 Anubis）拦截
///
/// 拦截的特征是：服务端返回 `HTTP 200`，但 `content-type` 不是 JSON，
/// 并且响应体是 HTML（trim 后以 `<` 开头）。
///
/// # 参数
/// * `content_type` - 响应的 `content-type` 头（缺失时传空串）
/// * `text` - 响应体文本
///
/// # 返回
/// - `true`: 响应不是 JSON，视为被拦截
/// - `false`: 响应是 JSON，正常
fn looks_like_intercepted(content_type: &str, text: &str) -> bool {
    let content_type_is_json = content_type.to_ascii_lowercase().contains("json");
    let body_is_html = text.trim_start().starts_with('<');
    !content_type_is_json || body_is_html
}

/// 取响应体的安全预览（用于日志，避免超长或破坏 UTF-8 边界）
///
/// # 参数
/// * `text` - 响应体文本
/// * `max_chars` - 最大字符数
///
/// # 返回
/// 截断后的响应体预览
fn preview(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

/// 将包名列表编码为 AUR RPC 所需的 `application/x-www-form-urlencoded` 请求体
///
/// 使用标准 form 编码而非字符串裸拼接：包名中的 `+` 必须转义为 `%2B`，
/// 否则会被服务端按 form 规则解码成空格，导致 `gtk+` / `c++` / `libsigc++`
/// 这类包永远查不到。
///
/// 注意：`form_urlencoded::Serializer` 不是 `Send`，因此必须放在同步函数中，
/// 不能让它跨越 `await` 点，否则 `get_packages_info` 的 future 将不满足 `Send`。
///
/// # 参数
/// * `chunk` - 单批包名列表
///
/// # 返回
/// 编码后的请求体字符串（重复 key `arg[]` 依次追加）
fn encode_info_body(chunk: &[String]) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for name in chunk {
        serializer.append_pair("arg[]", name);
    }
    serializer.finish()
}

/// 批量获取多个包的 AUR 信息（带批次间间隔，失败时不重试整个批次）
///
/// # 参数
/// - `client`: 已配置 User-Agent 的 HTTP 客户端
/// - `pkgnames`: 待查询的包名列表
/// - `batch_size`: 单批最大包数
/// - `batch_interval`: 批次之间的等待秒数
///
/// # 返回
/// - `AppResult<Vec<serde_json::Value>>`: 所有批次成功返回的结果合并
pub async fn get_packages_info(
    client: &Client,
    pkgnames: &[String],
    batch_size: usize,
    batch_interval: u64,
) -> AppResult<Vec<serde_json::Value>> {
    if pkgnames.is_empty() {
        return Ok(Vec::new());
    }

    let mut all_results = Vec::new();
    let chunks: Vec<&[String]> = pkgnames.chunks(batch_size).collect();
    let total_chunks = chunks.len();

    // 收集失败原因摘要，便于区分"全挂"与"确实没更新"
    let mut succeeded_batches = 0usize;
    let mut failure_reasons: std::collections::BTreeMap<&'static str, usize> =
        std::collections::BTreeMap::new();

    for (i, chunk) in chunks.iter().enumerate() {
        if i > 0 {
            info!(
                "[AUR 批量查询] 等待 {} 秒后继续下一批 ({}/{})",
                batch_interval,
                i + 1,
                total_chunks
            );
            tokio::time::sleep(std::time::Duration::from_secs(batch_interval)).await;
        }

        let url = format!("{}/info", AUR_RPC_URL);

        // 使用标准 form 编码：包名中的 `+` 必须转义为 `%2B`，
        // 否则会被服务端 form 解码成空格（ctk+/gtk+/libsigc++ 等包永远查不到）
        let body = encode_info_body(chunk);
        let body_len = body.len();

        info!(
            "[AUR 批量查询] 第 {}/{} 批：查询 {} 个包",
            i + 1,
            total_chunks,
            chunk.len()
        );
        debug!("[AUR 批量查询] 请求 URL: {}", url);
        debug!("[AUR 批量查询] 请求体长度: {} 字节", body_len);

        match client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                info!("[AUR 批量查询] 响应状态码: {}", status);

                if !status.is_success() {
                    warn!(
                        "[AUR 批量查询] 第 {}/{} 批请求失败: HTTP {}",
                        i + 1,
                        total_chunks,
                        status
                    );
                    *failure_reasons.entry("HTTP 非 2xx 状态码").or_insert(0) += 1;
                    if let Ok(text) = resp.text().await {
                        warn!("[AUR 批量查询] 错误响应: {}", preview(&text, 500));
                    }
                    // 批次内失败不重试，继续下一批（AUR 限流保护）
                    continue;
                }

                // 先取 content-type，再消费响应体
                let content_type = resp
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string();

                match resp.text().await {
                    Ok(text) => {
                        debug!(
                            "[AUR 批量查询] 响应长度: {} 字节, content-type: {}",
                            text.len(),
                            if content_type.is_empty() {
                                "<缺失>"
                            } else {
                                &content_type
                            }
                        );

                        // 反爬中间件拦截：HTTP 200 但返回 HTML 拦截页，
                        // 必须显式报错，否则会被当成"无结果"静默吞掉
                        if looks_like_intercepted(&content_type, &text) {
                            warn!(
                                "[AUR 批量查询] 第 {}/{} 批疑似被反爬中间件拦截，请检查 User-Agent\
                                 （content-type: {}，响应前 500 字符: {}）",
                                i + 1,
                                total_chunks,
                                if content_type.is_empty() {
                                    "<缺失>"
                                } else {
                                    &content_type
                                },
                                preview(&text, 500)
                            );
                            *failure_reasons.entry("疑似被反爬中间件拦截").or_insert(0) += 1;
                            continue;
                        }

                        match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(data) => {
                                let resultcount = data["resultcount"].as_i64().unwrap_or(0);
                                info!("[AUR 批量查询] 返回 {} 个结果", resultcount);

                                // AUR 在无错误时可能返回空字符串，需排除以免误判
                                if let Some(error) = data["error"].as_str() {
                                    if !error.is_empty() {
                                        warn!("[AUR 批量查询] AUR 返回错误: {}", error);
                                        *failure_reasons
                                            .entry("AUR 返回 error 字段")
                                            .or_insert(0) += 1;
                                        continue;
                                    }
                                }

                                if let Some(results) = data["results"].as_array() {
                                    all_results.extend(results.iter().cloned());
                                }
                                succeeded_batches += 1;
                            }
                            Err(e) => {
                                warn!("[AUR 批量查询] JSON 解析失败: {}", e);
                                warn!("[AUR 批量查询] 响应内容: {}", preview(&text, 500));
                                *failure_reasons.entry("JSON 解析失败").or_insert(0) += 1;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("[AUR 批量查询] 读取响应体失败: {}", e);
                        *failure_reasons.entry("读取响应体失败").or_insert(0) += 1;
                    }
                }
            }
            Err(e) => {
                warn!("[AUR 批量查询] 请求失败: {}", e);
                *failure_reasons.entry("请求发送失败").or_insert(0) += 1;
            }
        }
    }

    let failed_batches = total_chunks.saturating_sub(succeeded_batches);
    if failed_batches == 0 {
        info!(
            "[AUR 批量查询] 完成：成功 {}/{} 批，共获取 {} 个结果",
            succeeded_batches,
            total_chunks,
            all_results.len()
        );
    } else {
        let reason_summary = failure_reasons
            .iter()
            .map(|(reason, count)| format!("{} ×{}", reason, count))
            .collect::<Vec<String>>()
            .join("; ");
        warn!(
            "[AUR 批量查询] 完成：成功 {}/{} 批（失败 {} 批），共获取 {} 个结果；失败原因摘要: {}",
            succeeded_batches,
            total_chunks,
            failed_batches,
            all_results.len(),
            reason_summary
        );
        if succeeded_batches == 0 {
            warn!(
                "[AUR 批量查询] 全部 {} 批均失败，请检查网络、代理或 User-Agent（当前 UA 见 http_client::DEFAULT_USER_AGENT）",
                total_chunks
            );
        }
    }

    Ok(all_results)
}

#[cfg(test)]
mod tests;
