use anyhow::Result;           // 通用错误处理
use regex::Regex;              // 正则表达式，用于解析 PKGBUILD 变量
use std::path::Path;           // 文件路径操作
use tokio::fs;                 // 异步文件系统操作
use log::info;                 // 日志记录

use crate::models::{CheckerType, PackageType, SoftwareInfo}; // 项目数据模型

/// 读取 PKGBUILD 文件并解析为软件包信息
/// @param path - PKGBUILD 所在目录的路径
/// @returns 解析结果：(SoftwareInfo 结构体, 可选的上游 URL)
///          如果目录中不存在 PKGBUILD 文件则返回 None
pub async fn read_pkgbuild(path: &Path) -> Result<Option<(SoftwareInfo, Option<String>)>> {
    let pkgbuild_path = path.join("PKGBUILD");
    if !pkgbuild_path.exists() {
        return Ok(None); // 没有 PKGBUILD 文件，跳过
    }
    let content = fs::read_to_string(&pkgbuild_path).await?; // 读取 PKGBUILD 文件内容
    let (sw, upstream_url) = parse_pkgbuild(&content, path)?; // 解析内容
    Ok(Some((sw, upstream_url)))
}

/// 解析 PKGBUILD 内容，提取包名、描述、URL 和检查器类型
/// @param content - PKGBUILD 文件的文本内容
/// @param path - 包目录路径（用于在无 pkgname 时作为包名）
/// @returns (SoftwareInfo 结构体, 可选的上游 URL)
fn parse_pkgbuild(content: &str, path: &Path) -> Result<(SoftwareInfo, Option<String>)> {
    // 预编译正则表达式，匹配 PKGBUILD 中的变量赋值
    let re_pkgname = Regex::new(r"^pkgname=([a-zA-Z0-9@._+-]+)").unwrap();
    let re_pkgver = Regex::new(r"^pkgver=(.+)").unwrap();
    let re_url = Regex::new(r#"^url="([^"]*)"#).unwrap();
    let re_ghurl = Regex::new(r#"^_ghurl="([^"]*)"#).unwrap();
    let re_giteeurl = Regex::new(r#"^_giteeurl="([^"]*)"#).unwrap();
    let re_gitlaburl = Regex::new(r#"^_gitlaburl="([^"]*)"#).unwrap();
    let re_dlurl = Regex::new(r#"^_dlurl="([^"]*)"#).unwrap();
    // 匹配 source 数组中的 GitHub URL
    let re_source_gh = Regex::new(r#"github\.com/([^/]+/[^/]+)"#).unwrap();

    let mut pkgname = String::new();
    let mut pkgver = String::new();
    let mut url = None;
    let mut upstream_url = None;
    let mut in_source = false;
    let mut checker_type = CheckerType::GitHubRelease; // 默认 GitHub 检查器

    // 逐行解析 PKGBUILD
    for line in content.lines() {
        let trimmed = line.trim();

        // 处理 source 数组中的 URL
        if in_source {
            if let Some(cap) = re_source_gh.captures(trimmed) {
                let gh_url = format!("https://github.com/{}", &cap[1]);
                if upstream_url.is_none() {
                    upstream_url = Some(gh_url);
                }
            }
            if trimmed.contains(')') || (!trimmed.ends_with('\\') && !trimmed.ends_with('"')) {
                in_source = false;
            }
        }

        if let Some(cap) = re_pkgname.captures(trimmed) {
            pkgname = cap[1].to_string();
        } else if let Some(cap) = re_pkgver.captures(trimmed) {
            pkgver = cap[1].trim().to_string();
        } else if let Some(cap) = re_url.captures(trimmed) {
            url = Some(cap[1].to_string());
        } else if let Some(cap) = re_ghurl.captures(trimmed) {
            upstream_url = Some(cap[1].to_string());
        } else if let Some(cap) = re_giteeurl.captures(trimmed) {
            upstream_url = Some(cap[1].to_string());
            checker_type = CheckerType::Gitee;
        } else if let Some(cap) = re_gitlaburl.captures(trimmed) {
            upstream_url = Some(cap[1].to_string());
            checker_type = CheckerType::GitLab;
        } else if let Some(cap) = re_dlurl.captures(trimmed) {
            if upstream_url.is_none() {
                upstream_url = Some(cap[1].to_string());
            }
        } else if trimmed.starts_with("source=") || trimmed.starts_with("source=(") {
            // 开始 source 数组
            if let Some(cap) = re_source_gh.captures(trimmed) {
                let gh_url = format!("https://github.com/{}", &cap[1]);
                if upstream_url.is_none() {
                    upstream_url = Some(gh_url);
                }
            }
            if !trimmed.contains(')') {
                in_source = true;
            }
        }
    }

    // 根据主页 URL 推断检查器类型（如果尚未确定）
    if upstream_url.is_none() {
        if let Some(ref u) = url {
            upstream_url = Some(u.clone());
        }
    }
    if let Some(ref u) = upstream_url {
        if u.contains("github.com") {
            checker_type = CheckerType::GitHubRelease;
        } else if u.contains("gitee.com") {
            checker_type = CheckerType::Gitee;
        } else if u.contains("gitlab.com") {
            checker_type = CheckerType::GitLab;
        }
    }

    // 如果 PKGBUILD 中没有定义 pkgname，使用目录名作为包名
    let pkgname_final = if pkgname.is_empty() {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    } else {
        pkgname
    };

    // 根据包名后缀判断软件类型
    let package_type_id = if pkgname_final.ends_with("-bin") {
        PackageType::Binary
    } else if pkgname_final.ends_with("-git") {
        PackageType::Git
    } else if pkgname_final.ends_with("-appimage") {
        PackageType::AppImage
    } else {
        PackageType::Compiled
    };

    // 根据版本号判断是否为测试版本
    let version_lower = pkgver.to_lowercase();
    let check_test_versions = version_lower.contains("beta")
        || version_lower.contains("alpha")
        || version_lower.contains("rc")
        || version_lower.contains("dev")
        || version_lower.contains("pre");

    // -bin 包默认检查二进制文件
    let check_binary_files = pkgname_final.ends_with("-bin");

    // 构建 SoftwareInfo 结构体
    let sw = SoftwareInfo {
        software_id: None,
        pkgname: pkgname_final,
        upstream_url,
        package_type_id,
        checker_type_id: checker_type,
        is_outdated: false,
        check_test_versions,
        check_binary_files,
        auto_check_enabled: false,
        license_id: None,
        language_id: None,
    };

    Ok((sw, None))
}

/// 从本地目录同步包信息
/// 遍历指定目录下的所有子目录，读取每个子目录中的 PKGBUILD
/// @param pkgs_dir - 存放 AUR 包目录的父目录路径
/// @param pkgname - 可选，指定包名时只同步该包
/// @returns 解析得到的所有软件包信息列表
pub async fn sync_from_local_files(pkgs_dir: &Path, pkgname: Option<&str>) -> Result<Vec<SoftwareInfo>> {
    let mut packages = Vec::new();
    let mut entries = fs::read_dir(pkgs_dir).await?; // 读取目录内容
    // 遍历每个子目录
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {           // 只处理目录
            // 如果指定了包名，只处理匹配的目录
            if let Some(filter_name) = pkgname {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if dir_name != filter_name {
                    continue;
                }
            }
            if let Some((sw, _)) = read_pkgbuild(&entry.path()).await? {
                packages.push(sw);                       // 收集解析结果
            }
        }
    }
    info!("已从本地文件同步 {} 个软件包", packages.len());
    Ok(packages)
}
