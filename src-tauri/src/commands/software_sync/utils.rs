/**
 * utils.rs - 软件包同步工具函数和类型定义
 *
 * 本模块提供软件包同步相关的通用工具函数和数据结构：
 * - AurSyncResult: AUR 同步结果结构体，用于内存缓冲
 * - UpstreamCheckResult: 上游版本检查结果结构体，用于内存缓冲
 * - get_setting_opt: 安全获取数据库设置项（返回空字符串时视为未设置）
 * - parse_u64/parse_u32: 带默认值的数字解析
 * - detect_package_defaults: 根据软件包名推断包类型和检查器类型
 * - build_checker_settings: 从数据库构建检查器设置（Token 等）
 */
use serde::{Deserialize, Serialize};

use crate::models::{CheckerType, PackageType};

/// AUR 同步结果（用于内存缓冲）
///
/// 在批量同步 AUR 信息时，先将结果收集到此结构体中，
/// 最后统一批量写入数据库，减少数据库锁竞争。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AurSyncResult {
    /// 软件包名称
    pub pkgname: String,
    /// 软件包数据库 ID
    pub software_id: i64,
    /// 软件包描述
    pub desc: Option<String>,
    /// AUR 版本号
    pub version: Option<String>,
    /// 上游 URL
    pub url: Option<String>,
    /// 最后修改时间（Unix 时间戳）
    pub last_modified: Option<i64>,
    /// 许可证 SPDX ID
    pub license_spdx: Option<String>,
    /// 运行时依赖（JSON 数组字符串）
    pub depends: Option<String>,
    /// 构建时依赖（JSON 数组字符串）
    pub makedepends: Option<String>,
    /// 可选依赖（JSON 数组字符串）
    pub optdepends: Option<String>,
    /// 是否标记为过期
    pub out_of_date: Option<bool>,
    /// 推断的包类型
    pub package_type: PackageType,
    /// 推断的检查器类型
    pub checker_type: CheckerType,
    /// 是否检查测试版本
    pub check_test_versions: bool,
    /// 是否检查二进制文件
    pub check_binary_files: bool,
    /// 是否需要更新软件包记录
    pub need_update_software: bool,
}

/// 上游版本检查结果（用于内存缓冲）
///
/// 在批量检查上游版本时，先将结果收集到此结构体中，
/// 最后统一批量写入数据库，减少数据库锁竞争。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamCheckResult {
    /// 软件包名称
    pub pkgname: String,
    /// 软件包数据库 ID
    pub software_id: i64,
    /// 检测到的上游版本号
    pub upstream_version: String,
    /// 是否已过期（AUR 版本 < 上游版本）
    pub is_outdated: bool,
    /// License 列表（JSON 数组字符串）
    pub license_spdx_id: Option<String>,
    /// 编程语言名称列表
    pub language_names: Vec<String>,
}

/// 安全获取数据库设置项
///
/// 如果设置项不存在或值为空字符串，则返回 None。
///
/// # 参数
/// - `db`: 数据库连接
/// - `key`: 设置项键名
///
/// # 返回
/// - `Some(value)`: 设置项存在且非空
/// - `None`: 设置项不存在或为空
pub fn get_setting_opt(db: &crate::db::Database, key: &str) -> Option<String> {
    db.get_setting(key)
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
}

/// 解析 u64 数字，失败时返回默认值
///
/// # 参数
/// - `val`: 要解析的字符串
/// - `default`: 解析失败时的默认值
pub fn parse_u64(val: &str, default: u64) -> u64 {
    val.parse().unwrap_or(default)
}

/// 解析 u32 数字，失败时返回默认值
///
/// # 参数
/// - `val`: 要解析的字符串
/// - `default`: 解析失败时的默认值
pub fn parse_u32(val: &str, default: u32) -> u32 {
    val.parse().unwrap_or(default)
}

/// 根据软件包名称推断包类型和检查器类型
///
/// 推断规则：
/// - 以 `-git` 结尾：Git 包，使用 GitHubAPI 检查器，检查测试版本
/// - 以 `-bin` 结尾：二进制包，使用 GitHubAPI 检查器，检查二进制文件
/// - 以 `-appimage` 结尾：AppImage 包，使用 GitHubAPI 检查器，检查二进制文件
/// - 其他：编译包，使用 GitHubTags 检查器
///
/// # 参数
/// - `pkgname`: 软件包名称
///
/// # 返回
/// - `(PackageType, CheckerType, check_test_versions, check_binary_files)`
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

/// 从数据库构建检查器设置
///
/// 读取数据库中的 GitHub、Gitee、GitLab Token，
/// 构建 CheckerSettings 结构体供检查器使用。
///
/// # 参数
/// - `db`: 数据库连接
///
/// # 返回
/// - `CheckerSettings`: 包含各平台 Token 的设置结构体
pub fn build_checker_settings(db: &crate::db::Database) -> crate::checkers::CheckerSettings {
    crate::checkers::CheckerSettings {
        github_token: get_setting_opt(db, "github_token"),
        gitee_token: get_setting_opt(db, "gitee_token"),
        gitlab_token: get_setting_opt(db, "gitlab_token"),
    }
}