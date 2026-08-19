/**
 * proxy_utils.rs - 代理工具函数
 *
 * 提供 HTTP 客户端构建和代理获取功能
 */
use std::time::Duration;

use crate::models::ProxyType;

/// 获取 GNOME 系统代理设置
fn get_gnome_system_proxy() -> Option<String> {
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

/// 获取当前活跃的代理配置
///
/// 优先级：
/// 1. 数据库中启用的代理
/// 2. GNOME 系统代理
/// 3. 环境变量代理（http_proxy/https_proxy/all_proxy）
pub fn get_active_proxy(db: &crate::db::Database) -> Option<String> {
    let db_proxy = db
        .get_active_proxies(&ProxyType::Download)
        .ok()
        .and_then(|list| list.into_iter().next())
        .map(|p| p.url);
    if db_proxy.is_some() {
        return db_proxy;
    }

    if let Some(proxy) = get_gnome_system_proxy() {
        return Some(proxy);
    }

    std::env::var("http_proxy")
        .ok()
        .or_else(|| std::env::var("https_proxy").ok())
        .or_else(|| std::env::var("all_proxy").ok())
        .filter(|v| !v.is_empty())
}

/// 构建 HTTP 客户端
pub fn build_client(timeout_secs: u64, proxy_url: Option<&str>) -> reqwest::Client {
    build_client_with_redirect(timeout_secs, proxy_url, true)
}

/// 构建 HTTP 客户端（可配置是否跟随重定向）
///
/// 本函数的调用方（版本检查 / AUR 同步 / 枚举拉取）均为普通 API 请求，
/// 一律直连，不再把数据库里的 GitHub 加速镜像当成正向代理强加给请求。
///
/// 原因：本项目的代理全部来自「GitHub 加速」用户脚本，均为 GitHub 镜像
/// （反向代理），只能用于「主机替换」方式加速 GitHub 文件下载/克隆，
/// 绝对不能作为正向代理接管 api.github.com、appversion.115.com、
/// aur.archlinux.org 等请求——日志里 `proxy(cdn.crashmc.com) intercepts ...`
/// 之后连接失败，正是这个原因。
///
/// 若系统/环境配置了真正的正向代理（http_proxy/https_proxy 等），
/// reqwest 默认会自动读取并使用，无需在此手动设置。
pub fn build_client_with_redirect(
    timeout_secs: u64,
    _proxy_url: Option<&str>,
    follow_redirects: bool,
) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(10));

    if !follow_redirects {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    }

    builder.build().unwrap_or_default()
}
