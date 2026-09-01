/**
 * backup_install_helpers.rs - 备份包安装相关的路径校验与 sudoers 规则辅助函数
 *
 * 功能：
 * - PKG_EXTENSIONS: 合法的 pacman 包文件扩展名白名单
 * - validate_package_path: 校验前端传入路径，防止路径遍历/任意文件读取/以 root 安装恶意包
 * - build_pacman_install_rules: 生成 pacman -U 的 NOPASSWD 规则片段
 * - has_pacman_install_rule: 判断 sudoers 内容是否已包含指定目录的免密规则
 * - read_backup_dir: 读取备份目录设置，缺失时回退默认路径
 *
 * 这些函数被 backup_install.rs 的命令复用，也被 cache_install.rs / cache_cleanup.rs
 * 跨模块引用，故以 pub(crate) 暴露并在 backup_install 中再导出。
 */
use std::path::PathBuf;

use crate::db::Database;
use crate::errors::{AppError, AppResult};

/// 合法的 pacman 包文件扩展名
const PKG_EXTENSIONS: &[&str] = &[
    ".pkg.tar.zst",
    ".pkg.tar.xz",
    ".pkg.tar",
    ".tar.zst",
    ".tar.xz",
];

/// 校验前端传入的包文件路径，防止路径遍历导致的任意文件读取 / 以 root 安装恶意包。
///
/// 校验项：
/// 1. 必须是绝对路径；
/// 2. 扩展名必须是合法的 pacman 包扩展名；
/// 3. 规范化（解析符号链接与 `..`）后必须是真实存在的普通文件；
/// 4. 规范化路径必须落在允许的根目录（备份目录或系统缓存目录）内。
///
/// @param full_path - 待校验的路径
/// @param allowed_roots - 允许的根目录（绝对路径）
/// @returns 校验通过的规范化路径
pub(crate) fn validate_package_path(
    full_path: &str,
    allowed_roots: &[PathBuf],
) -> AppResult<PathBuf> {
    let raw = std::path::Path::new(full_path);
    if !raw.is_absolute() {
        return Err(AppError::InvalidInput(format!(
            "路径必须是绝对路径: {}",
            full_path
        )));
    }

    if !PKG_EXTENSIONS.iter().any(|ext| full_path.ends_with(ext)) {
        return Err(AppError::InvalidInput(format!(
            "仅允许操作 pacman 包文件: {}",
            full_path
        )));
    }

    let canon = std::fs::canonicalize(raw)
        .map_err(|e| AppError::InvalidInput(format!("无法访问路径 {}: {}", full_path, e)))?;
    if !canon.is_file() {
        return Err(AppError::InvalidInput(format!(
            "路径不是普通文件: {}",
            full_path
        )));
    }

    // 根目录本身可能含符号链接，逐一规范化后再做前缀比较
    let roots: Vec<PathBuf> = allowed_roots
        .iter()
        .map(|r| std::fs::canonicalize(r).unwrap_or_else(|_| r.clone()))
        .collect();
    if !roots.iter().any(|root| canon.starts_with(root)) {
        return Err(AppError::InvalidInput(format!(
            "路径不在允许的备份目录内: {}",
            full_path
        )));
    }

    Ok(canon)
}

/// 生成对指定目录（含子目录，最多三层）执行 `pacman -U --noconfirm` 的 NOPASSWD 规则片段。
///
/// 注意：sudoers 通配符 `*` 不能跨 `/`，子目录层级需逐层列出。
/// 这里覆盖到三层，足以应对常见的「分类/包」二级目录结构；更深层级可继续追加。
///
/// @param dir - 备份/缓存目录（绝对路径）
/// @returns 逗号分隔的多条规则片段
pub(crate) fn build_pacman_install_rules(dir: &str) -> String {
    let d = dir.trim_end_matches('/');
    format!(
        "/usr/bin/pacman -U --noconfirm {d}/*, /usr/bin/pacman -U --noconfirm {d}/*/*, /usr/bin/pacman -U --noconfirm {d}/*/*/*"
    )
}

/// 判断 sudoers 内容是否已包含对指定目录（含子目录）的 `pacman -U --noconfirm` 免密规则。
///
/// @param content - sudoers 文件内容
/// @param username - 当前用户名
/// @param dir - 备份/缓存目录（绝对路径）
/// @returns 是否存在匹配的免密规则
pub(crate) fn has_pacman_install_rule(content: &str, username: &str, dir: &str) -> bool {
    let d = dir.trim_end_matches('/');
    let base = format!("/usr/bin/pacman -U --noconfirm {d}");
    content.lines().any(|raw| {
        let line = raw.trim();
        // 行首须为当前用户，且整体为 NOPASSWD 规则
        if line.split_whitespace().next() != Some(username) {
            return false;
        }
        line.contains("NOPASSWD:") && line.contains(&base) && line.contains("/*")
    })
}

