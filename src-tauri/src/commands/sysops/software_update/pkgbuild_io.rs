/**
 * pkgbuild_io.rs - 软件更新模块的本地 PKGBUILD 读写
 *
 * 功能：按包名定位 <aur_packages_dir>/<pkgname>/PKGBUILD，解析 pkgver / pkgrel，
 *      生成只改这两行的新文本与 diff，并在写回前备份原文件。
 *
 * 安全约束：
 * - 包名走白名单校验，拒绝路径分隔符与 `..`，防止目录穿越；
 * - 目标路径必须仍在 AUR 根目录之下（纵深防御）；
 * - 只改 `pkgver=` 与 `pkgrel=`，不触碰 sha256sums / source 等其它字段；
 * - 写回前固定备份为同目录 `PKGBUILD.bak`，写回失败时尝试从备份恢复。
 *
 * 适用场景：软件更新工作台的状态推导、dry-run 预览、批量写回。
 */
use std::path::{Path, PathBuf};

use tokio::fs;

use crate::errors::{AppError, AppResult};

/// 合法包名字符（与 AUR 包名规范一致），并拒绝 `..` 目录片段
fn is_valid_pkgname(pkgname: &str) -> bool {
    !pkgname.is_empty()
        && pkgname.len() <= 128
        && !pkgname.contains("..")
        && pkgname
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '+' | '-'))
}

/// 校验包名，拒绝路径分隔符、空串与目录穿越
///
/// # 参数
/// - `pkgname`: 待校验的软件包名称
///
/// # 返回
/// - `Ok(())`: 包名合法，可安全用于路径拼接
/// - `Err(AppError::InvalidInput)`: 包名非法
pub fn validate_pkgname(pkgname: &str) -> AppResult<()> {
    if !is_valid_pkgname(pkgname) {
        return Err(AppError::InvalidInput(format!("非法包名: {pkgname}")));
    }
    Ok(())
}

/// 拼接 PKGBUILD 路径并确认未越出 AUR 根目录
///
/// # 参数
/// - `root`: `aur_packages_dir` 配置指向的本地 AUR 包根目录
/// - `pkgname`: 软件包名称
///
/// # 返回
/// - `PathBuf`: `<root>/<pkgname>/PKGBUILD` 的绝对路径或相对路径
/// - `Err(AppError::ConfigError)`: AUR 根目录未配置
/// - `Err(AppError::InvalidInput)`: 包名非法或路径越界
pub fn resolve_pkgbuild_path(root: &Path, pkgname: &str) -> AppResult<PathBuf> {
    validate_pkgname(pkgname)?;
    if root.as_os_str().is_empty() {
        return Err(AppError::ConfigError("AUR 文件目录未配置".to_string()));
    }
    let dir = root.join(pkgname);
    let path = dir.join("PKGBUILD");
    if !path.starts_with(root) {
        return Err(AppError::InvalidInput(format!(
            "PKGBUILD 路径越界: {pkgname}"
        )));
    }
    Ok(path)
}

/// 本地 PKGBUILD 的解析结果
#[derive(Debug, Clone)]
pub struct LocalPkgbuild {
    /// 文件路径
    pub path: PathBuf,
    /// 原始文件内容
    pub content: String,
    /// 当前 pkgver（缺失时为空串）
    pub pkgver: String,
    /// 当前 pkgrel（缺失时为空串）
    pub pkgrel: String,
}

/// 去掉版本值两端可能存在的引号
fn unquote(value: &str) -> String {
    value
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string()
}

/// 从文本中逐行提取行首 `pkgver=` / `pkgrel=` 的值（只取首次出现）
///
/// # 参数
/// - `content`: PKGBUILD 原始文本
///
/// # 返回
/// - `(Option<String>, Option<String>)`: (pkgver, pkgrel)，缺失项为 None
pub fn parse_pkgver_pkgrel(content: &str) -> (Option<String>, Option<String>) {
    let mut pkgver = None;
    let mut pkgrel = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("pkgver=") {
            if pkgver.is_none() {
                pkgver = Some(unquote(value));
            }
        } else if let Some(value) = trimmed.strip_prefix("pkgrel=") {
            if pkgrel.is_none() {
                pkgrel = Some(unquote(value));
            }
        }
    }
    (pkgver, pkgrel)
}

/// 读取本地 PKGBUILD 并解析版本字段
///
/// # 参数
/// - `root`: 本地 AUR 包根目录
/// - `pkgname`: 软件包名称
///
/// # 返回
/// - `Ok(Some(LocalPkgbuild))`: 文件存在且已解析
/// - `Ok(None)`: 该包目录下没有 PKGBUILD
/// - `Err(...)`: 包名非法、路径越界或读取失败
pub async fn read_local_pkgbuild(root: &Path, pkgname: &str) -> AppResult<Option<LocalPkgbuild>> {
    let path = resolve_pkgbuild_path(root, pkgname)?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).await?;
    let (pkgver, pkgrel) = parse_pkgver_pkgrel(&content);
    Ok(Some(LocalPkgbuild {
        path,
        content,
        pkgver: pkgver.unwrap_or_default(),
        pkgrel: pkgrel.unwrap_or_default(),
    }))
}

