// @LINE_LIMIT_EXEMPT: 测试文件
/**
 * aur/metadata/tests.rs - AUR 元数据归档下载参数防护测试
 *
 * 功能：守护归档下载超时与重试参数处于合理区间，防止未来改动退化成
 * 「15MB 下载频繁超时 → 降级到慢速 RPC 分批 → 同步卡 2 分钟」。
 *
 * 工作流程：直接引用 download 子模块的常量做区间断言，线性堆叠、无 I/O。
 *
 * 适用场景：`cargo test --lib` 回归下载参数的护栏，避免性能退化被无声引入。
 */
use std::time::Duration;

use super::download::{ARCHIVE_DOWNLOAD_RETRIES, ARCHIVE_DOWNLOAD_TIMEOUT, ARCHIVE_RETRY_BACKOFF};

/// 归档下载超时必须显著大于普通 API 的 http_timeout（默认 30s），
/// 防止未来有人改回 30s 导致 15MB 下载频繁超时、重新退化成「同步卡 2 分钟」。
#[test]
fn test_archive_download_timeout_is_generous() {
    assert!(
        ARCHIVE_DOWNLOAD_TIMEOUT >= Duration::from_secs(90),
        "归档下载超时过短，15MB 文件易在读取响应体阶段超时"
    );
}

/// 重试参数应处于合理区间：至少重试 1 次，退避不超过 5 秒。
/// 重试过多会拖慢「归档确实不可用」时的降级判定，退避过长同理。
#[test]
fn test_archive_retry_params_are_bounded() {
    assert!(
        (1..=3).contains(&ARCHIVE_DOWNLOAD_RETRIES),
        "归档重试次数应控制在 1~3 次"
    );
    assert!(
        ARCHIVE_RETRY_BACKOFF <= Duration::from_secs(5),
        "归档重试退避不应超过 5 秒"
    );
}
