use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamInfo {
    pub software_id: i64,
    pub upstream_version: Option<String>,
    pub upstream_license_id: Option<String>,
    pub last_checked: Option<i64>,
}