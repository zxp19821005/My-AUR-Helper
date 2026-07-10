use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 编程语言信息
/// 对应数据库 enum_programming_languages 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumProgrammingLanguage {
    /// 语言 ID，数据库自增主键
    pub id: Option<i64>,
    /// 语言名称（如 "Rust", "Python"）
    pub name: String,
    /// 语言简称（如 "rs", "py"）
    pub short_name: Option<String>,
}