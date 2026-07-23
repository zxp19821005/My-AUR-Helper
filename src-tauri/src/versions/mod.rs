pub mod aur;
pub mod comparison;
pub mod git_version;
pub mod rules;
pub mod upstream;

pub use aur::AurVersion;
pub use comparison::compare_versions as compare_vercmp;
pub use comparison::is_prerelease;
pub use comparison::VersionComparison;
pub use git_version::extract_commit_count;
pub use git_version::is_r_format;
pub use git_version::remove_git_describe_metadata;
pub use upstream::UpstreamVersion;

use log::debug;

pub fn compare_versions(aur_version: &str, upstream_version: &str) -> VersionComparison {
    debug!("[版本标准化] 开始版本比较流程:");
    debug!("[版本标准化]   原始 AUR 版本: {}", aur_version);
    debug!("[版本标准化]   原始上游版本: {}", upstream_version);

    let aur = AurVersion::parse(aur_version);
    debug!("[版本标准化]   AUR 版本解析结果:");
    debug!("[版本标准化]     epoch: {:?}", aur.epoch);
    debug!("[版本标准化]     version: {}", aur.version);
    debug!("[版本标准化]     pkgrel: {:?}", aur.pkgrel);
    debug!(
        "[版本标准化]     normalized_version: {}",
        aur.normalized_version
    );

    let upstream = UpstreamVersion::parse(upstream_version);
    debug!("[版本标准化]   上游版本解析结果:");
    debug!("[版本标准化]     raw: {}", upstream.raw);
    debug!(
        "[版本标准化]     normalized_version: {}",
        upstream.normalized_version
    );

    debug!(
        "[版本比较] 标准化后比较: AUR={} vs 上游={}",
        aur.normalized_version, upstream.normalized_version
    );

    let result =
        comparison::compare_versions(&aur.normalized_version, &upstream.normalized_version);

    debug!("[版本比较] 比较结果: {:?}", result);
    result
}

pub fn is_outdated(aur_version: &str, upstream_version: &str) -> bool {
    compare_versions(aur_version, upstream_version) == VersionComparison::LessThan
}

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
