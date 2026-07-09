use crate::errors::AppResult;
use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use super::trait_def::VersionChecker;
use super::utils::{clean_version, extract_version_with_regex};

/// GitLab 检查器
pub struct GitLabChecker {
    token: Option<String>,
}

impl GitLabChecker {
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

#[async_trait]
impl VersionChecker for GitLabChecker {
    fn name(&self) -> &'static str {
        "gitlab"
    }

    async fn check(&self, client: &Client, upstream_url: &str, pkgname: &str, version_extract_regex: Option<&str>) -> AppResult<Option<String>> {
        info!("[版本检查] 开始检查软件包: {} (检查器: {})", pkgname, self.name());
        debug!("[版本检查] 上游URL: {}", upstream_url);
        debug!("[版本检查] 版本提取正则: {:?}", version_extract_regex);

        if upstream_url.is_empty() {
            debug!("[版本检查] 上游URL为空，跳过检查");
            return Ok(None);
        }
        let parts: Vec<&str> = upstream_url.trim_end_matches('/').trim_end_matches(".git").split('/').collect();
        if parts.len() < 2 {
            debug!("[版本检查] 无法解析 GitLab URL: {}", upstream_url);
            return Ok(None);
        }
        let owner = parts[parts.len() - 2];
        let repo = parts[parts.len() - 1];
        debug!("[版本检查] 解析到仓库: owner={}, repo={}", owner, repo);
        debug!("[版本检查] GitLab Token: {}", if self.token.is_some() { "已配置" } else { "未配置" });

        let project_path = format!("{}%2F{}", owner, repo);
        let api_url = format!("https://gitlab.com/api/v4/projects/{}/releases/permalink/latest", project_path);

        debug!("[GitLab API] 请求URL: {}", api_url);
        debug!("[GitLab API] 项目路径: {}/{}", owner, repo);

        let mut req = client.get(&api_url).header("User-Agent", "my-aur-helper/0.1");
        if let Some(token) = &self.token {
            req = req.header("PRIVATE-TOKEN", token);
        }

        let resp = req.send().await?;
        let status = resp.status();
        debug!("[GitLab API] 响应状态码: {}", status);

        if !status.is_success() {
            debug!("[GitLab API] 请求失败，状态码: {}", status);
            if let Ok(body) = resp.text().await {
                debug!("[GitLab API] 错误响应内容: {}", body);
            }
            return Ok(None);
        }

        let data: serde_json::Value = resp.json().await?;
        debug!("[GitLab API] 原始响应数据: {}", serde_json::to_string_pretty(&data).unwrap_or_default());

        let result = if let Some(tag) = data["tag_name"].as_str() {
            let version = if let Some(regex) = version_extract_regex {
                debug!("[GitLab API] 使用自定义正则提取版本号");
                match extract_version_with_regex(tag, regex) {
                    Some(v) => v,
                    None => {
                        debug!("[GitLab API] 自定义正则匹配失败，使用默认清理");
                        clean_version(tag)
                    }
                }
            } else {
                clean_version(tag)
            };
            debug!("[GitLab API] 提取到版本号: 原始={}, 处理后={}", tag, version);
            Ok(Some(version))
        } else {
            debug!("[GitLab API] 响应中未找到 tag_name 字段");
            Ok(None)
        };

        if let Ok(Some(version)) = &result {
            info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, version);
        } else {
            debug!("[版本检查] 检查完成: {} -> 未找到上游版本", pkgname);
        }
        result
    }
}
