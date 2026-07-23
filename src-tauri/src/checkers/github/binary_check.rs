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
