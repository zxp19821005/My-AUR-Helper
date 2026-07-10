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

/// 清理版本号：去除开头的 v 或 V 前缀
/// @param ver - 原始版本字符串（如 "v1.2.3"）
/// @returns 清理后的版本号（如 "1.2.3"）
pub fn clean_version(ver: &str) -> String {
    ver.trim_start_matches('v')
        .trim_start_matches('V')
        .to_string()
}

/// 从 URL 中提取版本号
/// 匹配模式如 /v1.2.3 或 /1.2.3
/// @param url - 包含版本号的 URL 字符串
/// @returns 提取到的版本号
pub fn extract_version_from_url(url: &str) -> Option<String> {
    // 尝试匹配带 v 前缀的版本号
    let re = regex::Regex::new(r"[/-]v?(\d+\.\d+\.\d+[a-zA-Z0-9._+-]*)").ok()?;
    if let Some(cap) = re.captures(url) {
        return Some(cap[1].to_string());
    }
    // 尝试匹配不带 v 前缀的版本号
    let re2 = regex::Regex::new(r"[/-](\d+\.\d+\.\d+[a-zA-Z0-9._+-]*)").ok()?;
    if let Some(cap) = re2.captures(url) {
        return Some(cap[1].to_string());
    }
    None
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
