use log::info;
use std::time::Duration;

use crate::models::ProxyType;

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

pub fn build_client(timeout_secs: u64, proxy_url: Option<&str>) -> reqwest::Client {
    build_client_with_redirect(timeout_secs, proxy_url, true)
}

pub fn build_client_with_redirect(
    timeout_secs: u64,
    proxy_url: Option<&str>,
    follow_redirects: bool,
) -> reqwest::Client {
    let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(timeout_secs));

    if !follow_redirects {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    }

    if let Some(url) = proxy_url {
        if url.starts_with("http://") || url.starts_with("https://") {
            info!("[HTTP代理] 使用代理");
            if let Ok(proxy) = reqwest::Proxy::all(url) {
                builder = builder.proxy(proxy);
            }
        } else if url.starts_with("socks5://") {
            info!("[HTTP代理] SOCKS5代理不支持（需要启用 socks 特性），跳过");
        }
    }
    builder.build().unwrap_or_default()
}
