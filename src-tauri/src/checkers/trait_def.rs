/**
 * trait_def.rs - 版本检查器 trait 定义
 *
 * 定义 VersionChecker 统一接口（name / check）以及检查选项 CheckOptions、
 * 检查结果 CheckResult，供所有具体检查器实现。
 */
use async_trait::async_trait;
use reqwest::Client;

use crate::errors::AppResult;

#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    pub check_test_versions: bool,
    pub check_binary_files: bool,
}

/// 版本检查结果
#[derive(Debug, Clone, Default)]
pub struct CheckResult {
    /// 版本号
    pub version: Option<String>,
    /// License SPDX ID（如 "MIT", "Apache-2.0"）
    pub license: Option<String>,
    /// 编程语言名称列表（由检查器从上游获取）
    pub language_names: Vec<String>,
}

// async-trait (expand.rs) 会给 trait 中的 async 方法注入无 message 的 `#[must_use]`，
// 而展开后的 `Pin<Box<dyn Future>>` 返回类型本身已是 must_use，历史上会触发 clippy 的
// double_must_use。该 lint 来自上游宏展开，非本文件写法问题；项目依赖
// `Box<dyn VersionChecker>` 动态分发，不能改用原生 async fn in trait。
//
// 用 `allow` 而非 `expect`：是否触发取决于 clippy / async-trait 版本组合，
// `expect` 在 lint 不再触发时会因 `-D unfulfilled-lint-expectations` 直接编译失败
// （2026-11-26 clippy 1.96 实测）。
#[allow(clippy::double_must_use)]
#[async_trait]
pub trait VersionChecker: Send + Sync {
    fn name(&self) -> &'static str;

    async fn check(
        &self,
        client: &Client,
        upstream_url: &str,
        pkgname: &str,
        version_extract_regex: Option<&str>,
        options: &CheckOptions,
    ) -> AppResult<CheckResult>;
}
