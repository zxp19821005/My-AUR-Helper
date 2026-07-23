use log::debug;

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
/// @param repo_url - 仓库 URL，如 https://github.com/owner/repo 或 git@github.com:owner/repo.git
/// @returns (owner, repo) 元组，如果无法解析则返回 None
pub fn extract_owner_repo(repo_url: &str) -> Option<(String, String)> {
    // 去除末尾的斜杠和 .git 后缀，然后按 / 分割
    let parts: Vec<&str> = repo_url
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .split('/')
        .collect();
    if parts.len() >= 2 {
        // 倒数第二部分是 owner，最后一部分是 repo
        Some((
            parts[parts.len() - 2].to_string(),
            parts[parts.len() - 1].to_string(),
        ))
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
    // 使用正则表达式提取 v 或数字开头的版本部分
    if let Ok(re) = regex::Regex::new(r"[^0-9]*?(v?\d[\d.]*)") {
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
    // 尝试匹配带 v 前缀的版本号
    let re = regex::Regex::new(r"[/-]v?(\d+\.\d+\.\d+[a-zA-Z0-9._+-]*)").ok()?;
    if let Some(cap) = re.captures(url) {
        return Some(strip_file_extensions(&cap[1]));
    }
    // 尝试匹配不带 v 前缀的版本号
    let re2 = regex::Regex::new(r"[/-](\d+\.\d+\.\d+[a-zA-Z0-9._+-]*)").ok()?;
    if let Some(cap) = re2.captures(url) {
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
    // 模式1：匹配 "version" / "release" 等关键词后的版本号
    let re =
        regex::Regex::new(r"(?i)(?:version|release|ver\.?)[:\s]+v?(\d+\.\d+\.\d+[a-zA-Z0-9._-]*)")
            .ok()?;
    if let Some(cap) = re.captures(body) {
        return Some(cap[1].to_string());
    }
    // 模式2：匹配 HTML 表格中 <td> 标签内的版本号
    let re2 = regex::Regex::new(
        r"(?i)(?:v)?(\d+\.\d+\.\d+[a-zA-Z0-9._-]*)(?:\s*</[aA]>)?\s*</[tT][dD]>\s*<[tT][dD]",
    )
    .ok()?;
    if let Some(cap) = re2.captures(body) {
        return Some(cap[1].to_string());
    }
    None
}
