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

    let a_components = split_components(a_rest);
    let b_components = split_components(b_rest);

    debug!("比较组件: a={:?}, b={:?}", a_components, b_components);

    compare_components_list(&a_components, &b_components)
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
        if extra.iter().any(|c| c.starts_with('~') || is_prerelease_component(c)) {
            return VersionComparison::LessThan;
        }
        return VersionComparison::GreaterThan;
    }

    let extra = &b[a.len()..];
    if extra.iter().any(|c| c.starts_with('~') || is_prerelease_component(c)) {
        return VersionComparison::GreaterThan;
    }
    VersionComparison::LessThan
}