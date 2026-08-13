/**
 * test.rs - 代理延迟测试模块
 *
 * 功能：
 * - 测试代理的延迟和连通性
 * - 下载/克隆/RAW 代理按「代理基址 + 真实目标地址」拼接测试 URL
 * - SSH 代理退回验证其 HTTPS 网关连通性
 * - 规整代理 URL，剥离用户脚本中可能已拼接的下载地址后缀（如 /https://github.com）
 * - 输出结构化详细日志（含重定向链路、状态码、延迟），便于排查测试异常
 */
use log::{debug, info};
use reqwest::Client;
use std::time::Duration;

use crate::errors::{AppError, AppResult};
use crate::models::ProxyType;
use crate::proxy::normalize_proxy_url;

/// 单次代理测试的全局超时（连接 + 响应）。
/// 没有超时的情况下，遇到「连接被接受但不响应」的代理会一直挂到 OS TCP 超时，
/// 顺序批量测试时整批可能被拖死几十分钟。10s 是 HEAD 探测的合理上限。
const TEST_TIMEOUT: Duration = Duration::from_secs(10);

/// 创建带重定向日志的 HTTP 客户端
/// 通过自定义重定向策略，把每一次 3xx 跳转的目标地址打印出来，
/// 这样就能看清「代理 A 跳转到了代理 B」这类链路（例如 crashmc → akaere）。
fn create_test_client(proxy_id: i64, proxy_name: &str) -> AppResult<Client> {
    let name = proxy_name.to_string();
    Client::builder()
        .timeout(TEST_TIMEOUT)
        .redirect(reqwest::redirect::Policy::custom(move |attempt| {
            info!(
                "[代理#{} {}] 跟随重定向 -> {}",
                proxy_id,
                name,
                mask_url(&attempt.url().to_string())
            );
            attempt.follow()
        }))
        .build()
        .map_err(|e| AppError::NetworkError(format!("创建 HTTP 客户端失败: {}", e)))
}

/// 代理基址规整复用 `crate::proxy::normalize_proxy_url`，该函数会剥离
/// 用户脚本中可能已拼接的下载地址后缀（/https://、?https:// 等）。

/// 从任意代理 URL 中提取主机名（剔除协议头、用户、端口、路径）
fn extract_host(url: &str) -> String {
    let s = url
        .replace("ssh://", "")
        .replace("http://", "")
        .replace("https://", "");
    let s = if let Some(at) = s.rfind('@') {
        &s[at + 1..]
    } else {
        &s[..]
    };
    let s = s.split('/').next().unwrap_or(s);
    let s = s.split(':').next().unwrap_or(s);
    s.to_string()
}

/// 脱敏日志中的 URL：剥离协议头与可能的 userinfo（user:pass@），
/// 仅保留代理主机名 + 目标路径，避免把代理凭据或完整代理 URL 写入日志。
fn mask_url(url: &str) -> String {
    if let Some(i) = url.find("://") {
        let rest = &url[i + 3..];
        if let Some(j) = rest.find('/') {
            let base = &rest[..j];
            return format!("{}{}", extract_host(base), &rest[j..]);
        }
        return extract_host(rest);
    }
    extract_host(url)
}

/// 拼接测试 URL：{代理基址}/{真实下载地址}
/// 仅当目标地址非空时才拼接，避免产生裸代理地址（如 https://cdn.xxx/）。
/// @param strip_target_protocol - 为 true 时去除目标地址的协议头（如 cors.isteed.cc 类约定），
///        否则保留（如 cdn.crashmc.com 类约定）。约定由解析时推断并持久化。
fn build_test_url(proxy_url: &str, target: &str, strip_target_protocol: bool) -> AppResult<String> {
    let base = normalize_proxy_url(proxy_url).0; // 规整为源站基址
    let target = target.trim();
    if target.is_empty() {
        return Err(AppError::NetworkError(
            "测试目标地址为空，无法拼接代理测试 URL".into(),
        ));
    }
    let target = target.trim_start_matches('/');
    // 按约定决定是否去除目标协议头
    let target = if strip_target_protocol {
        target
            .replacen("https://", "", 1)
            .replacen("http://", "", 1)
    } else {
        target.to_string()
    };
    Ok(format!("{}/{}", base, target))
}

/// 发送 HEAD 请求并测量延迟，仅当响应状态码为 2xx 时视为成功
/// 打印完整测试地址、HTTP 状态码与延迟，便于排查。
async fn send_head(
    client: &Client,
    proxy_id: i64,
    proxy_name: &str,
    url: &str,
    err_prefix: &str,
) -> AppResult<i64> {
    info!(
        "[代理#{} {}] 测试地址: {}",
        proxy_id,
        proxy_name,
        mask_url(url)
    );
    let start = std::time::Instant::now();
    let resp = client.head(url).send().await?;
    let latency = start.elapsed().as_millis() as i64;
    let status = resp.status();
    if !status.is_success() {
        info!(
            "[代理#{} {}] 测试失败: {} (状态码 {}) 延迟 {}ms",
            proxy_id, proxy_name, err_prefix, status, latency
        );
        return Err(AppError::NetworkError(format!(
            "{}，状态码: {}",
            err_prefix, status
        )));
    }
    info!(
        "[代理#{} {}] 测试成功: 状态码 {} 延迟 {}ms",
        proxy_id, proxy_name, status, latency
    );
    Ok(latency)
}

