use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 编程语言信息
/// 对应数据库 enum_programming_languages 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumProgrammingLanguage {
    /// 语言 ID，数据库自增主键
    pub id: Option<i64>,
    /// 语言名称（如 "Rust", "Python"）
    pub name: String,
    /// 语言描述
    pub description: Option<String>,
    /// 关联的文件扩展名（逗号分隔，如 ".rs,.toml"）
    pub file_extensions: Option<String>,
    /// 使用的构建系统（如 "cargo", "npm", "make"）
    pub build_system: Option<String>,
    /// 构建命令（如 "cargo build", "npm run build"）
    pub build_command: Option<String>,
}
