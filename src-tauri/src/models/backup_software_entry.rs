use serde::{Deserialize, Serialize};

/// 备份软件包列表展示条目（含软件包名称）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSoftwareEntry {
    pub id: i64,
    pub software_id: Option<i64>,
    pub pkgname: String,
    pub filename: String,
    pub pkgver: String,
    pub epoch: i64,
    pub pkgrel: String,
    pub arch: String,
    pub subdirectory: Option<String>,
    pub full_path: String,
}
