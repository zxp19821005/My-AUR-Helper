use chrono::NaiveDateTime; // 日期时间类型
use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 日志条目
/// 对应数据库 logs 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 日志 ID，数据库自增主键
    pub id: Option<i64>,
    /// 日志级别：INFO / WARN / ERROR / DEBUG
    pub level: String,
    /// 日志消息内容
    pub message: String,
    /// 日志来源模块名称
    pub module: Option<String>,
    /// 日志创建时间
    pub created_at: NaiveDateTime,
}
