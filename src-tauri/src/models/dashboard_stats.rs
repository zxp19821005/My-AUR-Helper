use serde::{Deserialize, Serialize};

/// 仪表盘统计结果
/// 对应前端 DashboardStats 接口，一次 IPC 返回全部模块计数，
/// 避免前端为统计而全量拉取各表数据（软件包/代理/License 等）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    /// 软件包总数
    pub pkg_total: i64,
    /// 已是最新（is_outdated = 0）的软件包数
    pub pkg_updated: i64,
    /// 有更新（is_outdated = 1）的软件包数
    pub pkg_outdated: i64,
    /// 备份记录总数
    pub backup_total: i64,
    /// 缓存记录总数
    pub cache_total: i64,
    /// 代理源总数
    pub proxy_total: i64,
    /// 可用（is_active = 1）代理源数
    pub proxy_active: i64,
    /// License 枚举总数
    pub license_total: i64,
    /// 编程语言枚举总数
    pub language_total: i64,
}
