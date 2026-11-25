/**
 * api_urls.rs - GitHub REST / GraphQL API URL 集中构造
 *
 * 功能：统一生成 checkers/github 各检查器与 db/github_tag_cache_extend.rs 所需的 GitHub API URL。
 *
 * 工作流程：
 * 1. 所有 URL 由 GH_API_BASE 常量加路径片段拼接，域名不再散落到各检查器；
 * 2. 分页参数用 `impl Display` 承接调用点的 usize / i32，避免额外的类型转换；
 * 3. 查询串的键值、顺序与抽取前逐字一致（见本文件末尾的黄金值单测）。
 *
 * 适用场景：GitHub tags / releases / graphql / 仓库元信息 / commit / compare 请求。
 * 约定：全仓 `api.github.com` 字面量只允许出现在该文件（含单测）。
 */
use std::fmt::Display;

/// GitHub API 根地址（全仓唯一硬编码落点）
const GH_API_BASE: &str = "https://api.github.com";

/// 拼接仓库 API 根路径 `repos/{owner}/{repo}`
fn repo_base(owner: &str, repo: &str) -> String {
    format!("{GH_API_BASE}/repos/{owner}/{repo}")
}

/// GraphQL 查询端点
pub(crate) fn graphql_url() -> String {
    format!("{GH_API_BASE}/graphql")
}

/// 仓库信息端点（用于读取 default_branch 等基础字段）
pub(crate) fn repo_url(owner: &str, repo: &str) -> String {
    repo_base(owner, repo)
}

/// 最新 release 端点（release.rs 与 db/github_tag_cache_extend.rs 共用）
pub(crate) fn latest_release_url(owner: &str, repo: &str) -> String {
    let base = repo_base(owner, repo);
    format!("{base}/releases/latest")
}

/// releases 分页列表端点
pub(crate) fn releases_url(
    owner: &str,
    repo: &str,
    per_page: impl Display,
    page: impl Display,
) -> String {
    let base = repo_base(owner, repo);
    format!("{base}/releases?per_page={per_page}&page={page}")
}

/// 指定 tag 的 release 端点
pub(crate) fn release_tag_url(owner: &str, repo: &str, tag: &str) -> String {
    let base = repo_base(owner, repo);
    format!("{base}/releases/tags/{tag}")
}

/// tags 分页端点（按 push 时间倒序；tags.rs 与 graphql_batch_tags.rs 完全相同的 URL）
pub(crate) fn tags_page_url(
    owner: &str,
    repo: &str,
    per_page: impl Display,
    page: impl Display,
) -> String {
    let base = repo_base(owner, repo);
    format!("{base}/tags?per_page={per_page}&page={page}&sort=pushed_at&direction=desc")
}

/// tags 首页端点（不带排序参数；git_describe 用 per_page=1 / 100 两种取值）
pub(crate) fn tags_first_url(owner: &str, repo: &str, per_page: impl Display) -> String {
    let base = repo_base(owner, repo);
    format!("{base}/tags?per_page={per_page}")
}

/// 指定 sha / 分支的最新一个 commit 端点
pub(crate) fn commits_url(owner: &str, repo: &str, sha: &str) -> String {
    let base = repo_base(owner, repo);
    format!("{base}/commits?sha={sha}&per_page=1")
}

/// compare 端点（计算某个 tag commit 到分支 HEAD 的提交差异）
pub(crate) fn compare_url(owner: &str, repo: &str, base_ref: &str, head_ref: &str) -> String {
    let base = repo_base(owner, repo);
    format!("{base}/compare/{base_ref}...{head_ref}")
}

/// 仓库 license 端点
pub(crate) fn license_url(owner: &str, repo: &str) -> String {
    let base = repo_base(owner, repo);
    format!("{base}/license")
}

/// 仓库 languages 端点
pub(crate) fn languages_url(owner: &str, repo: &str) -> String {
    let base = repo_base(owner, repo);
    format!("{base}/languages")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 抽取前各 URL 的黄金值，防止路径 / 查询串参数顺序被无意改变
    #[test]
    fn test_urls_match_pre_extraction_literals() {
        assert_eq!(GH_API_BASE, "https://api.github.com");
        assert_eq!(graphql_url(), "https://api.github.com/graphql");
        assert_eq!(repo_url("o", "r"), "https://api.github.com/repos/o/r");
        assert_eq!(
            latest_release_url("o", "r"),
            "https://api.github.com/repos/o/r/releases/latest"
        );
        assert_eq!(
            releases_url("o", "r", 30, 2),
            "https://api.github.com/repos/o/r/releases?per_page=30&page=2"
        );
        assert_eq!(
            release_tag_url("o", "r", "v1.2"),
            "https://api.github.com/repos/o/r/releases/tags/v1.2"
        );
        assert_eq!(
            tags_page_url("o", "r", 100, 1),
            "https://api.github.com/repos/o/r/tags?per_page=100&page=1&sort=pushed_at&direction=desc"
        );
        assert_eq!(
            tags_first_url("o", "r", 1),
            "https://api.github.com/repos/o/r/tags?per_page=1"
        );
        assert_eq!(
            commits_url("o", "r", "main"),
            "https://api.github.com/repos/o/r/commits?sha=main&per_page=1"
        );
        assert_eq!(
            compare_url("o", "r", "v1", "main"),
            "https://api.github.com/repos/o/r/compare/v1...main"
        );
        assert_eq!(
            license_url("o", "r"),
            "https://api.github.com/repos/o/r/license"
        );
        assert_eq!(
            languages_url("o", "r"),
            "https://api.github.com/repos/o/r/languages"
        );
    }
}
