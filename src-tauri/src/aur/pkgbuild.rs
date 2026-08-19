use crate::errors::AppResult; // 通用错误处理
use log::info;
use regex::Regex; // 正则表达式，用于解析 PKGBUILD 变量
use std::path::Path; // 文件路径操作
use std::sync::OnceLock; // 惰性初始化静态正则集合
use tokio::fs; // 异步文件系统操作 // 日志记录

use crate::models::{CheckerType, PackageType, SoftwareInfo}; // 项目数据模型

/// PKGBUILD 解析所需的正则表达式集合
/// 进程级惰性编译一次，避免每次解析都重新构建 8 个正则
struct PkgRegexes {
    pkgname: Regex,
    pkgver: Regex,
    url: Regex,
    ghurl: Regex,
    giteeurl: Regex,
    gitlaburl: Regex,
    dlurl: Regex,
    source_gh: Regex,
}

/// 获取 PKGBUILD 正则集合单例
fn pkgbuild_regexes() -> &'static PkgRegexes {
    static RE: OnceLock<PkgRegexes> = OnceLock::new();
    RE.get_or_init(|| PkgRegexes {
        pkgname: Regex::new(r"^pkgname=([a-zA-Z0-9@._+-]+)").expect("正则 pkgname 编译失败"),
        pkgver: Regex::new(r"^pkgver=(.+)").expect("正则 pkgver 编译失败"),
        url: Regex::new(r#"^url="([^"]*)"#).expect("正则 url 编译失败"),
        ghurl: Regex::new(r#"^_ghurl="([^"]*)"#).expect("正则 ghurl 编译失败"),
        giteeurl: Regex::new(r#"^_giteeurl="([^"]*)"#).expect("正则 giteeurl 编译失败"),
        gitlaburl: Regex::new(r#"^_gitlaburl="([^"]*)"#).expect("正则 gitlaburl 编译失败"),
        dlurl: Regex::new(r#"^_dlurl="([^"]*)"#).expect("正则 dlurl 编译失败"),
        source_gh: Regex::new(r#"github\.com/([^/]+/[^/]+)"#).expect("正则 source_gh 编译失败"),
    })
}

/// 读取 PKGBUILD 文件并解析为软件包信息
/// @param path - PKGBUILD 所在目录的路径
/// @returns 解析结果：(SoftwareInfo 结构体, 可选的上游 URL)
///          如果目录中不存在 PKGBUILD 文件则返回 None
pub async fn read_pkgbuild(path: &Path) -> AppResult<Option<(SoftwareInfo, Option<String>)>> {
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
fn parse_pkgbuild(content: &str, path: &Path) -> AppResult<(SoftwareInfo, Option<String>)> {
    // 复用进程级惰性编译的正则集合，避免每次解析 PKGBUILD 都重新编译 8 个正则
    let re = pkgbuild_regexes();

    let mut pkgname = String::new();
    let mut pkgver = String::new();
    let mut url = None;
    let mut upstream_url = None;
    let mut in_source = false;
    let mut checker_type = CheckerType::GitHubAPI; // 默认 GitHub API 检查器

    // 逐行解析 PKGBUILD
    for line in content.lines() {
        let trimmed = line.trim();

        // 处理 source 数组中的 URL
        if in_source {
            if let Some(cap) = re.source_gh.captures(trimmed) {
                let gh_url = format!("https://github.com/{}", &cap[1]);
                if upstream_url.is_none() {
                    upstream_url = Some(gh_url);
                }
            }
            if trimmed.contains(')') || (!trimmed.ends_with('\\') && !trimmed.ends_with('"')) {
                in_source = false;
            }
        }

        if let Some(cap) = re.pkgname.captures(trimmed) {
            pkgname = cap[1].to_string();
        } else if let Some(cap) = re.pkgver.captures(trimmed) {
            pkgver = cap[1].trim().to_string();
        } else if let Some(cap) = re.url.captures(trimmed) {
            url = Some(cap[1].to_string());
        } else if let Some(cap) = re.ghurl.captures(trimmed) {
            upstream_url = Some(cap[1].to_string());
        } else if let Some(cap) = re.giteeurl.captures(trimmed) {
            upstream_url = Some(cap[1].to_string());
            checker_type = CheckerType::Gitee;
        } else if let Some(cap) = re.gitlaburl.captures(trimmed) {
            upstream_url = Some(cap[1].to_string());
            checker_type = CheckerType::GitLab;
        } else if let Some(cap) = re.dlurl.captures(trimmed) {
            if upstream_url.is_none() {
                upstream_url = Some(cap[1].to_string());
            }
        } else if trimmed.starts_with("source=") || trimmed.starts_with("source=(") {
            // 开始 source 数组
            if let Some(cap) = re.source_gh.captures(trimmed) {
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
            checker_type = CheckerType::GitHubAPI;
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

    // 根据包名后缀和类型重写检查器类型
    let checker_type = match package_type_id {
        PackageType::Git => CheckerType::GitHubAPI,
        PackageType::Compiled if matches!(checker_type, CheckerType::GitHubAPI) => {
            CheckerType::GitHubTags
        }
        _ => checker_type,
    };

    // 根据版本号判断是否为测试版本
    let version_lower = pkgver.to_lowercase();
    let check_test_versions = version_lower.contains("beta")
        || version_lower.contains("alpha")
        || version_lower.contains("rc")
        || version_lower.contains("dev")
        || version_lower.contains("pre");

    // -bin 和 -appimage 包默认检查二进制文件存在
    let check_binary_files =
        pkgname_final.ends_with("-bin") || pkgname_final.ends_with("-appimage");

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
        language_ids: vec![],
        version_extract_regex: None,
    };

    Ok((sw, None))
}

/// 从本地目录同步包信息
/// 遍历指定目录下的所有子目录，读取每个子目录中的 PKGBUILD
/// @param pkgs_dir - 存放 AUR 包目录的父目录路径
/// @param pkgname - 可选，指定包名时只同步该包
/// @returns 解析得到的所有软件包信息列表
pub async fn sync_from_local_files(
    pkgs_dir: &Path,
    pkgname: Option<&str>,
) -> AppResult<Vec<SoftwareInfo>> {
    let mut packages = Vec::new();
    let mut entries = fs::read_dir(pkgs_dir).await?; // 读取目录内容
                                                     // 遍历每个子目录
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            // 只处理目录
            // 如果指定了包名，只处理匹配的目录
            if let Some(filter_name) = pkgname {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if dir_name != filter_name {
                    continue;
                }
            }
            if let Some((sw, _)) = read_pkgbuild(&entry.path()).await? {
                packages.push(sw); // 收集解析结果
            }
        }
    }
    info!("已从本地文件同步 {} 个软件包", packages.len());
    Ok(packages)
}
