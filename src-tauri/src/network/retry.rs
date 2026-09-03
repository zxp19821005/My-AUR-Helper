/**
 * network/retry.rs - 指数退避重试逻辑
 *
 * 为可重试的网络错误提供自动重试机制，使用指数退避策略：
 * - 首次重试等待 1 秒，之后每次等待时间翻倍（1s, 2s, 4s, 8s...）
 * - 仅对 NetworkTimeout / NetworkConnect 类错误进行重试
 * - 非可重试错误（DNS 失败、404、解析错误等）立即返回
 */
use crate::errors::{AppError, AppResult};

/// 默认最大重试次数
pub const DEFAULT_MAX_RETRIES: u32 = 3;
/// 初始等待时间（秒）
pub const INITIAL_DELAY_SECS: u64 = 1;

/// 使用指数退避策略重试异步操作
///
/// # Arguments
/// * `max_retries` - 最大重试次数（不含首次尝试）
/// * `f` - 可重试的异步操作
///
/// # 重试规则
/// - 仅当错误是 `NetworkTimeout` 或 `NetworkConnect` 时进行重试
/// - 每次重试后等待时间翻倍：1s → 2s → 4s → ...
/// - 达到最大重试次数仍失败则返回最后一个错误
///
/// 使用指数退避重试异步操作（示例见下方测试，非 doctest）
pub async fn retry_with_backoff<F, Fut, T>(max_retries: u32, mut f: F) -> AppResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    let mut delay = INITIAL_DELAY_SECS;
    let mut last_err: Option<AppError> = None;

    for attempt in 0..=max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_retryable() && attempt < max_retries => {
                log::warn!(
                    "[重试] 第 {} 次尝试失败（{}），{} 秒后重试: {}",
                    attempt + 1,
                    e,
                    delay,
                    e
                );
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                delay *= 2; // 指数退避
                last_err = Some(e);
            }
            Err(e) => {
                // 非可重试错误或已达最大重试次数，返回错误
                return Err(e);
            }
        }
    }

    // 理论上不会到达此处（循环会 return），但编译器需要
    Err(last_err.unwrap_or_else(|| AppError::Unknown("重试逻辑异常".to_string())))
}

/// 判断错误是否可重试
///
/// 可重试：超时、连接失败（网络波动）
/// 不可重试：DNS 失败、404、403 限流、解析错误等
pub fn is_retryable_error(e: &AppError) -> bool {
    e.is_retryable()
}
