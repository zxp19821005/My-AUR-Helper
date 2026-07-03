use chrono::NaiveDateTime; // 日期时间类型
use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// License 信息
/// 对应数据库 enum_licenses 表，存储 SPDX License 标准数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumLicense {
    /// License ID，数据库自增主键
    pub id: Option<i64>,
    /// SPDX 标准 ID（如 "MIT", "GPL-3.0-only"）
    pub spdx_id: String,
    /// License 完整名称
    pub full_name: String,
    /// License 参考 URL
    pub url: Option<String>,
    /// 是否已被 SPDX 弃用
    pub is_deprecated: bool,
    /// 是否被 OSI（开放源代码促进会）批准
    pub is_osi_approved: bool,
    /// License 描述信息
    pub description: Option<String>,
    /// License 分类（如 "permissive", "copyleft"）
    pub category: Option<String>,
    /// 记录创建时间
    pub created_at: Option<NaiveDateTime>,
}
