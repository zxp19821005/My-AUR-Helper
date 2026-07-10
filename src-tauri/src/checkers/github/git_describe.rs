/**
 * git_describe.rs - Git Describe 版本格式化逻辑（-git 包专用）
 *
 * 功能：通过 GitHub API 获取 tag 和 commit 信息，无需克隆仓库，
 * 生成类似 `git describe --long --tags --abbrev=7` 的版本字符串。
 *
 * 版本格式：
 * - 有 tag 时：{tag}.r{commit_count}.g{short_hash}
 *   例如：1.0.0.beta.118.r0.gc044c59
 *   表示基于 tag 1.0.0.beta.118，之后有 0 次提交，当前 commit hash 为 c044c59
 * - 无 tag 时：r{total_commits}.{short_hash}
 *   例如：r1234.b9a08cc
 *   表示仓库总共有 1234 次提交，当前 commit hash 为 b9a08cc
 *
 * 实现原理：
 * 1. 通过 GitHub Tags API 获取最新 tag
 * 2. 通过 GitHub Repo API 获取默认分支
 * 3. 通过 GitHub Commits API 获取最新 commit hash
 * 4. 通过 GitHub Compare API 计算 tag 到当前分支的 commit 数量
 * 5. 通过 GitHub API 的 Link header 获取总 commit 数量
 *
 * 注意：本模块不需要克隆仓库，所有操作均通过 GitHub REST API 完成。
 */
use log::debug;
use reqwest::Client;
use serde_json;

use crate::errors::AppResult;

/// 通过 GitHub API 生成类似 git describe 的版本字符串
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `token`: GitHub API Token（可选）
/// - `pkgname`: 软件包名称（用于日志）
///
/// # 返回
/// - `Ok(Some(version))`: 格式化后的版本字符串
///   - 有 tag 时：{tag}.r{count}.g{hash}
///   - 无 tag 时：r{count}.{hash}
/// - `Ok(None)`: 无法获取任何版本信息
/// - `Err(e)`: 请求失败
pub async fn check_github_git_describe(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    pkgname: &str,
) -> AppResult<Option<String>> {
    // 1. 获取最新的 tag
    let tags_url = format!(
        "https://api.github.com/repos/{}/{}/tags?per_page=1",
        owner, repo
    );
    let mut tags_req = client
        .get(&tags_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        tags_req = tags_req.header("Authorization", format!("Bearer {}", t));
    }
    let tags_resp = tags_req.send().await?;

    let latest_tag_name = if tags_resp.status().is_success() {
        let tags: Vec<serde_json::Value> = tags_resp.json().await?;
        tags.first()
            .and_then(|t| t["name"].as_str())
            .map(|s| s.to_string())
    } else {
        debug!("[GitDescribe] {}: 获取 tags 失败", pkgname);
        None
    };

    // 2. 获取默认分支的最新 commit
    let repo_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let mut repo_req = client
        .get(&repo_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        repo_req = repo_req.header("Authorization", format!("Bearer {}", t));
    }
    let repo_resp = repo_req.send().await?;

    let default_branch = if repo_resp.status().is_success() {
        let repo_data: serde_json::Value = repo_resp.json().await?;
        repo_data["default_branch"]
            .as_str()
            .unwrap_or("main")
            .to_string()
    } else {
        "main".to_string()
    };

    let commits_url = format!(
        "https://api.github.com/repos/{}/{}/commits?sha={}&per_page=1",
        owner, repo, default_branch
    );
    let mut commits_req = client
        .get(&commits_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        commits_req = commits_req.header("Authorization", format!("Bearer {}", t));
    }
    let commits_resp = commits_req.send().await?;

    let latest_commit_sha = if commits_resp.status().is_success() {
        let commits: Vec<serde_json::Value> = commits_resp.json().await?;
        commits
            .first()
            .and_then(|c| c["sha"].as_str())
            .map(|s| s[..7].to_string())
    } else {
        debug!("[GitDescribe] {}: 获取 commits 失败", pkgname);
        None
    };

    // 3. 格式化版本
    if let Some(tag) = latest_tag_name {
        if let Some(hash) = latest_commit_sha {
            let commit_count =
                get_commit_count_since_tag(client, owner, repo, &tag, &default_branch)
                    .await;

            let version = if let Some(count) = commit_count {
                format!("{}.r{}.g{}", tag, count, hash)
            } else {
                format!("{}.r0.g{}", tag, hash)
            };

            debug!("[GitDescribe] {}: 格式化版本={}", pkgname, version);
            return Ok(Some(version));
        }
    }

    // 如果没有 tag，使用 r{count}.{hash} 格式
    if let Some(hash) = latest_commit_sha {
        let total_commits =
            get_total_commit_count(client, owner, repo, &default_branch).await;
        let version = if let Some(count) = total_commits {
            format!("r{}.{}", count, hash)
        } else {
            format!("r0.{}", hash)
        };
        debug!("[GitDescribe] {}: 无tag，使用={}", pkgname, version);
        return Ok(Some(version));
    }

    Ok(None)
}

