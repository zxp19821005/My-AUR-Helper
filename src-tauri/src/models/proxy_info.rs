use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

use super::ProxyType; // 代理类型枚举

/// 代理信息
/// 对应数据库 proxies_info 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    /// 代理 ID，数据库自增主键
    pub proxy_id: Option<i64>,
    /// 代理名称（通常使用 URL 作为名称）
    pub proxy_name: String,
    /// 代理类型：download / clone / raw / ssh
    pub proxy_type: ProxyType,
    /// 代理 URL 地址
    pub url: String,
    /// 是否启用
    pub is_active: bool,
}
