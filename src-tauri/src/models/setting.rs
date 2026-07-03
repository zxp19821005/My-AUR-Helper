use chrono::NaiveDateTime; // 日期时间类型
use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 设置项
/// 对应数据库 settings 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    /// 设置 ID，数据库自增主键
    pub id: Option<i64>,
    /// 设置键名（唯一标识）
    pub key: String,
    /// 设置值
    pub value: String,
    /// 设置描述
    pub description: Option<String>,
    /// 设置分类（如 "aur", "backup", "general"）
    pub category: String,
    /// 创建时间
    pub created_at: Option<NaiveDateTime>,
}
