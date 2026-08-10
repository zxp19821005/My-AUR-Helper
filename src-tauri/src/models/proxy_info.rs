use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

use super::ProxyType; // 代理类型枚举

/// 代理信息
/// 对应数据库 proxies_info 表，并可选关联 proxies_test 统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    /// 代理 ID，数据库自增主键
    pub proxy_id: Option<i64>,
    /// 代理名称（通常使用 URL 域名作为名称）
    pub proxy_name: String,
    /// 代理类型：download / clone / raw / ssh
    pub proxy_type: ProxyType,
    /// 代理 URL 地址
    pub url: String,
    /// 是否启用
    pub is_active: bool,
    /// 成功次数（来自关联的 proxies_test 最新记录）
    #[serde(default)]
    pub success_count: i64,
    /// 失败次数（来自关联的 proxies_test 最新记录）
    #[serde(default)]
    pub fail_count: i64,
    /// 平均延迟 ms（来自关联的 proxies_test 最新记录）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_latency: Option<i64>,
    /// 最后一次测试状态：success / fail（来自关联的 proxies_test 最新记录，None 表示未测试）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_test_status: Option<String>,
    /// 目标协议头约定：true 表示测试拼接时去除目标地址的 https://http://（如 cors.isteed.cc 类），
    /// false 表示保留（如 cdn.crashmc.com 类）。由解析时从原始脚本条目推断并持久化。
    #[serde(default)]
    pub strip_target_protocol: bool,
}
