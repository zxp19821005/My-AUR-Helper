/**
 * utils.rs - 版本处理工具函数
 *
 * 提供版本比较、排序、查找最新版本等通用功能
 */
use super::{comparison, AurVersion, UpstreamVersion, VersionComparison};

/// 比较 AUR 版本与上游版本（pacman vercmp 语义）
///
/// 归一化只用于消除上游 tag 与 AUR pkgver 的表示差异（v 前缀、发行版后缀、
/// build metadata、连字符），不得抹掉 vercmp 语义所需的 epoch、pkgrel 与
/// git describe revision，否则 `comparison.rs` 实现的比较规则会被整体绕过。
///
/// # 参数
/// - `aur_version`: AUR 侧完整版本串，如 `2:9.0.1-1`
/// - `upstream_version`: 上游侧版本串，如 `v9.0.1-release`
///
/// # 返回
/// - `VersionComparison`: 按 vercmp 语义得出的大小关系
pub fn compare_versions(aur_version: &str, upstream_version: &str) -> VersionComparison {
    log::trace!("[版本标准化] 开始版本比较流程:");
    log::trace!("[版本标准化]   原始 AUR 版本: {}", aur_version);
    log::trace!("[版本标准化]   原始上游版本: {}", upstream_version);

    let aur_raw = aur_version.trim();
    let upstream_raw = upstream_version.trim();

    // git describe 快路径：两边都带 `.rN.gHASH` 时，归一化会把 revision 一并抹掉，
    // 同一 tag 下的新提交将永远判为 Equal（-git 包不报更新）。
    // 此时直接按 vercmp 比较去掉 v/V 前缀后的原始串，保留 rN 与 hash 组件。
    if has_git_describe_metadata(aur_raw) && has_git_describe_metadata(upstream_raw) {
        let cmp = comparison::compare_versions(aur_raw, strip_v_prefix(upstream_raw));
        log::trace!(
            "[版本比较] git describe 快路径: {} vs {} -> {:?}",
            aur_raw,
            upstream_raw,
            cmp
        );
        return cmp;
    }

    let aur = AurVersion::parse(aur_raw);
    log::trace!("[版本标准化]   AUR 版本解析结果:");
    log::trace!("[版本标准化]     epoch: {:?}", aur.epoch);
    log::trace!("[版本标准化]     version: {}", aur.version);
    log::trace!("[版本标准化]     pkgrel: {:?}", aur.pkgrel);
    log::trace!(
        "[版本标准化]     normalized_version: {}",
        aur.normalized_version
    );

    let upstream = UpstreamVersion::parse(upstream_raw);
    log::trace!("[版本标准化]   上游版本解析结果:");
    log::trace!("[版本标准化]     raw: {}", upstream.raw);
    log::trace!(
        "[版本标准化]     normalized_version: {}",
        upstream.normalized_version
    );

    // 关键：把 epoch 与 pkgrel 补回可比较串。
    // 旧实现直接传 normalized_version（只含 version 部分），epoch 被丢弃会让
    // `2:1.2.3`（AUR）与 `1.9.9`（上游）误判为 AUR 落后；pkgrel 被丢弃会让
    // comparison.rs 的 split_pkgrel 分支永不生效。
    let aur_comparable = build_aur_comparable(&aur);

    log::trace!(
        "[版本比较] 标准化后比较: AUR={} vs 上游={}",
        aur_comparable,
        upstream.normalized_version
    );

    let result = comparison::compare_versions(&aur_comparable, &upstream.normalized_version);

    log::trace!("[版本比较] 比较结果: {:?}", result);
    result
}

/// 构造保留 epoch 与 pkgrel 的 AUR 可比较版本串
///
/// # 参数
/// - `aur`: 已解析的 AUR 版本
///
/// # 返回
/// - `String`: 形如 `2:1.2.3-1` 的可比较串
fn build_aur_comparable(aur: &AurVersion) -> String {
    let mut comparable = String::new();
    if let Some(epoch) = aur.epoch {
        comparable.push_str(&epoch.to_string());
        comparable.push(':');
    }
    comparable.push_str(&aur.normalized_version);
    if let Some(ref pkgrel) = aur.pkgrel {
        comparable.push('-');
        comparable.push_str(pkgrel);
    }
    comparable
}

/// 判断版本串是否含 git describe 的 `.rN.gHASH` 元数据
///
/// # 参数
/// - `version`: 原始版本串
///
/// # 返回
/// - `bool`: 含 revision/hash 段则为 true
fn has_git_describe_metadata(version: &str) -> bool {
    crate::versions::remove_git_describe_metadata(version) != version
}

/// 去掉上游 tag 常见的 v/V 前缀
///
/// # 参数
/// - `version`: 原始版本串
///
/// # 返回
/// - `&str`: 去掉前缀后的切片
fn strip_v_prefix(version: &str) -> &str {
    version
        .strip_prefix('v')
        .or_else(|| version.strip_prefix('V'))
        .unwrap_or(version)
}

