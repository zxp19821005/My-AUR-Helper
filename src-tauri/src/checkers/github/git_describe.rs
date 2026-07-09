use log::debug;
use reqwest::Client;
use serde_json;

use crate::errors::AppResult;

pub async fn check_github_git_describe(
    client: &Client,
    owner: &str,
    repo: &str,
    pkgname: &str,
) -> AppResult<Option<String>> {
    // 1. 获取最新的 tag
    let tags_url = format!("https://api.github.com/repos/{}/{}/tags?per_page=1", owner, repo);
    let tags_resp = client.get(&tags_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json")
        .send().await?;

    let latest_tag_name = if tags_resp.status().is_success() {
        let tags: Vec<serde_json::Value> = tags_resp.json().await?;
        tags.first().and_then(|t| t["name"].as_str()).map(|s| s.to_string())
    } else {
        debug!("[GitDescribe] {}: 获取 tags 失败", pkgname);
        None
    };

    // 2. 获取默认分支的最新 commit
    let repo_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let repo_resp = client.get(&repo_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json")
        .send().await?;

    let default_branch = if repo_resp.status().is_success() {
        let repo_data: serde_json::Value = repo_resp.json().await?;
        repo_data["default_branch"].as_str().unwrap_or("main").to_string()
    } else {
        "main".to_string()
    };

    let commits_url = format!(
        "https://api.github.com/repos/{}/{}/commits?sha={}&per_page=1",
        owner, repo, default_branch
    );
    let commits_resp = client.get(&commits_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json")
        .send().await?;

    let latest_commit_sha = if commits_resp.status().is_success() {
        let commits: Vec<serde_json::Value> = commits_resp.json().await?;
        commits.first().and_then(|c| c["sha"].as_str()).map(|s| s[..7].to_string())
    } else {
        debug!("[GitDescribe] {}: 获取 commits 失败", pkgname);
        None
    };

    // 3. 格式化版本
    if let Some(tag) = latest_tag_name {
        if let Some(hash) = latest_commit_sha {
            let commit_count = get_commit_count_since_tag(client, owner, repo, &tag, &default_branch).await;
            
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
        let total_commits = get_total_commit_count(client, owner, repo, &default_branch).await;
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

async fn get_commit_count_since_tag(
    client: &Client,
    owner: &str,
    repo: &str,
    tag: &str,
    branch: &str,
) -> Option<usize> {
    let tags_url = format!("https://api.github.com/repos/{}/{}/tags?per_page=100", owner, repo);
    let tags_resp = client.get(&tags_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .send().await.ok()?;
    
    let tags: Vec<serde_json::Value> = tags_resp.json().await.ok()?;
    let tag_commit = tags.iter().find(|t| t["name"].as_str() == Some(tag))
        .and_then(|t| t["commit"]["sha"].as_str())?;

    let compare_url = format!(
        "https://api.github.com/repos/{}/{}/compare/{}...{}",
        owner, repo, tag_commit, branch
    );
    let compare_resp = client.get(&compare_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .send().await.ok()?;
    
    if compare_resp.status().is_success() {
        let compare_data: serde_json::Value = compare_resp.json().await.ok()?;
        compare_data["ahead_by"].as_u64().map(|n| n as usize)
    } else {
        None
    }
}

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
    
    let resp = client.get(&commits_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .send().await.ok()?;
    
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