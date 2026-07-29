use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 备份软件包信息
/// 对应数据库 backup_software 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSoftware {
    /// 备份记录 ID，数据库自增主键
    pub id: Option<i64>,
    /// 备份文件名（仅文件名，不含路径）
    pub filename: String,
    /// 版本 epoch 号
    pub epoch: i64,
    /// 软件包版本号（如 1.0.0）
    pub pkgver: String,
    /// 包发布号（如 pkgrel）
    pub pkgrel: String,
    /// 目标架构（如 x86_64）
    pub arch: String,
    /// 备份存放的子目录（可选）
    pub subdirectory: Option<String>,
    /// 完整文件路径（backup_dir/subdir/filename）
    pub full_path: String,
    /// 记录创建时间（数据库默认 datetime('now')，插入时可为 None）
    #[serde(default)]
    pub created_at: Option<String>,
    /// 记录更新时间（数据库默认 datetime('now')，插入时可为 None）
    #[serde(default)]
    pub updated_at: Option<String>,
}
