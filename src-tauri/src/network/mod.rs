/**
 * network/mod.rs - 网络工具模块入口
 *
 * 子模块：
 * - retry    : 指数退避重试逻辑
 */
pub mod retry;

pub use retry::{retry_with_backoff, is_retryable_error};
