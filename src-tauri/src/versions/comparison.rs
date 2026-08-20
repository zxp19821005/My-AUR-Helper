/**
 * comparison.rs - 版本比较算法（ALPM/pacman vercmp）
 *
 * 功能：
 * - 实现 pacman 风格的版本号比较（vercmp 算法）
 * - 支持 epoch、tilde 预发布、字母后缀等复杂版本格式
 * - 支持 rN.HASH 格式的 git 版本比较
 *
 * 版本格式支持：
 * - epoch:version (如 2:1.0.0)
 * - tilde 预发布: 1.0.0~beta (小于正式版)
 * - 字母后缀: 1.0.0a, 1.0.0b
 * - rc/dev/pre: 1.0.0rc1, 1.0.0dev
 */
use super::git_version::{extract_commit_count, is_r_format};
use log::debug;

mod parser;
#[cfg(test)]
mod tests;

pub use parser::{compare_component, is_prerelease_component, split_components, split_epoch};

/// 版本比较结果枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    LessThan,
    GreaterThan,
    Equal,
    Incomparable,
}

/// 判断版本是否为预发布版本
pub fn is_prerelease(version: &str) -> bool {
    let lower = version.to_lowercase();
    lower.contains("alpha")
        || lower.contains("beta")
        || lower.contains("rc")
        || lower.contains("pre")
        || lower.contains("dev")
        || lower.contains("snapshot")
}

/// 比较两个版本号字符串
///
/// # 参数
/// - `a`: 第一个版本号
/// - `b`: 第二个版本号
///
/// # 返回
/// - `VersionComparison::Equal`: 版本相等
/// - `VersionComparison::LessThan`: a 小于 b
/// - `VersionComparison::GreaterThan`: a 大于 b
/// - `VersionComparison::Incomparable`: 无法比较（如 r 格式解析失败）
pub fn compare_versions(a: &str, b: &str) -> VersionComparison {
    compare_vercmp(a, b)
}

/// 核心 vercmp 比较逻辑
pub(crate) fn compare_vercmp(a: &str, b: &str) -> VersionComparison {
    let a = a.trim().to_string();
    let b = b.trim().to_string();

    if a == b {
        return VersionComparison::Equal;
    }

    // 特殊处理 rN.HASH 格式（没有 tag 的 git 版本）
    if is_r_format(&a) && is_r_format(&b) {
        return compare_r_format(&a, &b);
    }

    let (a_epoch, a_rest) = split_epoch(&a);
    let (b_epoch, b_rest) = split_epoch(&b);

    if let (Some(ae), Some(be)) = (a_epoch, b_epoch) {
        match ae.cmp(&be) {
            std::cmp::Ordering::Less => return VersionComparison::LessThan,
            std::cmp::Ordering::Greater => return VersionComparison::GreaterThan,
            std::cmp::Ordering::Equal => {}
        }
    } else if a_epoch.is_some() {
        return VersionComparison::GreaterThan;
    } else if b_epoch.is_some() {
        return VersionComparison::LessThan;
    }

    // 将 pkgrel（末尾的 -<数字> 段）与 pkgver 分离。
    // 关键语义：pkgrel 仅在 pkgver 完全相等时参与比较，
    // 否则 pkgrel 会被错误地当成版本组件进行错位比较（如 9.0.1-1 vs 9.0-5）。
    let (a_ver, a_pkgrel) = split_pkgrel(a_rest);
    let (b_ver, b_pkgrel) = split_pkgrel(b_rest);

    let a_components = split_components(a_ver);
    let b_components = split_components(b_ver);

    debug!("比较组件: a={:?}, b={:?}", a_components, b_components);

    match compare_components_list(&a_components, &b_components) {
        // pkgver 相等时才比较 pkgrel（缺失 pkgrel 视作 0）
        VersionComparison::Equal => {
            let ap = a_pkgrel.unwrap_or(0);
            let bp = b_pkgrel.unwrap_or(0);
            match ap.cmp(&bp) {
                std::cmp::Ordering::Less => VersionComparison::LessThan,
                std::cmp::Ordering::Greater => VersionComparison::GreaterThan,
                std::cmp::Ordering::Equal => VersionComparison::Equal,
            }
        }
        other => other,
    }
}

/// 将「去掉 epoch 后的版本字符串」拆分为 (pkgver, pkgrel)
///
/// # 参数
/// - `rest`: 去掉 epoch 的版本部分，如 `9.0.1-1`、`9.0-5`、`1.2.3~alpha-2`
///
/// # 返回
/// - `(pkgver, Some(pkgrel))`：当末尾存在 `-<纯数字>` 段时
/// - `(原字符串, None)`：否则（无 pkgrel 或末尾段非纯数字，如 `-rc1`）
///
/// # 说明
/// 仅取**最后一个** `-` 之后的纯数字段作为 pkgrel，以兼容
/// `1.2.3-rc1-2` 这类「版本含连字符 + pkgrel」的合法格式。
fn split_pkgrel(rest: &str) -> (&str, Option<u32>) {
    if let Some(dash_idx) = rest.rfind('-') {
        let tail = &rest[dash_idx + 1..];
        if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(n) = tail.parse::<u32>() {
                return (&rest[..dash_idx], Some(n));
            }
        }
    }
    (rest, None)
}

/// 比较 rN.HASH 格式的 git 版本
fn compare_r_format(a: &str, b: &str) -> VersionComparison {
    let a_count = extract_commit_count(a);
    let b_count = extract_commit_count(b);

    match (a_count, b_count) {
        (Some(a_count), Some(b_count)) => {
            debug!(
                "[版本比较] r格式比较: AUR commit_count={} vs 上游 commit_count={}",
                a_count, b_count
            );
            match a_count.cmp(&b_count) {
                std::cmp::Ordering::Less => VersionComparison::LessThan,
                std::cmp::Ordering::Greater => VersionComparison::GreaterThan,
                std::cmp::Ordering::Equal => VersionComparison::Equal,
            }
        }
        _ => VersionComparison::Incomparable,
    }
}

/// 比较两个组件列表
fn compare_components_list(a: &[String], b: &[String]) -> VersionComparison {
    let min_len = a.len().min(b.len());
    for i in 0..min_len {
        let cmp = compare_component(&a[i], &b[i]);
        match cmp {
            std::cmp::Ordering::Less => return VersionComparison::LessThan,
            std::cmp::Ordering::Greater => return VersionComparison::GreaterThan,
            std::cmp::Ordering::Equal => {}
        }
    }

    if a.len() == b.len() {
        return VersionComparison::Equal;
    }

    if a.len() > b.len() {
        let extra = &a[b.len()..];
        if extra
            .iter()
            .any(|c| c.starts_with('~') || is_prerelease_component(c))
        {
            return VersionComparison::LessThan;
        }
        return VersionComparison::GreaterThan;
    }

    let extra = &b[a.len()..];
    if extra
        .iter()
        .any(|c| c.starts_with('~') || is_prerelease_component(c))
    {
        return VersionComparison::GreaterThan;
    }
    VersionComparison::LessThan
}
