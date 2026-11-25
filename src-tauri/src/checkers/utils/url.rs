/**
 * checkers/utils/url.rs - 仓库 URL 解析与 URL/HTML 版本提取
 *
 * 功能：
 * - 从 GitHub/GitLab/Gitee 仓库 URL（HTTP(S) 与 git@ SSH）提取 owner/repo
 * - 从下载 URL 与 HTML 页面文本中提取版本号
 *
 * 工作流程：
 * 1. extract_owner_repo 先处理 SSH 冒号格式，再按域名段定位 owner/repo
 * 2. extract_version_from_url / extract_version_from_html 复用 regex 子模块的
 *    静态正则集合，并对 URL 结果剥离文件扩展名
 *
 * 适用场景：GitHub/GitLab/Gitee/HTTP/浏览器检查器，对外经 utils barrel 暴露路径不变。
 */
use super::regex::static_regexes;
use crate::checkers::version_extract_clean::strip_file_extensions;

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
