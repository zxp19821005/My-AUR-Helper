/**
 * binary_check.rs - GitHub Release 二进制文件检查工具
 *
 * 功能：
 * - 检查 release 的 assets 是否包含 Linux 二进制文件
 * - 记录资产检查日志
 *
 * 平台判断规则：
 * - 文件名包含 darwin/macos/windows 视为非 Linux
 * - 其余视为 Linux 文件
 */
use log::{info, warn};

use crate::checkers::utils::extract_version_with_regex;

/// 判断文件名是否明显是非 Linux 平台
fn is_not_linux_platform(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("darwin") || lower.contains("macos") || lower.contains("windows")
}

/// 检查 release 的 assets 是否包含 Linux 二进制文件
///
/// # 参数
/// - `assets`: release 的资产文件列表
/// - `asset_filter`: 资产文件名过滤器（可选），使用正则表达式匹配
///
/// # 返回
/// - `true`: 存在匹配的 Linux 二进制文件
/// - `false`: 不存在匹配的 Linux 二进制文件
pub fn has_linux_binary(assets: &[serde_json::Value], asset_filter: Option<&str>) -> bool {
    if let Some(filter) = asset_filter {
        if let Ok(re) = regex::Regex::new(filter) {
            return assets.iter().any(|a| {
                if let Some(name) = a["name"].as_str() {
                    !is_not_linux_platform(name) && re.is_match(name)
                } else {
                    false
                }
            });
        }
    }

    assets.iter().any(|a| {
        a["name"]
            .as_str()
            .is_some_and(|n| !is_not_linux_platform(n))
    })
}

/// 从匹配资产过滤器的文件名中提取版本号
///
/// 当 release 的 tag 本身不是版本号（如 "latest"、"continuous"）时，
/// 真实版本号往往包含在资产文件名中（如 `csBooks-9.0.0.pacman`）。
/// 此函数遍历资产，找到首个匹配 `asset_filter` 的 Linux 平台文件名，
/// 并用同一正则提取其中的捕获组作为版本号。
///
/// # 参数
/// - `assets`: release 的资产文件列表
/// - `asset_filter`: 资产文件名过滤器正则表达式（同时用于匹配与提取）
///
/// # 返回
/// - `Some(version)`: 找到匹配资产并成功提取版本号
/// - `None`: 未找到匹配资产或无法提取版本
pub fn extract_version_from_assets(
    assets: &[serde_json::Value],
    asset_filter: &str,
) -> Option<String> {
    if let Ok(re) = regex::Regex::new(asset_filter) {
        for asset in assets {
            if let Some(name) = asset["name"].as_str() {
                if !is_not_linux_platform(name) && re.is_match(name) {
                    if let Some(version) = extract_version_with_regex(name, asset_filter) {
                        return Some(version);
                    }
                }
            }
        }
    }
    None
}

/// 检查并打印 release 资产的详细信息
///
/// # 参数
/// - `data`: release 的 JSON 数据
/// - `pkgname`: 软件包名称（用于日志）
/// - `asset_filter`: 资产文件名过滤器（可选）
pub fn check_release_assets(data: &serde_json::Value, pkgname: &str, asset_filter: Option<&str>) {
    let assets = data["assets"].as_array();
    if let Some(list) = assets {
        if list.is_empty() {
            warn!("[二进制检查] {}: Release 无任何附件", pkgname);
        } else if !has_linux_binary(list, asset_filter) {
            let names: Vec<&str> = list.iter().filter_map(|a| a["name"].as_str()).collect();
            warn!(
                "[二进制检查] {}: Release 附件中未找到 Linux 二进制文件: {:?}",
                pkgname, names
            );
        } else {
            let linux_assets: Vec<&str> = list
                .iter()
                .filter_map(|a| {
                    let name = a["name"].as_str()?;
                    if !is_not_linux_platform(name) {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect();
            info!(
                "[二进制检查] {}: 找到 Linux 二进制文件: {:?}",
                pkgname, linux_assets
            );
        }
    }
}
