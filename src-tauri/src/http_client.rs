/**
 * http_client.rs - 共享 HTTP 客户端管理
 *
 * 功能：
 * - 提供按域名分组的客户端缓存（连接池复用）
 * - 支持代理配置
 * - 提供默认全局客户端
 *
 * 设计原则：
 * - reqwest::Client 内部持有连接池，属重量级资源，应在进程内复用
 * - 不同域名可共享连接池（同一进程内 reqwest 自动管理）
 * - 代理仅在特定请求中按需注入，避免非 GitHub 请求走代理
 */
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use log::debug;
use reqwest::Client;

use crate::commands::sysops::proxy_utils::get_forward_proxy;

/// 默认请求超时时间（秒）
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// 默认连接超时时间（秒）
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;

/// 客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// 请求超时时间（秒）
    pub timeout_secs: u64,
    /// 连接超时时间（秒）
    pub connect_timeout_secs: u64,
    /// 是否启用代理
    pub use_proxy: bool,
    /// 是否跟随重定向
    pub follow_redirects: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            connect_timeout_secs: DEFAULT_CONNECT_TIMEOUT_SECS,
            use_proxy: false,
            follow_redirects: true,
        }
    }
}

/// 全局客户端单例（默认配置）
static DEFAULT_CLIENT: OnceLock<Client> = OnceLock::new();

/// 域名 -> 客户端映射缓存（带锁保护）
static CLIENT_CACHE: OnceLock<Mutex<HashMap<String, Client>>> = OnceLock::new();

/// 获取全局默认 HTTP 客户端
///
/// 客户端启用 30 秒超时、10 秒连接超时，并自动跟随重定向。
/// reqwest 内部连接池在所有调用方之间复用。
///
/// # Returns
/// 全局共享的 `Client` 引用
pub fn shared_client() -> &'static Client {
    DEFAULT_CLIENT.get_or_init(build_default_client)
}

/// 获取或创建针对特定域名的 HTTP 客户端
///
/// 使用域名作为键缓存客户端实例，避免重复构建。
/// 同一域名内的请求可复用 TCP 连接和 TLS 会话。
///
/// # 参数
/// * `domain` - 目标域名，如 "aur.archlinux.org"
/// * `config` - 客户端配置
///
/// # Returns
/// 目标域名的 HTTP 客户端
pub fn get_client_for_domain(domain: &str, config: &ClientConfig) -> Client {
    let cache = CLIENT_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = cache.lock().unwrap();

    // 尝试从缓存获取
    if let Some(client) = map.get(domain) {
        debug!("[HTTP 客户端] 命中缓存: {}", domain);
        return client.clone();
    }

    // 创建新客户端并缓存
    let client = build_client(config);
    map.insert(domain.to_string(), client.clone());
    debug!(
        "[HTTP 客户端] 新建客户端并缓存: {}, 缓存大小: {}",
        domain,
        map.len()
    );
    client
}

/// 构建默认配置的 HTTP 客户端
fn build_default_client() -> Client {
    build_client(&ClientConfig::default())
}

/// 根据配置构建 HTTP 客户端
///
/// # 参数
/// * `config` - 客户端配置
///
/// # Returns
/// 配置好的 HTTP 客户端
pub fn build_client(config: &ClientConfig) -> Client {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .connect_timeout(Duration::from_secs(config.connect_timeout_secs));

    if !config.follow_redirects {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    }

    // 代理在调用方按需注入，此处不全局配置
    // 确保 AUR、SPDX 等非 GitHub 请求不走代理

    builder.build().expect("初始化 HTTP 客户端失败")
}

/// 构建带代理的 HTTP 客户端（用于 GitHub 等特殊请求）
///
/// # 参数
/// * `timeout_secs` - 请求超时秒数
///
/// # Returns
/// 启用代理的 HTTP 客户端
pub fn build_proxy_client(timeout_secs: u64) -> Client {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS));

    if let Some(proxy_url) = get_forward_proxy() {
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            builder = builder.proxy(proxy);
            debug!("[HTTP 客户端] 使用代理: {}", proxy_url);
        }
    }

    builder.build().unwrap_or_default()
}

/// 清空客户端缓存（用于测试或配置变更时）
pub fn clear_client_cache() {
    if let Some(cache) = CLIENT_CACHE.get() {
        cache.lock().unwrap().clear();
        debug!("[HTTP 客户端] 缓存已清空");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_client_singleton() {
        let client1 = shared_client();
        let client2 = shared_client();
        // 两次调用应返回同一实例
        assert!(std::ptr::eq(client1, client2));
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.connect_timeout_secs, 10);
        assert!(!config.use_proxy);
        assert!(config.follow_redirects);
    }
}
