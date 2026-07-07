use serde::{Deserialize, Serialize};

use super::{CheckerType, PackageType};

/// 软件包详情（基本信息 + AUR + 上游）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareDetail {
    pub software_id: Option<i64>,
    pub pkgname: String,
    pub upstream_url: Option<String>,
    pub package_type_id: PackageType,
    pub checker_type_id: CheckerType,
    pub is_outdated: bool,
    pub check_test_versions: bool,
    pub check_binary_files: bool,
    pub auto_check_enabled: bool,
    pub license_id: Option<i64>,
    pub language_id: Option<i64>,
    pub version_extract_regex: Option<String>,
    pub aur_version: Option<String>,
    pub aur_last_updated: Option<i64>,
    pub aur_pkgdesc: Option<String>,
    pub upstream_version: Option<String>,
    pub upstream_last_checked: Option<String>,
}
