/**
 * proxy_utils.rs - 代理工具函数
 *
 * 提供 HTTP 客户端构建和代理获取功能。
 *
 * 正向代理检测优先级：
 * 1. 环境变量（HTTP_PROXY / HTTPS_PROXY / ALL_PROXY）- reqwest 自动读取
 * 2. 本地代理端口（Mihomo / Clash Verge / V2Ray 等常见端口）
 * 3. GNOME/KDE 系统代理设置
 *
 * 本地代理检测原理：
 *   通过尝试连接 127.0.0.1 上的常见代理端口（7897/7898/1080/10808/...），
 *   检测是否有本地代理程序（如 Mihomo、Clash Verge Rev）在运行。
 *   这是因为用户从 .desktop 文件启动应用时，桌面环境可能没有
 *   将代理环境变量传递给子进程，但本地代理端口通常始终可用。
 */
use std::net::TcpStream;
use std::time::Duration;

use crate::models::ProxyType;

/// 本地代理常见端口列表（按优先级排序）
///
/// 涵盖主流代理客户端的默认端口：
/// - 7897: Mihomo/Clash Verge Rev mixed-port
/// - 7898: Mihomo/Clash Verge Rev socks-port
/// - 7890: Clash standard mixed-port
/// - 7891: Clash standard port
/// - 1080: SOCKS 代理默认端口
/// - 10808: V2Ray 默认 SOCKS 端口
/// - 10809: V2Ray 默认 HTTP 端口
/// - 8080: 通用 HTTP 代理
const LOCAL_PROXY_PORTS: &[u16] = &[
    7897, 7898, 7890, 7891, 7892, 7893, 1080, 10808, 10809, 1081, 8080,
];

/// 检测本地是否有代理服务在运行
///
/// 逐个尝试连接常见代理端口，返回第一个可用的代理地址。
/// 连接超时设为 200ms，避免在无代理环境下产生明显延迟。
fn detect_local_proxy() -> Option<String> {
    let connect_timeout = Duration::from_millis(200);
    for port in LOCAL_PROXY_PORTS {
        let addr = format!("127.0.0.1:{}", port);
        if TcpStream::connect_timeout(&addr.parse().unwrap(), connect_timeout).is_ok() {
            return Some(format!("http://{}", addr));
        }
    }
    None
}

/// 获取环境变量中配置的代理地址
///
/// reqwest 本身会自动读取这些环境变量，但这里先提取出来
/// 用于日志记录和本地代理的优先级判定。
fn get_env_proxy() -> Option<String> {
    std::env::var("http_proxy")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| {
            std::env::var("https_proxy")
                .ok()
                .filter(|v| !v.is_empty())
        })
        .or_else(|| {
            std::env::var("ALL_PROXY")
                .ok()
                .filter(|v| !v.is_empty())
        })
        .or_else(|| {
            std::env::var("all_proxy")
                .ok()
                .filter(|v| !v.is_empty())
        })
}

/// 获取 GNOME/KDE 系统代理设置
///
/// 通过 gsettings 读取 GNOME 系统代理配置，
/// KDE 桌面环境下 gsettings 同样可用（使用 glib-networking）。
fn get_desktop_proxy() -> Option<String> {
    let mode = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.system.proxy", "mode"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                Some(s)
            } else {
                None
            }
        })?;
    if mode != "'manual'" {
        return None;
    }
    let host = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.system.proxy.http", "host"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .trim_matches('\'')
                    .to_string();
                (!s.is_empty()).then_some(s)
            } else {
                None
            }
        })?;
    let port = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.system.proxy.http", "port"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                s.parse::<u16>().ok()
            } else {
                None
            }
        })?;
    Some(format!("http://{}:{}", host, port))
}

/// 获取系统级正向代理
///
/// 优先级：环境变量 > 本地代理 > 桌面系统代理
///
/// # Returns
/// 可用的正向代理 URL，格式如 `http://127.0.0.1:7897`
pub fn get_forward_proxy() -> Option<String> {
    // 1. 环境变量（最高优先，reqwest 也会自动读取）
    if let Some(proxy) = get_env_proxy() {
        return Some(proxy);
    }
    // 2. 本地代理端口（覆盖 .desktop 启动时环境变量丢失的情况）
    if let Some(proxy) = detect_local_proxy() {
        return Some(proxy);
    }
    // 3. 桌面系统代理设置（GNOME/KDE 通用）
    if let Some(proxy) = get_desktop_proxy() {
        return Some(proxy);
    }
    None
}

/// 获取当前活跃的数据库代理配置（GitHub 加速镜像，仅用于主机替换）
///
/// 仅返回数据库中配置的 GitHub 加速镜像（反向代理），
/// 不返回系统级正向代理。正向代理请使用 `get_forward_proxy()`。
pub fn get_active_proxy(db: &crate::db::Database) -> Option<String> {
    let db_proxy = db
        .get_active_proxies(&ProxyType::Download)
        .ok()
        .and_then(|list| list.into_iter().next())
        .map(|p| p.url);
    if db_proxy.is_some() {
        return db_proxy;
    }
    None
}

/// 构建 HTTP 客户端
///
/// # Arguments
/// * `timeout_secs` - 请求超时秒数
/// * `use_proxy` - 是否为 GitHub 请求启用正向代理
pub fn build_client(timeout_secs: u64, use_proxy: bool) -> reqwest::Client {
    build_client_with_redirect(timeout_secs, use_proxy, true)
}

/// 构建 HTTP 客户端（可配置是否跟随重定向）
///
/// 代理在客户端级别配置，确保 GitHub 请求走代理，非 GitHub 请求不走代理。
///
/// # Arguments
/// * `timeout_secs` - 请求超时秒数
/// * `use_proxy` - 是否为 GitHub 请求启用正向代理
/// * `follow_redirects` - 是否自动跟随 HTTP 重定向
pub fn build_client_with_redirect(
    timeout_secs: u64,
    use_proxy: bool,
    follow_redirects: bool,
) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(10));

    if !follow_redirects {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    }

    if use_proxy {
        if let Some(proxy_url) = get_forward_proxy() {
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                builder = builder.proxy(proxy);
            }
        }
    }

    builder.build().unwrap_or_default()
}