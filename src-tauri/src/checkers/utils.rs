/**
 * utils.rs - 版本检查器通用工具函数
 *
 * 提供版本提取（正则 / URL）、owner/repo 解析、版本清洗等被各检查器
 * 复用的纯函数。
 */
use log::debug;
use std::sync::OnceLock;

/// 版本提取所需的静态正则集合
/// 进程级惰性编译一次，避免每次提取版本都重新编译
struct StaticRegexes {
    clean: regex::Regex,
    url_v: regex::Regex,
    url_num: regex::Regex,
    html_kw: regex::Regex,
    html_td: regex::Regex,
}

/// 获取版本提取静态正则集合单例
fn static_regexes() -> &'static StaticRegexes {
    static RE: OnceLock<StaticRegexes> = OnceLock::new();
    RE.get_or_init(|| StaticRegexes {
        clean: regex::Regex::new(r"^[^0-9]*?(v?\d\S*)$").expect("正则 clean 编译失败"),
        url_v: regex::Regex::new(r"[/-]v?(\d+\.\d+\.\d+[a-zA-Z0-9._+-]*)")
            .expect("正则 url_v 编译失败"),
        url_num: regex::Regex::new(r"[/-](\d+\.\d+\.\d+[a-zA-Z0-9._+-]*)")
            .expect("正则 url_num 编译失败"),
        html_kw: regex::Regex::new(
            r"(?i)(?:version|release|ver\.?|版本)[:\s]+v?(\d+\.\d+\.\d+[a-zA-Z0-9._-]*)",
        )
        .expect("正则 html_kw 编译失败"),
        html_td: regex::Regex::new(
            r"(?i)(?:v)?(\d+\.\d+\.\d+[a-zA-Z0-9._-]*)(?:\s*</[aA]>)?\s*</[tT][dD]>\s*<[tT][dD]",
        )
        .expect("正则 html_td 编译失败"),
    })
}

/// 使用自定义正则表达式从文本中提取版本号
/// @param text - 包含版本号的文本
/// @param regex_pattern - 正则表达式模式
/// @returns 提取到的版本号，如果匹配失败或正则无效则返回 None
pub fn extract_version_with_regex(text: &str, regex_pattern: &str) -> Option<String> {
    debug!("[正则提取] 使用正则表达式: {}", regex_pattern);
    match regex::Regex::new(regex_pattern) {
        Ok(re) => {
            if let Some(caps) = re.captures(text) {
                let version = if caps.len() > 1 {
                    caps[1].to_string()
                } else {
                    caps[0].to_string()
                };
                debug!("[正则提取] 提取成功: {}", version);
                Some(version)
            } else {
                debug!("[正则提取] 未匹配到任何内容");
                None
            }
        }
        Err(e) => {
            debug!("[正则提取] 正则表达式无效: {}", e);
            None
        }
    }
}

/// 从 GitHub/GitLab/Gitee 仓库 URL 提取 owner 和 repo
///
/// 支持的 URL 格式：
/// - https://github.com/owner/repo
/// - https://github.com/owner/repo/releases/tag/v1.0
/// - https://gitlab.com/owner/repo
/// - https://gitee.com/owner/repo
/// - git@github.com:owner/repo.git
///
/// @param repo_url - 仓库 URL
/// @returns (owner, repo) 元组，如果无法解析则返回 None
pub fn extract_owner_repo(repo_url: &str) -> Option<(String, String)> {
    let url = repo_url.trim_end_matches('/').trim_end_matches(".git");

    // 处理 SSH 格式: git@github.com:owner/repo.git
    if let Some(colon_pos) = url.find(':') {
        if url.starts_with("git@") {
            let after_colon = &url[colon_pos + 1..];
            let parts: Vec<&str> = after_colon.split('/').collect();
            if !parts.is_empty() {
                let first = parts[0];
                if let Some(slash_pos) = first.find('/') {
                    let owner = first[..slash_pos].to_string();
                    let repo = first[slash_pos + 1..].to_string();
                    return Some((owner, repo));
                }
            }
        }
    }

    // 处理 HTTP(S) 格式: 按 / 分割，找到域名后的前两段作为 owner/repo
    let parts: Vec<&str> = url.split('/').collect();

    // 查找域名位置（包含 . 的段即为域名）
    let domain_idx = parts.iter().position(|p| p.contains('.'))?;

    // owner 和 repo 紧跟在域名之后
    let owner_idx = domain_idx.checked_add(1)?;
    let repo_idx = domain_idx.checked_add(2)?;

    if owner_idx < parts.len() && repo_idx < parts.len() {
        Some((parts[owner_idx].to_string(), parts[repo_idx].to_string()))
    } else {
        None
    }
}

