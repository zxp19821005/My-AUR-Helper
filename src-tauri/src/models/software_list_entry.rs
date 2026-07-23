use serde::{Deserialize, Serialize};

use super::{CheckerType, PackageType, UpstreamUrlStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareListEntry {
    pub software_id: i64,
    pub pkgname: String,
    pub package_type_id: PackageType,
    pub checker_type_id: CheckerType,
    pub is_outdated: bool,
    pub aur_version: Option<String>,
    pub aur_last_updated: Option<i64>,
    pub upstream_version: Option<String>,
    pub upstream_last_checked: Option<i64>,
    pub upstream_url: Option<String>,
    pub upstream_url_status: Option<UpstreamUrlStatus>,
    pub upstream_license_id: Option<String>,
}
