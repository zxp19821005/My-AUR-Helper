use serde::{Deserialize, Serialize};

/// 缓存软件包列表展示条目
/// 对应前端 CachePackage 接口，用于列表页展示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSoftwareEntry {
    /// 缓存记录 ID
    pub id: i64,
    /// 软件包名称（从 name 字段读取，未填写则从文件名解析）
    pub pkgname: String,
    /// 缓存文件名
    pub filename: String,
    /// 版本 epoch 号
    pub epoch: i64,
    /// 版本号
    pub pkgver: String,
    /// 包发布号（pkgrel）
    pub pkgrel: String,
    /// 目标架构（如 x86_64）
    pub arch: String,
    /// 缓存文件所在目录路径
    pub cache_directory: String,
    /// 完整文件路径（cache_directory/filename）
    #[serde(default)]
    pub full_path: String,
}
