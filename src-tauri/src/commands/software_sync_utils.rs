use serde::{Deserialize, Serialize};

use crate::models::{CheckerType, PackageType};

/// AUR 同步结果（用于内存缓冲）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurSyncResult {
    pub pkgname: String,
    pub software_id: i64,
    pub desc: Option<String>,
    pub version: Option<String>,
    pub url: Option<String>,
    pub last_modified: Option<i64>,
    pub license_spdx: Option<String>,
    pub depends: Option<String>,
    pub makedepends: Option<String>,
    pub optdepends: Option<String>,
    pub out_of_date: Option<bool>,
    pub package_type: PackageType,
    pub checker_type: CheckerType,
    pub check_test_versions: bool,
    pub check_binary_files: bool,
    pub need_update_software: bool,
}

/// 上游检查结果（用于内存缓冲）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamCheckResult {
    pub pkgname: String,
    pub software_id: i64,
    pub upstream_version: String,
    pub is_outdated: bool,
}

pub fn get_setting_opt(db: &crate::db::Database, key: &str) -> Option<String> {
    db.get_setting(key)
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
}

pub fn parse_u64(val: &str, default: u64) -> u64 {
    val.parse().unwrap_or(default)
}

pub fn parse_u32(val: &str, default: u32) -> u32 {
    val.parse().unwrap_or(default)
}

pub fn detect_package_defaults(pkgname: &str) -> (PackageType, CheckerType, bool, bool) {
    if pkgname.ends_with("-git") {
        (PackageType::Git, CheckerType::GitHubAPI, true, false)
    } else if pkgname.ends_with("-bin") {
        (PackageType::Binary, CheckerType::GitHubAPI, false, true)
    } else if pkgname.ends_with("-appimage") {
        (PackageType::AppImage, CheckerType::GitHubAPI, false, true)
    } else {
        (PackageType::Compiled, CheckerType::GitHubTags, false, false)
    }
}

pub fn build_checker_settings(db: &crate::db::Database) -> crate::checkers::CheckerSettings {
    crate::checkers::CheckerSettings {
        github_token: get_setting_opt(db, "github_token"),
        gitee_token: get_setting_opt(db, "gitee_token"),
        gitlab_token: get_setting_opt(db, "gitlab_token"),
    }
}