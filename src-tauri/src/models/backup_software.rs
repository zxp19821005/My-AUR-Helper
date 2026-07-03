use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 备份软件包信息
/// 对应数据库 backup_software 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSoftware {
    /// 备份记录 ID，数据库自增主键
    pub id: Option<i64>,
    /// 关联的软件包 ID
    pub software_id: i64,
    /// 备份文件名
    pub filename: String,
    /// 版本 epoch 号
    pub epoch: i64,
    /// 包发布号（如 pkgrel）
    pub pkgrel: String,
    /// 目标架构（如 x86_64）
    pub arch: String,
    /// 备份存放的子目录（可选）
    pub subdirectory: Option<String>,
}
