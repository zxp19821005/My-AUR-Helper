use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 缓存软件包信息
/// 对应数据库 cache_software 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSoftware {
    /// 缓存记录 ID，数据库自增主键
    pub id: Option<i64>,
    /// 关联的软件包 ID（可为 0 表示未关联）
    pub software_id: i64,
    /// 缓存文件名
    pub filename: String,
    /// 软件包名称
    pub name: String,
    /// 版本 epoch 号
    pub epoch: i64,
    /// 版本号
    pub version: String,
    /// 包发布号（如 pkgrel）
    pub pkgrel: String,
    /// 目标架构（如 x86_64）
    pub arch: String,
    /// 文件大小（字节）
    pub size: i64,
    /// 来源缓存目录名称
    pub source_dir: Option<String>,
    /// 缓存文件所在目录路径
    pub cache_directory: String,
}
