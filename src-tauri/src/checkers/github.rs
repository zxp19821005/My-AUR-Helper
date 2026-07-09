use async_trait::async_trait;
use log::{debug, info, warn};
use reqwest::Client;

use super::trait_def::{CheckOptions, VersionChecker};
use super::utils::{clean_version, extract_owner_repo, extract_version_with_regex};
use crate::errors::AppResult;
use crate::versions;

pub struct GitHubReleaseChecker {
    token: Option<String>,
}

impl GitHubReleaseChecker {
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

#[async_trait]
impl VersionChecker for GitHubReleaseChecker {
    fn name(&self) -> &'static str {
        "github_release"
    }

    async fn check(
        &self,
        client: &Client,
        upstream_url: &str,
        pkgname: &str,
        version_extract_regex: Option<&str>,
        options: &CheckOptions,
    ) -> AppResult<Option<String>> {
        info!("[版本检查] 开始检查软件包: {} (检查器: {})", pkgname, self.name());
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);

        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(None);
        }
        let (owner, repo) = match extract_owner_repo(upstream_url) {
            Some(pair) => pair,
            None => {
                debug!("[版本检查] 无法解析 GitHub URL: {}", upstream_url);
                return Ok(None);
            }
        };
        info!("[GitHub API] 认证: {}", if self.token.is_some() { "已配置 Token" } else { "未配置 Token" });

        if options.check_test_versions {
            let result = check_github_releases(
                client, &owner, &repo, self.token.as_deref(),
                version_extract_regex, options.check_binary_files, pkgname,
            ).await;
            if let Ok(Some(version)) = &result {
                info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, version);
            }
            return result;
        }

        let result = check_github_release_latest(
            client, &owner, &repo, self.token.as_deref(),
            version_extract_regex, options.check_binary_files, pkgname,
        ).await;
        if let Ok(Some(version)) = &result {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, version);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        result
    }
}

pub struct GitHubTagChecker {
    token: Option<String>,
}

impl GitHubTagChecker {
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

#[async_trait]
impl VersionChecker for GitHubTagChecker {
    fn name(&self) -> &'static str {
        "github_tag"
    }

    async fn check(
        &self,
        client: &Client,
        upstream_url: &str,
        pkgname: &str,
        version_extract_regex: Option<&str>,
        _options: &CheckOptions,
    ) -> AppResult<Option<String>> {
        info!("[版本检查] 开始检查软件包: {} (检查器: {})", pkgname, self.name());
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);

        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(None);
        }
        let (owner, repo) = match extract_owner_repo(upstream_url) {
            Some(pair) => pair,
            None => {
                debug!("[版本检查] 无法解析 GitHub URL: {}", upstream_url);
                return Ok(None);
            }
        };

        let result = check_github_tag(client, &owner, &repo, self.token.as_deref(), version_extract_regex).await;
        if let Ok(Some(version)) = &result {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, version);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        result
    }
}

fn has_linux_binary(assets: &[serde_json::Value]) -> bool {
    let not_linux = |name: &str| {
        let lower = name.to_lowercase();
        lower.contains("darwin") || lower.contains("macos") || lower.contains("windows")
    };
    assets.iter().any(|a| {
        a["name"].as_str().is_some_and(|n| !not_linux(n))
    })
}

fn check_release_assets(data: &serde_json::Value, pkgname: &str) {
    let assets = data["assets"].as_array();
    if let Some(list) = assets {
        if list.is_empty() {
            warn!("[二进制检查] {}: Release 无任何附件", pkgname);
        } else if !has_linux_binary(list) {
            let names: Vec<&str> = list.iter().filter_map(|a| a["name"].as_str()).collect();
            warn!("[二进制检查] {}: Release 附件中未找到 Linux 二进制文件: {:?}", pkgname, names);
        } else {
            let linux_assets: Vec<&str> = list.iter().filter_map(|a| {
                let name = a["name"].as_str()?;
                let lower = name.to_lowercase();
                if !lower.contains("darwin") && !lower.contains("macos") && !lower.contains("windows") {
                    Some(name)
                } else {
                    None
                }
            }).collect();
            info!("[二进制检查] {}: 找到 Linux 二进制文件: {:?}", pkgname, linux_assets);
        }
    }
}

pub async fn check_github_release_latest(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    version_extract_regex: Option<&str>,
    check_binary_files: bool,
    pkgname: &str,
) -> AppResult<Option<String>> {
    let api_url = format!("https://api.github.com/repos/{}/{}/releases/latest", owner, repo);

    let mut req = client
        .get(&api_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }

    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }

    let data: serde_json::Value = resp.json().await?;

    if check_binary_files {
        check_release_assets(&data, pkgname);
    }

    if let Some(tag) = data["tag_name"].as_str() {
        let version = if let Some(regex) = version_extract_regex {
            extract_version_with_regex(tag, regex).unwrap_or_else(|| clean_version(tag))
        } else {
            clean_version(tag)
        };
        return Ok(Some(version));
    }
    Ok(None)
}

pub async fn check_github_releases(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    version_extract_regex: Option<&str>,
    check_binary_files: bool,
    pkgname: &str,
) -> AppResult<Option<String>> {
    let api_url = format!("https://api.github.com/repos/{}/{}/releases?per_page=10", owner, repo);

    let mut req = client
        .get(&api_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }

    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }

    let releases: Vec<serde_json::Value> = resp.json().await?;
    let mut best_version: Option<String> = None;

    for release in &releases {
        if let Some(tag) = release["tag_name"].as_str() {
            if release["prerelease"].as_bool().unwrap_or(false) {
                continue;
            }

            if check_binary_files {
                check_release_assets(release, pkgname);
            }

            let version = if let Some(regex) = version_extract_regex {
                extract_version_with_regex(tag, regex).unwrap_or_else(|| clean_version(tag))
            } else {
                clean_version(tag)
            };

            best_version = match best_version.take() {
                Some(current) if versions::compare_versions(&current, &version) == versions::VersionComparison::LessThan => Some(version),
                Some(current) => Some(current),
                None => Some(version),
            };
        }
    }

    Ok(best_version)
}

pub async fn check_github_tag(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    version_extract_regex: Option<&str>,
) -> AppResult<Option<String>> {
    let tags_url = format!("https://api.github.com/repos/{}/{}/tags?per_page=1", owner, repo);

    let mut req = client
        .get(&tags_url)
        .header("User-Agent", "my-aur-helper/0.1")
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }

    let resp = req.send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }

    if let Ok(tags) = resp.json::<Vec<serde_json::Value>>().await {
        if let Some(tag) = tags.first() {
            if let Some(name) = tag["name"].as_str() {
                let version = if let Some(regex) = version_extract_regex {
                    extract_version_with_regex(name, regex).unwrap_or_else(|| clean_version(name))
                } else {
                    clean_version(name)
                };
                return Ok(Some(version));
            }
        }
    }
    Ok(None)
}
