/**
 * utils.rs - 版本处理工具函数
 *
 * 提供版本比较、排序、查找最新版本等通用功能
 */
use super::{AurVersion, UpstreamVersion, VersionComparison, comparison};

/// 比较 AUR 版本与上游版本
pub fn compare_versions(aur_version: &str, upstream_version: &str) -> VersionComparison {
    log::debug!("[版本标准化] 开始版本比较流程:");
    log::debug!("[版本标准化]   原始 AUR 版本: {}", aur_version);
    log::debug!("[版本标准化]   原始上游版本: {}", upstream_version);

    let aur = AurVersion::parse(aur_version);
    log::debug!("[版本标准化]   AUR 版本解析结果:");
    log::debug!("[版本标准化]     epoch: {:?}", aur.epoch);
    log::debug!("[版本标准化]     version: {}", aur.version);
    log::debug!("[版本标准化]     pkgrel: {:?}", aur.pkgrel);
    log::debug!(
        "[版本标准化]     normalized_version: {}",
        aur.normalized_version
    );

    let upstream = UpstreamVersion::parse(upstream_version);
    log::debug!("[版本标准化]   上游版本解析结果:");
    log::debug!("[版本标准化]     raw: {}", upstream.raw);
    log::debug!(
        "[版本标准化]     normalized_version: {}",
        upstream.normalized_version
    );

    log::debug!(
        "[版本比较] 标准化后比较: AUR={} vs 上游={}",
        aur.normalized_version, upstream.normalized_version
    );

    let result =
        comparison::compare_versions(&aur.normalized_version, &upstream.normalized_version);

    log::debug!("[版本比较] 比较结果: {:?}", result);
    result
}

/// 判断 AUR 版本是否过时（落后于上游版本）
pub fn is_outdated(aur_version: &str, upstream_version: &str) -> bool {
    compare_versions(aur_version, upstream_version) == VersionComparison::LessThan
}

/// 对版本列表进行排序（从旧到新）
pub fn sort_versions<T: AsRef<str>>(versions: &mut [T]) {
    versions.sort_by(|a, b| {
        let cmp = comparison::compare_versions(a.as_ref(), b.as_ref());
        match cmp {
            VersionComparison::LessThan => std::cmp::Ordering::Less,
            VersionComparison::GreaterThan => std::cmp::Ordering::Greater,
            VersionComparison::Equal => std::cmp::Ordering::Equal,
            VersionComparison::Incomparable => std::cmp::Ordering::Equal,
        }
    });
}

/// 从版本列表中查找最新版本
pub fn find_latest_version<T: AsRef<str>>(versions: &[T]) -> Option<&T> {
    versions.iter().max_by(|a, b| {
        let cmp = comparison::compare_versions(a.as_ref(), b.as_ref());
        match cmp {
            VersionComparison::LessThan => std::cmp::Ordering::Less,
            VersionComparison::GreaterThan => std::cmp::Ordering::Greater,
            VersionComparison::Equal => std::cmp::Ordering::Equal,
            VersionComparison::Incomparable => std::cmp::Ordering::Equal,
        }
    })
}
