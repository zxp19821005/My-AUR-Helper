/**
 * aur/metadata/download.rs - AUR 元数据归档下载（独立长超时 client + 重试）
 *
 * 功能：
 * - 用独立长超时 client（120s 总超时 / 15s 连接超时）下载约 15MB 归档，
 *   不读取调用方的短 http_timeout，避免大文件读取响应体时超时而误降级
 * - 携带统一 User-Agent 规避 Anubis 反爬
 * - 带条件请求（ETag / Last-Modified）与有限重试（瞬时抖动恢复）
 *
 * 工作流程：
 * 1. build_archive_client 构造长超时客户端
 * 2. read_validators 取出校验器并在请求中附带 IF_NONE_MATCH / IF_MODIFIED_SINCE
 * 3. 下载循环：网络错误重试并退避；304 返回 NotModified；成功返回 Fresh 字节
 * 4. 解压/解析由主文件在 spawn_blocking 中完成，本模块只负责拿到原始字节
 *
 * 适用场景：get_packages_from_archive 的下载阶段，失败由主文件降级到 RPC。
 */
use std::time::{Duration, Instant};

use log::{info, warn};
use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::{Client, StatusCode};

use super::cache;
use crate::errors::AppError;
use crate::http_client::DEFAULT_USER_AGENT;

/// AUR 全量元数据归档地址（顶层为 JSON 数组，含依赖与许可证字段）
pub(super) const ARCHIVE_URL: &str = "https://aur.archlinux.org/packages-meta-ext-v1.json.gz";
/// 归档下载专用超时（秒）。
///
/// 归档 gzip 约 15MB、解压后约 74MB。普通 API 请求用的 `http_timeout`
/// 默认 30 秒是「从发请求到读完响应体」的总超时，15MB 下载稍遇网络
/// 波动就触发 `error decoding response body` 超时，进而降级到 RPC 分批
/// （1873 包 / 100 ≈ 19 批 × 5s ≈ 95s）——这就是「同步卡 2 分钟」的根因。
/// 归档一次成功后 TTL 300 秒内所有同步零网络，值得给宽松的下载超时。
pub(super) const ARCHIVE_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
/// 归档下载失败后的重试次数（不含首次）。网络瞬时抖动重试即可恢复，
/// 仍失败才降级到 RPC，避免一次抖动就付出 95 秒的降级代价。
pub(super) const ARCHIVE_DOWNLOAD_RETRIES: u32 = 2;
/// 两次归档下载尝试之间的退避（秒）
pub(super) const ARCHIVE_RETRY_BACKOFF: Duration = Duration::from_secs(1);

/// 归档下载结果：未变化（复用缓存）或拿到新字节与校验器
pub(super) enum ArchiveFetch {
    /// 归档未变化（HTTP 304），调用方应刷新缓存 TTL 并复用本地索引
    NotModified,
    /// 下载到新归档字节及其校验器
    Fresh {
        /// gzip 压缩后的原始字节
        bytes: Vec<u8>,
        /// 响应 ETag
        etag: Option<String>,
        /// 响应 Last-Modified
        last_modified: Option<String>,
    },
}

/// 构建归档下载专用客户端。
///
/// 与调用方传入的普通 client 隔离：归档是大文件下载，用独立的长总超时
/// （120s）与 15s 连接超时；仍携带统一 User-Agent 避免被 Anubis 拦截。
/// 不读取 `http_timeout` 设置——该设置面向普通 API，过短会误伤归档。
fn build_archive_client() -> Client {
    Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .timeout(ARCHIVE_DOWNLOAD_TIMEOUT)
        .connect_timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default()
}

