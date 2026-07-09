use reqwest::Client;

use crate::checkers::utils::{clean_version, extract_version_with_regex};
use crate::errors::AppResult;
use crate::versions;

pub async fn check_github_tags(
    client: &Client,
    owner: &str,
    repo: &str,
    token: Option<&str>,
    version_extract_regex: Option<&str>,
    check_test_versions: bool,
) -> AppResult<Option<String>> {
    let mut page = 1;
    let mut all_tags = Vec::new();

    loop {
        let tags_url = format!(
            "https://api.github.com/repos/{}/{}/tags?per_page=100&page={}",
            owner, repo, page
        );

        let mut req = client
            .get(&tags_url)
            .header("User-Agent", "my-aur-helper/0.1")
            .header("Accept", "application/vnd.github.v3+json");
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            break;
        }

        let tags: Vec<serde_json::Value> = resp.json().await?;
        if tags.is_empty() {
            break;
        }

        for tag in &tags {
            if let Some(name) = tag["name"].as_str() {
                all_tags.push(name.to_string());
            }
        }

        if tags.len() < 100 {
            break;
        }
        page += 1;
    }

    if all_tags.is_empty() {
        return Ok(None);
    }

    let mut best_version: Option<String> = None;

    for tag in &all_tags {
        let version = if let Some(regex) = version_extract_regex {
            extract_version_with_regex(tag, regex)
        } else {
            Some(clean_version(tag))
        };

        if let Some(version) = version {
            if !check_test_versions && versions::is_prerelease(&version) {
                continue;
            }

            best_version = match best_version.take() {
                Some(current) if versions::compare_versions(&current, &version) == versions::VersionComparison::LessThan => Some(version),
                Some(current) => Some(current),
                None => Some(version),
            };
        }
    }

    Ok(best_version)
}