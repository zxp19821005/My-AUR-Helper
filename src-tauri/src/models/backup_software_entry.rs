use serde::{Deserialize, Serialize};

/// 备份软件包列表展示条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSoftwareEntry {
    pub id: i64,
    pub pkgname: String,
    pub filename: String,
    pub epoch: i64,
    pub pkgver: String,
    pub pkgrel: String,
    pub arch: String,
    pub subdirectory: Option<String>,
    pub full_path: String,
    /// 记录创建时间
    #[serde(default)]
    pub created_at: Option<String>,
    /// 记录更新时间
    #[serde(default)]
    pub updated_at: Option<String>,
}
