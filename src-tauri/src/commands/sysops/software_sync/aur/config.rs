/**
 * software_sync/aur/config.rs - AUR 同步设置读取聚合
 *
 * 功能：把 sync_from_aur 所需的数据库设置读取集中到一处，
 * 返回用户名、HTTP 超时、批次大小、批次间隔与归档偏好。
 *
 * 工作流程：
 * 1. 从 settings 表读 aur_username（未配置时为空字符串）
 * 2. 读 http_timeout / aur_batch_size / aur_batch_interval / 归档偏好
 * 3. 调用方据此决定 build_client 参数与 RPC 降级分批参数
 *
 * 适用场景：AUR 全量同步命令的设置装配阶段。
 */
use crate::commands::sysops::software_sync::utils::{
    get_setting_opt, parse_u32, parse_u64, read_archive_preference, read_http_timeout,
};
use crate::db::Database;
use crate::errors::{AppError, AppResult};

/// AUR 同步设置集合（从 settings 表一次性读出）
pub(super) struct SyncSettings {
    /// AUR 用户名（为空表示未配置）
    pub(super) username: String,
    /// HTTP 超时秒数
    pub(super) timeout: u64,
    /// 降级 RPC 时每批最大包数
    pub(super) batch_size: usize,
    /// 降级 RPC 时批次间隔秒数
    pub(super) batch_interval: u64,
    /// 是否优先使用全量元数据归档
    pub(super) prefer_archive: bool,
}

/// 从数据库读取 AUR 同步所需设置。
///
/// # 参数
/// - `db`: 已加锁的数据库引用
///
/// # 返回
/// - `Ok(SyncSettings)`: 装配好的设置集合
/// - `Err(AppError)`: 读取设置失败
pub(super) fn read_sync_settings(db: &Database) -> AppResult<SyncSettings> {
    let username = db
        .get_setting("aur_username")?
        .map(|s| s.value)
        .unwrap_or_default();
    let timeout = read_http_timeout(db);
    let batch_size = parse_u32(
        &get_setting_opt(db, "aur_batch_size").unwrap_or_default(),
        50,
    ) as usize;
    let batch_interval = parse_u64(
        &get_setting_opt(db, "aur_batch_interval").unwrap_or_default(),
        5,
    );
    let prefer_archive = read_archive_preference(db);
    Ok(SyncSettings {
        username,
        timeout,
        batch_size,
        batch_interval,
        prefer_archive,
    })
}

/// 校验用户名是否已配置，未配置时返回 ConfigError。
///
/// # 参数
/// - `username`: 数据库读取的用户名
///
/// # 返回
/// - `Ok(())`: 用户名有效
/// - `Err(AppError)`: 用户名为空
pub(super) fn require_username(username: &str) -> AppResult<()> {
    if username.is_empty() {
        Err(AppError::ConfigError("AUR 用户名未配置".to_string()))
    } else {
        Ok(())
    }
}
