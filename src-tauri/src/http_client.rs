/**
 * http_client.rs - 共享 HTTP 客户端单例
 *
 * reqwest::Client 内部持有连接池，属重量级资源，应在进程内复用，
 * 避免每次请求重新构建导致连接池无法复用、TLS 会话无法恢复。
 * 提供默认配置的全局单例，供无需特殊配置的网络请求复用。
 */
use std::sync::OnceLock;
use std::time::Duration;

use reqwest::Client;

/// 默认请求超时时间（秒）
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// 获取进程级共享的 HTTP 客户端单例
///
/// 客户端启用 30 秒超时并自动跟随重定向；reqwest 内部连接池在所有
/// 调用方之间复用。首次调用时惰性初始化，之后始终返回同一实例。
///
/// @returns 全局共享的 `Client` 引用
pub fn shared_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("初始化共享 HTTP 客户端失败")
    })
}
