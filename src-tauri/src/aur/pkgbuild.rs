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
    let re_pkgdesc = Regex::new(r#"^pkgdesc="([^"]*)"#).unwrap();
    let re_url = Regex::new(r#"^url="([^"]*)"#).unwrap();
    let re_ghurl = Regex::new(r#"^_ghurl="([^"]*)"#).unwrap();     // GitHub URL 自定义变量
    let re_giteeurl = Regex::new(r#"^_giteeurl="([^"]*)"#).unwrap(); // Gitee URL 自定义变量
    let re_gitlaburl = Regex::new(r#"^_gitlaburl="([^"]*)"#).unwrap(); // GitLab URL 自定义变量
    let re_dlurl = Regex::new(r#"^_dlurl="([^"]*)"#).unwrap();     // 下载 URL 自定义变量

    let mut pkgname = String::new();
    let mut _pkgdesc = None;        // 包描述（当前未使用）
    let mut url = None;             // 项目主页 URL
    let mut upstream_url = None;    // 上游版本检查 URL
    let mut checker_type = CheckerType::Manual; // 默认为手动检查

    // 逐行解析 PKGBUILD
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(cap) = re_pkgname.captures(trimmed) {
            pkgname = cap[1].to_string();                        // 提取 pkgname
        } else if let Some(cap) = re_pkgdesc.captures(trimmed) {
            _pkgdesc = Some(cap[1].to_string());                 // 提取 pkgdesc
        } else if let Some(cap) = re_url.captures(trimmed) {
            let u = cap[1].to_string();
            url = Some(u.clone());
            // 如果 URL 包含 github.com，自动设为 GitHub Release 检查器
            if u.contains("github.com") {
                checker_type = CheckerType::GitHubRelease;
            }
        } else if let Some(cap) = re_ghurl.captures(trimmed) {
            upstream_url = Some(cap[1].to_string());             // 提取自定义 GitHub URL
            checker_type = CheckerType::GitHubRelease;
        } else if let Some(cap) = re_giteeurl.captures(trimmed) {
            upstream_url = Some(cap[1].to_string());             // 提取自定义 Gitee URL
            checker_type = CheckerType::Gitee;
        } else if let Some(cap) = re_gitlaburl.captures(trimmed) {
            upstream_url = Some(cap[1].to_string());             // 提取自定义 GitLab URL
            checker_type = CheckerType::GitLab;
        } else if let Some(cap) = re_dlurl.captures(trimmed) {
            upstream_url = Some(cap[1].to_string());             // 提取自定义下载 URL
            if checker_type == CheckerType::Manual {
                checker_type = CheckerType::Redirect;            // 仅当尚未识别时设为重定向检查器
            }
        }
    }

    // 根据主页 URL 进一步推断检查器类型（如果尚未确定）
    if let Some(ref u) = url {
        if u.contains("gitee.com") && checker_type == CheckerType::Manual {
            checker_type = CheckerType::Gitee;
        }
        if u.contains("gitlab.com") && checker_type == CheckerType::Manual {
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

    // 构建 SoftwareInfo 结构体
    let sw = SoftwareInfo {
        software_id: None,                       // 新记录，尚无 ID
        pkgname: pkgname_final,
        upstream_url: upstream_url.clone().or(url), // 优先使用上游 URL，否则用主页 URL
        package_type: PackageType::Compiled,     // AUR 包默认为编译类型
        checker_type,
        is_outdated: false,                      // 初始设为未过期
        check_test_versions: false,
        check_binary_files: false,
        auto_check_enabled: true,                // 默认启用自动检查
        license_id: None,
        language_id: None,
        created_at: chrono::Utc::now().timestamp(), // 当前时间戳
    };

    Ok((sw, upstream_url))
}

/// 从本地目录同步包信息
/// 遍历指定目录下的所有子目录，读取每个子目录中的 PKGBUILD
/// @param pkgs_dir - 存放 AUR 包目录的父目录路径
/// @returns 解析得到的所有软件包信息列表
pub async fn sync_from_local_files(pkgs_dir: &Path) -> Result<Vec<SoftwareInfo>> {
    let mut packages = Vec::new();
    let mut entries = fs::read_dir(pkgs_dir).await?; // 读取目录内容
    // 遍历每个子目录
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {           // 只处理目录
            if let Some((sw, _)) = read_pkgbuild(&entry.path()).await? {
                packages.push(sw);                       // 收集解析结果
            }
        }
    }
    info!("Synced {} packages from local files", packages.len());
    Ok(packages)
}
