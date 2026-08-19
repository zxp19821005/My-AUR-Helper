/**
 * graphql_batch_helpers.rs - GitHub GraphQL 批量检查的版本挑选辅助函数
 *
 * 从 graphql_batch.rs 拆分而来，包含纯函数逻辑：
 * - select_version：按软件包选项（测试版本 / 二进制 / 正则）从仓库快照挑选上游版本
 * - max_by_vercmp / tags_max_version / releases_max_version：版本比较与扫描
 * - has_file_extension / looks_like_version：正则与版本号判定
 *
 * 逻辑与 GitHubAPIChecker / GitHubTagsChecker 的 REST 路径保持一致。
 */
use crate::checkers::github::binary_check::{extract_version_from_assets, has_linux_binary};
use crate::checkers::github::graphql_batch::GithubBatchItem;
use crate::checkers::github::graphql_batch_parse::{ReleaseData, RepoSnapshot};
use crate::checkers::utils::{clean_version, extract_version_with_regex};
use crate::versions;

/// 根据软件包选项从仓库快照中挑选上游版本
///
/// 逻辑与 GitHubAPIChecker / GitHubTagsChecker 的 REST 路径保持一致：
/// - 测试版本分支：扫描所有 release 取匹配正则的最大版本，回退 tags
/// - 二进制分支：按时间倒序找首个含匹配 Linux 二进制的 release，优先从资产文件名提取版本
/// - 稳定版本分支：取最新非 prerelease release，正则失败回退扫描 releases / tags
pub(crate) fn select_version(snap: &RepoSnapshot, item: &GithubBatchItem) -> Option<String> {
    let regex = item.version_extract_regex.as_deref();
    let is_asset_regex = regex.map(has_file_extension).unwrap_or(false);

    // ---- 测试版本分支 ----
    if item.check_test_versions {
        let mut best: Option<String> = None;
        for r in &snap.releases {
            if r.is_draft {
                continue;
            }
            let ver = if is_asset_regex {
                None
            } else if let Some(re) = regex {
                extract_version_with_regex(&r.tag_name, re)
                    .or_else(|| extract_version_with_regex(&r.name, re))
            } else {
                Some(clean_version(&r.tag_name))
            };
            if let Some(v) = ver {
                best = max_by_vercmp(best, v);
            }
        }
        return best.or_else(|| tags_max_version(&snap.tags, regex, true));
    }

    // ---- 二进制分支 ----
    if item.check_binary_files {
        let mut sorted: Vec<&ReleaseData> = snap.releases.iter().filter(|r| !r.is_draft).collect();
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        for r in sorted {
            if has_linux_binary(&r.assets, regex) {
                if let Some(filter) = regex {
                    if let Some(v) = extract_version_from_assets(&r.assets, filter) {
                        return Some(v);
                    }
                }
                return Some(clean_version(&r.tag_name));
            }
        }
        return None;
    }

    // ---- 稳定版本分支 ----
    let latest = snap
        .releases
        .iter()
        .filter(|r| !r.is_draft && !r.is_prerelease)
        .max_by(|a, b| a.created_at.cmp(&b.created_at));

    match latest {
        Some(r) => {
            if let Some(re) = regex {
                if let Some(v) = extract_version_with_regex(&r.tag_name, re)
                    .or_else(|| extract_version_with_regex(&r.name, re))
                {
                    return Some(v);
                }
                // 正则不匹配 latest：扫描所有 release 取匹配最大版本（含 prerelease），再回退 tags
                return releases_max_version(&snap.releases, regex, true)
                    .or_else(|| tags_max_version(&snap.tags, regex, true));
            }
            Some(clean_version(&r.tag_name))
        }
        None => tags_max_version(&snap.tags, regex, false),
    }
}

/// 取两个版本中的较新者
fn max_by_vercmp(current: Option<String>, candidate: String) -> Option<String> {
    match current {
        Some(c)
            if versions::compare_versions(&c, &candidate)
                == versions::VersionComparison::LessThan =>
        {
            Some(candidate)
        }
        Some(c) => Some(c),
        None => Some(candidate),
    }
}

/// 从 tags 列表中挑选最新版本（与 check_github_tags 一致）
fn tags_max_version(
    tags: &[String],
    regex: Option<&str>,
    include_prerelease: bool,
) -> Option<String> {
    let mut best: Option<String> = None;
    for tag in tags {
        if !include_prerelease && versions::is_prerelease(tag) {
            continue;
        }
        let version = match regex {
            Some(re) => extract_version_with_regex(tag, re),
            None => {
                if looks_like_version(tag) {
                    Some(clean_version(tag))
                } else {
                    None
                }
            }
        };
        if let Some(v) = version {
            best = max_by_vercmp(best, v);
        }
    }
    best
}

/// 从 releases 列表中挑选匹配正则的最大版本（与 check_github_releases 非二进制路径一致）
fn releases_max_version(
    releases: &[ReleaseData],
    regex: Option<&str>,
    include_prerelease: bool,
) -> Option<String> {
    let mut best: Option<String> = None;
    for r in releases {
        if r.is_draft || (!include_prerelease && r.is_prerelease) {
            continue;
        }
        let ver = match regex {
            Some(re) => extract_version_with_regex(&r.tag_name, re)
                .or_else(|| extract_version_with_regex(&r.name, re)),
            None => Some(clean_version(&r.tag_name)),
        };
        if let Some(v) = ver {
            best = max_by_vercmp(best, v);
        }
    }
    best
}

/// 判断版本提取正则是否用于匹配资产文件名（含常见文件扩展名）
fn has_file_extension(regex: &str) -> bool {
    regex.contains(".rpm")
        || regex.contains(".deb")
        || regex.contains(".zip")
        || regex.contains(".tar")
        || regex.contains(".pkg")
        || regex.contains(".dmg")
        || regex.contains(".exe")
        || regex.contains(".AppImage")
}

/// 判断字符串是否看起来像版本号（至少包含一个数字）
fn looks_like_version(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_digit())
}