/// 生成改写后的 PKGBUILD 文本与变更摘要（纯函数，不执行任何写操作）
///
/// # 参数
/// - `content`: PKGBUILD 原始文本
/// - `new_pkgver`: 归一化后的目标版本
///
/// # 返回
/// - `(String, String, String, String)`: (新文本, diff 文本, 原 pkgver, 原 pkgrel)
/// - `Err(AppError::ParseError)`: 原文件缺少 `pkgver=` 行
/// - `Err(AppError::InvalidInput)`: 目标版本为空
pub fn build_rewrite(
    content: &str,
    new_pkgver: &str,
) -> AppResult<(String, String, String, String)> {
    let (parsed_pkgver, parsed_pkgrel) = parse_pkgver_pkgrel(content);
    let old_pkgver = parsed_pkgver
        .ok_or_else(|| AppError::ParseError("PKGBUILD 缺少 pkgver= 行".to_string()))?;
    let old_pkgrel = parsed_pkgrel.unwrap_or_else(|| "1".to_string());
    let target = new_pkgver.trim();
    if target.is_empty() {
        return Err(AppError::InvalidInput("目标版本为空".to_string()));
    }

    let had_trailing_newline = content.ends_with('\n');
    let mut out = String::with_capacity(content.len() + 64);
    let mut diff = String::new();
    let mut seen_pkgver = false;
    let mut seen_pkgrel = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];
        if !seen_pkgver && trimmed.starts_with("pkgver=") {
            out.push_str(indent);
            out.push_str("pkgver=");
            out.push_str(target);
            out.push('\n');
            diff.push_str(&format!("- pkgver={old_pkgver}\n+ pkgver={target}\n"));
            seen_pkgver = true;
        } else if !seen_pkgrel && trimmed.starts_with("pkgrel=") {
            out.push_str(indent);
            out.push_str("pkgrel=1\n");
            diff.push_str(&format!("- pkgrel={old_pkgrel}\n+ pkgrel=1\n"));
            seen_pkgrel = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    // 不擅自改变原文件的末尾换行风格
    if !had_trailing_newline && out.ends_with('\n') {
        out.pop();
    }
    Ok((out, diff, old_pkgver, old_pkgrel))
}

/// 备份原文件后写回新内容
///
/// # 参数
/// - `path`: PKGBUILD 路径
/// - `rewritten`: 改写后的完整文本
///
/// # 返回
/// - `String`: 备份文件路径（`PKGBUILD.bak`）
/// - `Err(AppError::FileOperation)`: 备份或写回失败；写回失败时会尝试从备份恢复
pub async fn backup_and_write(path: &Path, rewritten: &str) -> AppResult<String> {
    let backup = path.with_file_name("PKGBUILD.bak");
    fs::copy(path, &backup)
        .await
        .map_err(|e| AppError::FileOperation(format!("备份 PKGBUILD 失败: {e}")))?;
    if let Err(e) = fs::write(path, rewritten).await {
        let _ = fs::copy(&backup, path).await;
        return Err(AppError::FileOperation(format!(
            "写回 PKGBUILD 失败（已尝试从备份恢复）: {e}"
        )));
    }
    Ok(backup.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::{build_rewrite, parse_pkgver_pkgrel, validate_pkgname};

    #[test]
    fn pkgname_rejects_path_traversal() {
        assert!(validate_pkgname("foo-bar").is_ok());
        assert!(validate_pkgname("../etc").is_err());
        assert!(validate_pkgname("a/b").is_err());
        assert!(validate_pkgname("").is_err());
    }

    #[test]
    fn parses_quoted_versions() {
        let (pkgver, pkgrel) = parse_pkgver_pkgrel("pkgver=\"1.2.3\"\npkgrel=4\n");
        assert_eq!(pkgver.as_deref(), Some("1.2.3"));
        assert_eq!(pkgrel.as_deref(), Some("4"));
    }

    #[test]
    fn rewrite_only_touches_pkgver_and_pkgrel() {
        let src = "pkgname=foo\npkgver=1.2.3\npkgrel=4\nsha256sums=('abc')\n";
        let (out, diff, old_pkgver, old_pkgrel) = build_rewrite(src, "1.2.4").unwrap();
        assert_eq!(old_pkgver, "1.2.3");
        assert_eq!(old_pkgrel, "4");
        assert!(out.contains("pkgver=1.2.4"));
        assert!(out.contains("pkgrel=1"));
        assert!(out.contains("sha256sums=('abc')"));
        assert!(diff.contains("+ pkgver=1.2.4"));
        assert!(diff.contains("+ pkgrel=1"));
    }

    #[test]
    fn rewrite_preserves_missing_trailing_newline() {
        let (out, _, _, _) = build_rewrite("pkgver=1.0", "2.0").unwrap();
        assert_eq!(out, "pkgver=2.0");
    }

    #[test]
    fn rewrite_rejects_missing_pkgver() {
        assert!(build_rewrite("pkgname=foo\n", "2.0").is_err());
    }
}