/// 读取备份目录设置，缺失或为空时回退到默认路径
pub(crate) fn read_backup_dir(db: &impl std::ops::Deref<Target = Database>) -> String {
    db.get_setting("backup_dir")
        .ok()
        .flatten()
        .map(|s| s.value)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/run/media/zxp/Backup/Linux/ZST".to_string())
}

/// 校验备份路径和子目录，防止路径穿越写入任意位置。
///
/// 校验项：
/// 1. backup_path 必须是绝对路径；
/// 2. subdirectory 不能包含 `..` 路径穿越段（如 `../`、`..` 等）；
/// 3. canonicalize 解析真实路径后，必须落在允许的备份根目录内。
///
/// @param backup_path - 前端传入的备份根目录路径
/// @param subdirectory - 子目录名称（可为空字符串）
/// @param allowed_root - 允许的备份根目录绝对路径
/// @returns 校验通过的规范化目标目录 PathBuf
pub(crate) fn validate_backup_path(
    backup_path: &str,
    subdirectory: &str,
    allowed_root: &std::path::PathBuf,
) -> AppResult<std::path::PathBuf> {
    let raw = std::path::Path::new(backup_path);
    if !raw.is_absolute() {
        return Err(AppError::InvalidInput(format!(
            "备份路径必须是绝对路径: {}",
            backup_path
        )));
    }

    // 检查 subdirectory 是否包含路径穿越段
    for segment in subdirectory.split('/') {
        if segment == ".." {
            return Err(AppError::InvalidInput(format!(
                "子目录参数包含非法的路径穿越: {}",
                subdirectory
            )));
        }
    }

    // canonicalize 解析真实路径（会自动解析 .. 和符号链接）
    let canon_root = std::fs::canonicalize(raw)
        .map_err(|e| AppError::InvalidInput(format!("无法访问备份路径 {}: {}", backup_path, e)))?;

    // 验证规范化后的路径在允许的根目录下
    if !canon_root.starts_with(allowed_root) {
        return Err(AppError::InvalidInput(format!(
            "备份路径不在允许的备份目录内: {}（允许: {}）",
            canon_root.display(),
            allowed_root.display()
        )));
    }

    // 构建最终目标路径（subdirectory 已在前面的循环中检查过无 .. 段）
    Ok(canon_root.join(subdirectory))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::env;

    /// 测试正常路径校验通过
    #[test]
    fn test_validate_backup_path_normal() {
        let tmp = env::temp_dir().join("aur_test_normal");
        fs::create_dir_all(&tmp).unwrap();
        let allowed_root = tmp.clone();

        let result = validate_backup_path(tmp.to_str().unwrap(), "", &allowed_root);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp);

        fs::remove_dir_all(&tmp).unwrap();
    }

    /// 测试相对路径被拒绝
    #[test]
    fn test_validate_backup_path_relative_rejected() {
        let allowed_root = PathBuf::from("/tmp");
        let result = validate_backup_path("relative/path", "", &allowed_root);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));
    }

    /// 测试子目录包含 .. 被拒绝
    #[test]
    fn test_validate_backup_path_dotdot_subdirectory_rejected() {
        let tmp = env::temp_dir().join("aur_test_dotdot");
        fs::create_dir_all(&tmp).unwrap();
        let allowed_root = tmp.clone();

        let result = validate_backup_path(tmp.to_str().unwrap(), "../etc", &allowed_root);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::InvalidInput(_)));

        fs::remove_dir_all(&tmp).unwrap();
    }

    /// 测试路径穿越到允许根目录之外被拒绝
    #[test]
    fn test_validate_backup_path_outside_allowed_root() {
        let tmp = env::temp_dir().join("aur_test_outside");
        fs::create_dir_all(&tmp).unwrap();
        let allowed_root = env::temp_dir().join("aur_test_allowed_root");
        fs::create_dir_all(&allowed_root).unwrap();

        // 尝试从 tmp 穿越到 allowed_root 外面
        let deep_path = tmp.join("sub").join("..").join("..");
        let result = validate_backup_path(deep_path.to_str().unwrap(), "", &allowed_root);
        assert!(result.is_err());

        fs::remove_dir_all(&tmp).unwrap();
        fs::remove_dir_all(&allowed_root).unwrap();
    }

    /// 测试空子目录返回根目录
    #[test]
    fn test_validate_backup_path_empty_subdirectory() {
        let tmp = env::temp_dir().join("aur_test_empty_sub");
        fs::create_dir_all(&tmp).unwrap();
        let allowed_root = tmp.clone();

        let result = validate_backup_path(tmp.to_str().unwrap(), "", &allowed_root);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp);

        fs::remove_dir_all(&tmp).unwrap();
    }
}
