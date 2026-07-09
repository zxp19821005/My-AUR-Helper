use async_trait::async_trait;
use log::{debug, info, warn};
use reqwest::Client;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use super::trait_def::{CheckOptions, VersionChecker};
use crate::errors::AppResult;

pub struct GitDescribeChecker;

#[async_trait]
impl VersionChecker for GitDescribeChecker {
    fn name(&self) -> &'static str {
        "git_describe"
    }

    async fn check(
        &self,
        _client: &Client,
        upstream_url: &str,
        pkgname: &str,
        _version_extract_regex: Option<&str>,
        _options: &CheckOptions,
    ) -> AppResult<Option<String>> {
        info!("[GitDescribe] {}: 开始检查", pkgname);

        if upstream_url.is_empty() {
            debug!("[GitDescribe] {}: 上游URL为空", pkgname);
            return Ok(None);
        }

        let clone_url = if upstream_url.contains("github.com")
            || upstream_url.contains("gitlab")
            || upstream_url.contains("gitee.com")
        {
            upstream_url.to_string()
        } else {
            let url = upstream_url.trim_end_matches('/');
            if url.starts_with("http") || url.starts_with("git@") {
                url.to_string()
            } else {
                format!("https://{}", url)
            }
        };

        let tmp_dir = std::env::temp_dir().join(format!("aur_vcs_{}", pkgname));
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir)?;

        let version = match fetch_git_describe(&clone_url, &tmp_dir).await {
            Ok(Some(v)) => {
                info!("[GitDescribe] {}: 上游版本={}", pkgname, v);
                Some(v)
            }
            Ok(None) => {
                warn!("[GitDescribe] {}: 无法获取版本", pkgname);
                None
            }
            Err(e) => {
                warn!("[GitDescribe] {}: 出错: {}", pkgname, e);
                None
            }
        };

        let _ = std::fs::remove_dir_all(&tmp_dir);
        Ok(version)
    }
}

async fn fetch_git_describe(repo_url: &str, dest: &Path) -> AppResult<Option<String>> {
    debug!("[GitDescribe] 克隆仓库: {}", repo_url);
    let clone_output = Command::new("git")
        .args([
            "clone",
            "--depth=1",
            "--bare",
            repo_url,
            dest.to_str().unwrap_or(""),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| crate::errors::AppError::VersionCheckError(format!("git clone失败: {}", e)))?;

    if !clone_output.status.success() {
        let stderr = String::from_utf8_lossy(&clone_output.stderr);
        debug!("[GitDescribe] 克隆失败: {}", stderr);
        return Ok(None);
    }

    let describe_output = Command::new("git")
        .args(["--git-dir", dest.to_str().unwrap_or(""), "describe", "--always", "--long", "--abbrev=7"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| crate::errors::AppError::VersionCheckError(format!("git describe失败: {}", e)))?;

    if !describe_output.status.success() {
        let stderr = String::from_utf8_lossy(&describe_output.stderr);
        debug!("[GitDescribe] describe失败: {}", stderr);
        return Ok(None);
    }

    let raw_version = String::from_utf8_lossy(&describe_output.stdout).trim().to_string();
    if raw_version.is_empty() {
        return Ok(None);
    }

    debug!("[GitDescribe] git describe原始输出: {}", raw_version);

    let version = raw_version
        .trim_start_matches('v')
        .trim_start_matches('V')
        .replace('-', ".");

    debug!("[GitDescribe] 转换后版本: {}", version);
    Ok(Some(version))
}