/// 根据代理类型测试代理
/// @param proxy_id - 代理 ID（用于日志上下文）
/// @param proxy_name - 代理名称（用于日志上下文）
/// @param proxy_url - 代理 URL（会被规整为基址）
/// @param proxy_type - 代理类型
/// @param test_url - 测试目标地址（可选，为 None 时按类型使用默认下载地址）
/// @param strip_target_protocol - 目标协议头约定（见 build_test_url）
/// @returns 测试结果（延迟毫秒数）
pub async fn test_proxy_by_type(
    proxy_id: i64,
    proxy_name: &str,
    proxy_url: &str,
    proxy_type: &ProxyType,
    test_url: Option<&str>,
    strip_target_protocol: bool,
) -> AppResult<i64> {
    let client = create_test_client(proxy_id, proxy_name)?;
    debug!(
        "[代理#{} {}] 开始测试（类型 {:?}），代理主机: {}",
        proxy_id,
        proxy_name,
        proxy_type,
        extract_host(&normalize_proxy_url(proxy_url).0)
    );
    match proxy_type {
        ProxyType::Download => {
            // 下载代理：代理基址 + 真实下载地址
            let target = test_url.unwrap_or(
                "https://github.com/zxp19821005/My_AUR_Files/releases/latest/download/README.md",
            );
            let full = build_test_url(proxy_url, target, strip_target_protocol)?;
            send_head(&client, proxy_id, proxy_name, &full, "下载代理测试失败").await
        }
        ProxyType::Clone => {
            // 克隆代理：代理基址 + 真实克隆地址
            let target = test_url.unwrap_or("https://github.com/zxp19821005/My_AUR_Files.git");
            let full = build_test_url(proxy_url, target, strip_target_protocol)?;
            send_head(&client, proxy_id, proxy_name, &full, "克隆代理测试失败").await
        }
        ProxyType::Raw => {
            // RAW 代理：代理基址 + 真实 RAW 文件地址
            let target = test_url.unwrap_or(
                "https://raw.githubusercontent.com/zxp19821005/My_AUR_Files/main/README.md",
            );
            let full = build_test_url(proxy_url, target, strip_target_protocol)?;
            send_head(&client, proxy_id, proxy_name, &full, "RAW 代理测试失败").await
        }
        ProxyType::Ssh => {
            // SSH 代理本质走 git/ssh 协议，HTTP HEAD 无法直接验证，
            // 退回验证其 HTTPS 网关的连通性
            let base = normalize_proxy_url(proxy_url).0;
            let gateway = format!("https://{}", extract_host(&base));
            send_head(
                &client,
                proxy_id,
                proxy_name,
                &gateway,
                "SSH 代理网关测试失败",
            )
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_appended_download_suffix() {
        // 后缀带协议头 → 源站基址 + 保留目标协议头约定(false)
        assert_eq!(
            normalize_proxy_url("https://cdn.akaere.online/https://github.com"),
            ("https://cdn.akaere.online".to_string(), false)
        );
        // 兼容 ?https:// 形式的后缀（如 down.npee.cn）
        assert_eq!(
            normalize_proxy_url("https://down.npee.cn/?https://github.com"),
            ("https://down.npee.cn".to_string(), false)
        );
        // 干净源站 → 默认保留目标协议头
        assert_eq!(
            normalize_proxy_url("https://cdn.crashmc.com/"),
            ("https://cdn.crashmc.com".to_string(), false)
        );
        // 后缀为裸主机（无协议头）→ 源站基址 + 去除目标协议头约定(true)
        assert_eq!(
            normalize_proxy_url("https://cors.isteed.cc/github.com"),
            ("https://cors.isteed.cc".to_string(), true)
        );
    }

    #[test]
    fn build_url_keeps_scheme_of_target() {
        // crashmc 类约定：保留目标协议头
        let u = build_test_url(
            "https://cdn.crashmc.com/",
            "https://github.com/zxp19821005/My_AUR_Files/releases/latest/download/README.md",
            false,
        )
        .unwrap();
        assert_eq!(
            u,
            "https://cdn.crashmc.com/https://github.com/zxp19821005/My_AUR_Files/releases/latest/download/README.md"
        );
    }

    #[test]
    fn build_url_strips_target_scheme_for_isteed() {
        // isteed 类约定：去除目标协议头，得到 github.com/... 而非 https://github.com/...
        let u = build_test_url(
            "https://cors.isteed.cc/github.com",
            "https://github.com/zxp19821005/My_AUR_Files/releases/latest/download/README.md",
            true,
        )
        .unwrap();
        assert_eq!(
            u,
            "https://cors.isteed.cc/github.com/zxp19821005/My_AUR_Files/releases/latest/download/README.md"
        );
    }

    #[test]
    fn build_url_strips_leading_slash() {
        let u = build_test_url("https://cdn.crashmc.com/", "/github.com/foo", false).unwrap();
        assert_eq!(u, "https://cdn.crashmc.com/github.com/foo");
    }

    #[test]
    fn build_url_rejects_empty_target() {
        assert!(build_test_url("https://cdn.crashmc.com/", "", false).is_err());
    }

    #[test]
    fn extract_host_handles_ssh_and_https() {
        assert_eq!(
            extract_host("ssh://git@ssh.github.com:443"),
            "ssh.github.com"
        );
        assert_eq!(extract_host("https://cdn.crashmc.com/"), "cdn.crashmc.com");
    }
}