/// 带条件请求与有限重试地下载归档。
///
/// 网络错误（发送失败 / 读取响应体失败）按 ARCHIVE_DOWNLOAD_RETRIES 重试并退避；
/// 304 / 4xx 属于确定性结果，不重试。校验器在 await 之前脱离锁作用域，
/// 避免 MutexGuard 跨 await 破坏 Send。
///
/// # 返回
/// - `Ok(ArchiveFetch::NotModified)`: 归档未变化，调用方 touch_cache 后复用索引
/// - `Ok(ArchiveFetch::Fresh { .. })`: 下载成功，调用方负责解压解析与写缓存
/// - `Err(AppError)`: 重试耗尽或确定性 HTTP 失败
pub(super) async fn download_archive_with_retry() -> crate::errors::AppResult<ArchiveFetch> {
    // 使用独立长超时 client，而非调用方传入的普通 client（后者 http_timeout=30s，
    // 作为 15MB 下载的总超时过短，是历史「同步卡 2 分钟」的根因）。
    let client = build_archive_client();
    let started = Instant::now();
    let (etag, last_modified) = cache::read_validators();

    let mut attempt = 0u32;
    loop {
        attempt += 1;
        info!(
            "[AUR 元数据归档] 开始下载 {}（第 {}/{} 次尝试）",
            ARCHIVE_URL,
            attempt,
            ARCHIVE_DOWNLOAD_RETRIES + 1
        );

        let mut request = client.get(ARCHIVE_URL);
        if let Some(ref value) = etag {
            request = request.header(IF_NONE_MATCH, value.as_str());
        }
        if let Some(ref value) = last_modified {
            request = request.header(IF_MODIFIED_SINCE, value.as_str());
        }

        let resp = match request.send().await {
            Ok(resp) => resp,
            Err(e) => {
                let err = format!("发送请求失败: {}", e);
                warn!(
                    "[AUR 元数据归档] 第 {}/{} 次下载失败: {}",
                    attempt,
                    ARCHIVE_DOWNLOAD_RETRIES + 1,
                    err
                );
                if attempt > ARCHIVE_DOWNLOAD_RETRIES {
                    return Err(AppError::NetworkError(format!(
                        "归档下载失败（已重试 {} 次）: {}",
                        ARCHIVE_DOWNLOAD_RETRIES, err
                    )));
                }
                tokio::time::sleep(ARCHIVE_RETRY_BACKOFF).await;
                continue;
            }
        };

        let status = resp.status();
        info!(
            "[AUR 元数据归档] 响应 HTTP {}，Content-Length: {:?}，连接耗时 {:?}",
            status,
            resp.content_length(),
            started.elapsed()
        );
        if status == StatusCode::NOT_MODIFIED {
            return Ok(ArchiveFetch::NotModified);
        }
        if !status.is_success() {
            return Err(AppError::NetworkError(format!(
                "AUR 元数据归档请求失败: HTTP {}",
                status
            )));
        }

        let resp_etag = resp
            .headers()
            .get(ETAG)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let resp_last_modified = resp
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);

        // 读取响应体是最易超时的环节（error decoding response body）
        match resp.bytes().await {
            Ok(body) => {
                info!(
                    "[AUR 元数据归档] 下载完成：{} 字节，耗时 {:?}，开始解压与 JSON 解析",
                    body.len(),
                    started.elapsed()
                );
                return Ok(ArchiveFetch::Fresh {
                    bytes: body.to_vec(),
                    etag: resp_etag,
                    last_modified: resp_last_modified,
                });
            }
            Err(e) => {
                let err = format!("读取响应体失败: {}", e);
                warn!(
                    "[AUR 元数据归档] 第 {}/{} 次下载失败: {}",
                    attempt,
                    ARCHIVE_DOWNLOAD_RETRIES + 1,
                    err
                );
                if attempt > ARCHIVE_DOWNLOAD_RETRIES {
                    return Err(AppError::NetworkError(format!(
                        "归档响应体读取失败（已重试 {} 次）: {}",
                        ARCHIVE_DOWNLOAD_RETRIES, err
                    )));
                }
                tokio::time::sleep(ARCHIVE_RETRY_BACKOFF).await;
                continue;
            }
        }
    }
}
