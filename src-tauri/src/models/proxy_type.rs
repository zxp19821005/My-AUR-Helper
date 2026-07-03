use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 代理类型枚举
/// 定义代理的不同用途
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProxyType {
    /// 下载代理 - 用于加速文件下载
    Download,
    /// 克隆代理 - 用于 Git 克隆操作
    Clone,
    /// Raw 代理 - 用于 raw 文件访问
    Raw,
    /// SSH 代理 - 用于 SSH 连接
    Ssh,
}

impl ProxyType {
    /// 将枚举转换为字符串（用于数据库存储）
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyType::Download => "download",
            ProxyType::Clone => "clone",
            ProxyType::Raw => "raw",
            ProxyType::Ssh => "ssh",
        }
    }

    /// 从字符串创建枚举（用于数据库查询反序列化）
    /// @param s - 字符串标识
    /// @returns 对应的枚举值，未知字符串默认返回 Download
    pub fn parse_from(s: &str) -> Self {
        match s {
            "download" => ProxyType::Download,
            "clone" => ProxyType::Clone,
            "raw" => ProxyType::Raw,
            "ssh" => ProxyType::Ssh,
            _ => ProxyType::Download, // 默认值
        }
    }
}