/// 获取从指定 tag 到当前分支的 commit 数量
///
/// 通过 GitHub Compare API 计算 tag commit 和分支 HEAD 之间的差异。
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `tag`: tag 名称
/// - `branch`: 分支名称
///
/// # 返回
/// - `Some(count)`: tag 之后的 commit 数量
/// - `None`: 获取失败
async fn get_commit_count_since_tag(
    client: &Client,
    owner: &str,
    repo: &str,
    tag: &str,
    branch: &str,
) -> Option<usize> {
    // 先获取 tag 对应的 commit SHA
    let tags_url = format!(
        "https://api.github.com/repos/{}/{}/tags?per_page=100",
        owner, repo
    );
    let tags_resp = client
        .get(&tags_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .send()
        .await
        .ok()?;

    let tags: Vec<serde_json::Value> = tags_resp.json().await.ok()?;
    let tag_commit = tags
        .iter()
        .find(|t| t["name"].as_str() == Some(tag))
        .and_then(|t| t["commit"]["sha"].as_str())?;

    // 通过 Compare API 获取 commit 差异
    let compare_url = format!(
        "https://api.github.com/repos/{}/{}/compare/{}...{}",
        owner, repo, tag_commit, branch
    );
    let compare_resp = client
        .get(&compare_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .send()
        .await
        .ok()?;

    if compare_resp.status().is_success() {
        let compare_data: serde_json::Value = compare_resp.json().await.ok()?;
        compare_data["ahead_by"].as_u64().map(|n| n as usize)
    } else {
        None
    }
}

/// 获取仓库的总 commit 数量
///
/// 通过 GitHub API 的 Link header 中的最后一页页码获取总 commit 数量。
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `owner`: GitHub 仓库所有者
/// - `repo`: GitHub 仓库名称
/// - `branch`: 分支名称
///
/// # 返回
/// - `Some(count)`: 总 commit 数量
/// - `None`: 获取失败
async fn get_total_commit_count(
    client: &Client,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Option<usize> {
    let commits_url = format!(
        "https://api.github.com/repos/{}/{}/commits?sha={}&per_page=1",
        owner, repo, branch
    );

    let resp = client
        .get(&commits_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .send()
        .await
        .ok()?;

    // 从 Link header 中解析最后一页的页码
    if let Some(link) = resp.headers().get("link") {
        let link_str = link.to_str().ok()?;
        if let Some(page) = link_str.split(',').find(|l| l.contains("rel=\"last\"")) {
            if let Some(num) = page.split('?').nth(1) {
                if let Some(count) = num.split('&').find(|p| p.starts_with("page=")) {
                    return count[5..].parse::<usize>().ok();
                }
            }
        }
    }

    None
}