/// 判断 AUR 版本是否过时（落后于上游版本）
pub fn is_outdated(aur_version: &str, upstream_version: &str) -> bool {
    compare_versions(aur_version, upstream_version) == VersionComparison::LessThan
}

/// 结合 AUR `OutOfDate` 补充信号判定是否提示更新
///
/// 主判定始终是 vercmp：AUR 版本落后于上游才算过期。AUR 的 `OutOfDate`
/// 是社区人工确认的补充信号，仅在 vercmp 判为「版本相同」时生效——常见于
/// 上游未打新 tag、提取正则抓错版本等导致 checker 漏报的场景。
/// vercmp 已判为 GreaterThan / Incomparable 时不覆盖，避免陈旧的社区标记
/// 压过确定的版本关系；判为 LessThan 时本就应提示，无需信号叠加。
///
/// # 参数
/// - `aur_version`: AUR 侧版本串
/// - `upstream_version`: 上游侧版本串
/// - `aur_flagged_out_of_date`: AUR RPC v5 的 `OutOfDate` 标记
/// - `pkgname`: 仅用于日志定位
///
/// # 返回
/// - `bool`: 是否应提示更新
pub fn is_outdated_with_aur_signal(
    aur_version: &str,
    upstream_version: &str,
    aur_flagged_out_of_date: bool,
    pkgname: &str,
) -> bool {
    let cmp = compare_versions(aur_version, upstream_version);
    if cmp == VersionComparison::LessThan {
        return true;
    }
    if cmp == VersionComparison::Equal && aur_flagged_out_of_date {
        log::info!(
            "[版本检查] {} 上游版本与 AUR 相同，但 AUR 已标记 out-of-date，按补充信号提示更新",
            pkgname
        );
        return true;
    }
    false
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_must_not_be_dropped_by_normalization() {
        // AUR 用 epoch 重置版本序列，2:1.2.3 高于任何无 epoch 的版本。
        // 旧实现把 epoch 丢掉，会误报「AUR 落后于上游」。
        assert_eq!(
            compare_versions("2:1.2.3", "1.9.9"),
            VersionComparison::GreaterThan
        );
        assert!(!is_outdated("2:1.2.3", "1.9.9"));
        assert_eq!(
            compare_versions("1:1.2.3", "2:1.2.3"),
            VersionComparison::LessThan
        );
    }

    #[test]
    fn pkgrel_is_passed_through_to_vercmp() {
        // pkgrel 补回后，「同 pkgver、AUR pkgrel 更高」判为 GreaterThan，
        // 不会被误标为过时；上游无 pkgrel 时按 comparison.rs 约定视作 0。
        assert_eq!(
            compare_versions("1.2.3-5", "1.2.3"),
            VersionComparison::GreaterThan
        );
        assert!(!is_outdated("1.2.3-5", "1.2.3"));
        assert_eq!(
            compare_versions("1.2.3-1", "1.2.3-2"),
            VersionComparison::LessThan
        );
    }

    #[test]
    fn git_describe_revision_is_not_discarded() {
        // 同一 tag 下的新提交：归一化前旧实现两边都变成 2.0.1，判 Equal，
        // 导致 -git 包永远不报更新。快路径保留 rN 组件后应判为落后。
        assert_eq!(
            compare_versions("2.0.1.r0.g30a6260", "v2.0.1.r5.g9a27946"),
            VersionComparison::LessThan
        );
        assert!(is_outdated("2.0.1.r0.g30a6260", "2.0.1.r5.g9a27946"));
    }

    #[test]
    fn normalization_still_strips_tag_noise() {
        assert_eq!(
            compare_versions("1.2.3", "v1.2.3"),
            VersionComparison::Equal
        );
        assert_eq!(
            compare_versions("1.2.3", "1.2.3-release"),
            VersionComparison::Equal
        );
        assert_eq!(
            compare_versions("1.2.3", "1.2.3+build7"),
            VersionComparison::Equal
        );
    }

    #[test]
    fn aur_out_of_date_signal_only_fills_equal_case() {
        // vercmp 已明确落后：无需信号也提示更新
        assert!(is_outdated_with_aur_signal(
            "1.2.3", "1.2.4", false, "pkg-a"
        ));
        // 版本相同 + AUR 标记 out-of-date：补充信号兜底
        assert!(is_outdated_with_aur_signal("1.2.3", "1.2.3", true, "pkg-b"));
        // 版本相同但无标记：不误报
        assert!(!is_outdated_with_aur_signal(
            "1.2.3", "1.2.3", false, "pkg-c"
        ));
        // AUR 版本更高时，陈旧的社区标记不得压过 vercmp 结论
        assert!(!is_outdated_with_aur_signal(
            "1.2.4", "1.2.3", true, "pkg-d"
        ));
    }
}