/// 清理版本号：提取纯版本号部分
///
/// 支持以下格式：
/// - `v1.2.3` / `V1.2.3` -> `1.2.3`
/// - `appname-v1.2.3` -> `1.2.3`
/// - `appname-1.2.3` -> `1.2.3`
/// - `1.2.3` -> `1.2.3`
/// - `continuous` / `latest` -> 保持原样（无数字的 tag）
///
/// @param ver - 原始版本字符串
/// @returns 清理后的版本号，如果无法提取版本则返回原始字符串
pub fn clean_version(ver: &str) -> String {
    // 尝试匹配 appname-vX.Y.Z 或 appname-X.Y.Z 模式
    // 复用惰性编译的静态正则提取 v 或数字开头的版本部分
    let re = &static_regexes().clean;
    if let Some(cap) = re.captures(ver) {
        if let Some(version) = cap.get(1) {
            let version_str = version.as_str();
            // 去除开头的 v/V 前缀
            return version_str
                .trim_start_matches('v')
                .trim_start_matches('V')
                .to_string();
        }
    }
    // 回退：只去除开头的 v/V 前缀
    ver.trim_start_matches('v')
        .trim_start_matches('V')
        .to_string()
}

/// 从 URL 中提取版本号
/// 匹配模式如 /v1.2.3 或 /1.2.3
/// @param url - 包含版本号的 URL 字符串
/// @returns 提取到的版本号（已去除常见文件扩展名）
pub fn extract_version_from_url(url: &str) -> Option<String> {
    let re = &static_regexes();
    // 尝试匹配带 v 前缀的版本号
    if let Some(cap) = re.url_v.captures(url) {
        return Some(strip_file_extensions(&cap[1]));
    }
    // 尝试匹配不带 v 前缀的版本号
    if let Some(cap) = re.url_num.captures(url) {
        return Some(strip_file_extensions(&cap[1]));
    }
    None
}

/// 去除常见的文件扩展名
/// @param version - 包含可能扩展名的版本字符串
/// @returns 去除扩展名后的版本字符串
fn strip_file_extensions(version: &str) -> String {
    let extensions = [
        ".AppImage",
        ".appimage",
        ".flatpak",
        ".deb",
        ".rpm",
        ".exe",
        ".msi",
        ".dmg",
        ".pkg",
        ".tar.gz",
        ".tar.xz",
        ".zip",
        ".tar.bz2",
        ".tar.zst",
        ".7z",
        ".snap",
        ".AppImage.zsync",
    ];

    let mut result = version.to_string();
    for ext in &extensions {
        if result.ends_with(ext) {
            result = result[..result.len() - ext.len()].to_string();
            break;
        }
    }

    // 去除架构后缀 (如 .x86_64, .aarch64, .arm64, .amd64, .i386 等)
    let arch_patterns = [
        ".x86_64",
        ".aarch64",
        ".arm64",
        ".amd64",
        ".i386",
        ".i686",
        ".armv7l",
        ".armhf",
        ".noarch",
        ".all",
        ".universal",
    ];
    for arch in &arch_patterns {
        if result.ends_with(arch) {
            result = result[..result.len() - arch.len()].to_string();
            break;
        }
    }

    result
}

/// 从 HTML 内容中提取版本号
/// 匹配常见的版本信息模式：如 "Version: 1.2.3" 或表格中的版本号
/// @param body - HTML 页面文本内容
/// @returns 提取到的版本号
pub fn extract_version_from_html(body: &str) -> Option<String> {
    let re = static_regexes();
    // 模式1：匹配 "version" / "release" / "版本" 等关键词后的版本号
    if let Some(cap) = re.html_kw.captures(body) {
        return Some(cap[1].to_string());
    }
    // 模式2：匹配 HTML 表格中 <td> 标签内的版本号
    if let Some(cap) = re.html_td.captures(body) {
        return Some(cap[1].to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// clean_version 必须保留 prerelease/build 元数据，否则 GitHub GraphQL 批量
    /// 检查会把 v1.0.0-alpha.16 错误地截断为 1.0.0，导致版本比较失真。
    /// 真实 bug 复现：deskaide/deskaide 仓库全部 tag 都是 v1.0.0-alpha.X，
    /// 旧正则 `[^0-9]*?(v?\d[\d.]*)` 把它们全部截为 1.0.0，max_by_vercmp 比较
    /// 全部返回 Equal，最终返回 1.0.0（错的），用户 AUR 1.0.0_alpha.16 被认为无需更新。
    #[test]
    fn test_clean_version_preserves_prerelease() {
        assert_eq!(clean_version("v1.0.0-alpha.16"), "1.0.0-alpha.16");
        assert_eq!(clean_version("1.0.0-alpha.16"), "1.0.0-alpha.16");
        assert_eq!(clean_version("v1.0.0-beta.2"), "1.0.0-beta.2");
        assert_eq!(clean_version("v2.0.0-rc.1"), "2.0.0-rc.1");
    }

    #[test]
    fn test_clean_version_strips_v_prefix() {
        assert_eq!(clean_version("v1.0.0"), "1.0.0");
        assert_eq!(clean_version("V1.0.0"), "1.0.0");
        assert_eq!(clean_version("v1.2.3"), "1.2.3");
    }

    #[test]
    fn test_clean_version_appname_prefix() {
        assert_eq!(clean_version("appname-v1.0.0"), "1.0.0");
        assert_eq!(clean_version("my-app-v1.2.3"), "1.2.3");
    }

    #[test]
    fn test_clean_version_keeps_pkgrel() {
        assert_eq!(clean_version("v1.0.0-1"), "1.0.0-1");
    }

    #[test]
    fn test_clean_version_passthrough_non_version() {
        // 看起来不像版本号的字符串（无数字）原样保留
        assert_eq!(clean_version("continuous"), "continuous");
        assert_eq!(clean_version("latest"), "latest");
    }
}
