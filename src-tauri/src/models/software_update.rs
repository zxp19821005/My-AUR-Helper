use serde::{Deserialize, Serialize};
/**
 * software_update.rs - 软件更新模块数据传输对象
 *
 * 功能：定义更新工作台列表行、状态枚举、dry-run 预览与写回结果结构。
 *
 * 设计说明：本模块不新增数据库表。列表只取 software_info.is_outdated = 1 的包，
 * 写回状态由「本地 PKGBUILD 实际 pkgver」推导，写回后重新 rescan 即自动刷新，
 * 因此无需 update_history 之类的状态表，也不改动既有 schema。
 *
 * 适用场景：软件更新页面列表、diff 预览、批量写回 PKGBUILD。
 */
use specta::Type;

/// 写回状态（列表仅含 is_outdated = 1 的包，本枚举不持久化到数据库）
///
/// 说明：这里只描述「本地 PKGBUILD 是否已处理」，不描述「是否需要更新」——
/// 后者由 software_info.is_outdated 在版本检查阶段决定，页面不再重复判断。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SoftwareUpdateStatus {
    /// 本地 PKGBUILD 仍是旧版本，等待写回
    Pending,
    /// 本地 PKGBUILD 已写到上游版本，等待推送 AUR
    LocalUpdated,
    /// 本地包目录缺少 PKGBUILD，无法写回
    Missing,
    /// 无上游版本信息，无法写回
    NoUpstream,
}

/// 软件更新工作台列表行
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SoftwareUpdateItem {
    /// 软件包主键，同时作为表格 rowKey
    pub software_id: i64,
    /// 包名
    pub pkgname: String,
    /// AUR 现网版本（可能为空）
    pub aur_version: Option<String>,
    /// 上游最新版本（可能为空）
    pub upstream_version: Option<String>,
    /// 静默归一化后可写入 PKGBUILD 的目标版本（与写回值同源、无逐包日志）
    pub target_version: String,
    /// 本地 PKGBUILD 当前 pkgver
    pub local_pkgver: Option<String>,
    /// 本地 PKGBUILD 当前 pkgrel
    pub local_pkgrel: Option<String>,
    /// 本地 PKGBUILD 绝对路径
    pub pkgbuild_path: Option<String>,
    /// AUR 是否已标记 out-of-date
    pub aur_out_of_date: bool,
    /// 推导出的状态
    pub status: SoftwareUpdateStatus,
    /// 上游最近检查时间（Unix 秒）
    pub last_checked: Option<i64>,
}

/// dry-run 预览条目：只描述将要发生的改动，不执行任何写操作
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SoftwareUpdatePreview {
    /// 包名
    pub pkgname: String,
    /// PKGBUILD 绝对路径（不可预览时为空串）
    pub pkgbuild_path: String,
    /// 原 pkgver
    pub old_pkgver: String,
    /// 目标 pkgver
    pub new_pkgver: String,
    /// 原 pkgrel
    pub old_pkgrel: String,
    /// 目标 pkgrel（固定重置为 1）
    pub new_pkgrel: String,
    /// 仅列出发生变化行的文本 diff
    pub diff: String,
    /// 不可预览的原因；为 None 时该条目可安全执行
    pub error: Option<String>,
}

/// 单个软件包的写回结果（批量执行时逐包返回，单包失败不影响其它包）
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SoftwareUpdateApplyResult {
    /// 包名
    pub pkgname: String,
    /// 是否写回成功
    pub success: bool,
    /// 结果说明或失败原因
    pub message: String,
    /// 写回前的备份文件路径
    pub backup_path: Option<String>,
}

/// db 层跨表 JOIN 的原始行（不直接暴露给前端）
#[derive(Debug, Clone, Type)]
pub struct UpdateCandidateRow {
    /// 软件包主键
    pub software_id: i64,
    /// 包名
    pub pkgname: String,
    /// AUR 现网版本
    pub aur_version: Option<String>,
    /// AUR 是否标记 out-of-date
    pub out_of_date: bool,
    /// 上游最新版本
    pub upstream_version: Option<String>,
    /// 上游最近检查时间（Unix 秒）
    pub last_checked: Option<i64>,
}
