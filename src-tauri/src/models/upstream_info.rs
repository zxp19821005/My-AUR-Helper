use chrono::NaiveDateTime; // 日期时间类型
use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 上游版本信息
/// 对应数据库 upstream_info 表，存储版本检查器检测到的上游版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamInfo {
    /// 关联的软件包 ID（主键）
    pub software_id: i64,
    /// 上游项目地址
    pub upstream_url: Option<String>,
    /// 检测到的上游最新版本号
    pub upstream_version: Option<String>,
    /// 上游项目的 License
    pub upstream_license: Option<String>,
    /// 上次版本检查的时间
    pub last_checked: Option<NaiveDateTime>,
}
