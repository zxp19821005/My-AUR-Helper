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
/// # 参数
/// - `snap`: GraphQL 批量查询的仓库快照
/// - `item`: 软件包检查项
/// - `all_tags`: 可选的完整 tag 列表（已按 pushed_at DESC 排列，含 REST 回填数据）
///   若为 Some，当 GraphQL tags 窗口不够时会使用该列表；为 None 时仅使用快照内 tags
///
/// 逻辑与 GitHubAPIChecker / GitHubTagsChecker 的 REST 路径保持一致：
/// - 测试版本分支：扫描所有 release 取匹配正则的最大版本，回退 tags
/// - 二进制分支：按时间倒序找首个含匹配 Linux 二进制的 release，优先从资产文件名提取版本
/// - 稳定版本分支：取最新非 prerelease release，正则失败回退扫描 releases / tags
pub(crate) fn select_version(
    snap: &RepoSnapshot,
    item: &GithubBatchItem,
    all_tags: Option<&[String]>,
) -> Option<String> {
    let regex = item.version_extract_regex.as_deref();
    let is_asset_regex = regex.map(has_file_extension).unwrap_or(false);

    // 选择要使用的 tag 列表：优先使用全量（含 REST 回填），回退到快照内窗口
    let tags_to_scan = all_tags.unwrap_or(&snap.tags);

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
        return best.or_else(|| tags_max_version(tags_to_scan, regex, true, None));
    }

    // ---- 二进制分支 ----
    if item.check_binary_files {
        let mut sorted: Vec<&ReleaseData> = snap.releases.iter().filter(|r| !r.is_draft).collect();
        // D6 修复：stable release 同 createdAt 时按 tag_name 字典序取最大（更稳定）
        sorted.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| a.tag_name.cmp(&b.tag_name))
        });
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
        // D4 修复：二进制分支找不到匹配资产时回退到 tags
        return tags_max_version(tags_to_scan, regex, false, None);
    }

    // ---- 稳定版本分支 ----
    // D6 修复：stable release 同 createdAt 时按 tag_name 字典序取最大
    let latest = snap
        .releases
        .iter()
        .filter(|r| !r.is_draft && !r.is_prerelease)
        .max_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.tag_name.cmp(&b.tag_name))
        });

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
                    .or_else(|| tags_max_version(tags_to_scan, regex, true, None));
            }
            Some(clean_version(&r.tag_name))
        }
        None => tags_max_version(tags_to_scan, regex, false, None),
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
///
/// # 参数
/// - `tags`: tag 名称列表，按 pushed_at DESC 排列（最早在尾部）
/// - `regex`: 版本提取正则（可选）
/// - `include_prerelease`: 是否包含 prerelease
/// - `max_search`: 最多从前 N 个 tag 中搜索；None 表示扫描全部
///   用于「大窗口早停」场景：已知某 tag 之后无更新，只搜前 max_search 个
pub(crate) fn tags_max_version(
    tags: &[String],
    regex: Option<&str>,
    include_prerelease: bool,
    max_search: Option<usize>,
) -> Option<String> {
    let limit = max_search.unwrap_or(tags.len());
    let mut best: Option<String> = None;
    for tag in tags.iter().take(limit) {
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

/// 判断版本提取正则是否用于匹配**资产文件名**而非 tag 名
///
/// # 判据
/// 正则里出现文件扩展名或平台标识片段，即视为 asset 正则。
/// 扩展名列表覆盖 Arch 常见预编译产物；平台标识覆盖不带扩展名但
/// 含平台词的资产名（如 `-linux-x64-`、`_linux_`、`x86_64.rpm`）。
///
/// # 为什么必须区分
/// `version_extract_regex` 在 DB 中是单一字段，但有两种互斥语义：
/// - **tag 正则**（`v10\.\d+\.\d+`、`^v?(\d+\.\d+\.\d+)`）→ 匹配 tag 名
/// - **asset 正则**（`_(\d+\.\d+\.\d+)_amd64\.deb`、`x86_64.rpm`）→ 匹配资产文件名
///
/// asset 正则拿去匹配 tag 名必然全 false（tag 形如 `v1.3.1`，不含
/// `_1.3.1_amd64.deb`）。凡是把二者混用的路径都会静默返回"未找到版本"。
pub(crate) fn has_file_extension(regex: &str) -> bool {
    const EXTENSIONS: &[&str] = &[
        ".rpm",
        ".deb",
        ".zip",
        ".tar",
        ".pkg",
        ".dmg",
        ".exe",
        ".AppImage",
        ".snap",
        ".pacman",
        ".appimage",
        ".msi",
        ".msixbundle",
        ".AppImage.tar",
    ];
    const PLATFORM_TOKENS: &[&str] = &[
        "x86_64",
        "amd64",
        "aarch64",
        "arm64",
        "-linux",
        "_linux",
        "linux-",
        "Linux_X86",
    ];
    EXTENSIONS.iter().any(|e| regex.contains(e))
        || PLATFORM_TOKENS.iter().any(|p| regex.contains(p))
}

/// 判断字符串是否看起来像版本号（至少包含一个数字）
fn looks_like_version(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::has_file_extension;

    /// asset 正则必须被识别为 asset 语义（用于匹配 release 资产文件名）
    #[test]
    fn detects_asset_regexes() {
        for rx in [
            r"_(\d+\.\d+\.\d+)_amd64\.deb", // ai-assistant-studio-bin
            r"_(\d+\.\d+\.\d+)_amd64\.deb", // cultivation-bin
            r"x86_64.rpm",                  // aione-bin / arctis-monitor-bin
            r"Anytxt_(\d+\.\d+\.\d+)_Linux_X86_64\.deb",
            r"browser360-cn-stable_([0-9.]+-[0-9]+)_amd64\.deb",
            r"(\d+(?:\.\d+)+)-x86_64\.rpm",
            r"Plexamp-(\d+\.\d+\.\d+)\.AppImage",
            r"-(\d+\.\d+\.\d+)\.pacman", // csbooks-bin
            r"115br_v(\d+(?:\.\d+)*)\.deb",
            r"-linux-x64-",               // alpaca-electron-bin（无扩展名）
            r"_linux_",                   // ai-infra-guard-bin（无扩展名）
            r"[Vv](\d+(?:\.\d+)+)_linux", // sqlark-bin（无扩展名）
        ] {
            assert!(has_file_extension(rx), "应识别为 asset 正则: {rx}");
        }
    }

    /// tag 正则必须被识别为 tag 语义（用于匹配 tag 名）
    #[test]
    fn detects_tag_regexes() {
        for rx in [
            r"v17\.\d+\.\d+",                 // electron17-bin
            r"v2\.\d+\.\d+",                  // electron2-bin
            r"v30\.\d+\.\d+\+wvcus",          // electron30-castlab-bin
            r"^v?(\d+\.\d+\.\d+)",            // coolterm-bin
            r"V(\d+\.\d+\.\d+)",              // baiduwenku-bin
            r"ubuntu\s*(\d+\.\d+\.\d+\.\d+)", // atrust-bin
        ] {
            assert!(
                !has_file_extension(rx),
                "应识别为 tag 正则（不得当成 asset）: {rx}"
            );
        }
    }

    /// 核心回归：asset 正则对 tag 名必然全 false —— 这正是原 bug 的成因。
    /// 若误把 asset 正则当 tag 正则，ai-assistant-studio-bin 会返回"未找到版本"。
    #[test]
    fn asset_regex_never_matches_tag_names() {
        let re = regex::Regex::new(r"_(\d+\.\d+\.\d+)_amd64\.deb").unwrap();
        for tag in ["v2.1.0", "v2.0.0", "v1.3.1", "v1.3.0", "v1.0.0"] {
            assert!(
                !re.is_match(tag),
                "asset 正则不应匹配 tag 名 {tag} —— 混用会导致静默返回 None"
            );
        }
        // 但它确实匹配目标资产名
        assert!(re.is_match("ai-assistant-studio_1.3.1_amd64.deb"));
    }
}
