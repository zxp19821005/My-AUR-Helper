/**
 * batch_helpers.rs - 批量检查执行引擎的辅助函数
 *
 * 从 batch.rs 拆分而来，包含纯逻辑辅助函数：
 * - classify：按检查器类型将软件包分为 Manual / Browser / 网络三类
 * - run_one：执行单个软件包的上游版本检查（含结果映射）
 * - check_with_retry：带重试的版本检查
 *
 * 模块设计原则：仅负责辅助逻辑，不含批量调度与数据库写入。
 */
use log::{error, info, warn};
use reqwest::Client;

use crate::checkers::{self, CheckOptions, CheckResult, CheckerSettings};
use crate::errors::{AppError, AppResult};
use crate::models::CheckerType;

use super::batch::PackageTask;
use super::utils::UpstreamCheckResult;

/// 将软件包按检查器类型分为 Manual / Browser / 其余网络检查三类
///
/// # 参数
/// - `tasks`: 待分类的软件包任务列表
///
/// # 返回
/// - `(browser, network, manual)`：浏览器类任务、网络类任务与手动类包名
pub(crate) fn classify(tasks: Vec<PackageTask>) -> (Vec<PackageTask>, Vec<PackageTask>, Vec<String>) {
    let mut browser = Vec::new();
    let mut network = Vec::new();
    let mut manual = Vec::new();
    for t in tasks {
        match t.checker_type {
            // Manual 检查器不产生网络请求，仅收集包名
            CheckerType::Manual => manual.push(t.pkgname),
            // Browser 检查器需严格限制并发（进程资源昂贵）
            CheckerType::Browser => browser.push(t),
            // 其余检查器纳入全局并发池
            _ => network.push(t),
        }
    }
    (browser, network, manual)
}

/// 执行单个软件包的上游版本检查（含重试）
///
/// 返回 UpstreamCheckResult；检查失败时版本留空，交由调用方决定不写库。
///
/// # 参数
/// - `client`: 共享 HTTP 客户端
/// - `settings`: 检查器配置（各平台 Token）
/// - `task`: 待检查的软件包任务
/// - `retry`: 最大重试次数
pub(crate) async fn run_one(
    client: &Client,
    settings: &CheckerSettings,
    task: PackageTask,
    retry: u32,
) -> UpstreamCheckResult {
    let checker = checkers::get_checker(&task.checker_type, settings.clone());
    let options = CheckOptions {
        check_test_versions: task.check_test_versions,
        check_binary_files: task.check_binary_files,
    };
    let result = check_with_retry(
        &*checker,
        client,
        &task.upstream_url,
        &task.pkgname,
        task.version_extract_regex.as_deref(),
        &options,
        retry,
    )
    .await;

    match result {
        Ok(CheckResult {
            version,
            license,
            language_names,
        }) => UpstreamCheckResult {
            pkgname: task.pkgname,
            software_id: task.software_id,
            // 是否过期由调用方与 AUR 版本比较后决定，这里先置 false
            upstream_version: version.unwrap_or_default(),
            is_outdated: false,
            license_spdx_id: license,
            language_names,
        },
        Err(e) => {
            warn!("[批量检查] {} 检查失败: {}", task.pkgname, e);
            UpstreamCheckResult {
                pkgname: task.pkgname,
                software_id: task.software_id,
                upstream_version: String::new(),
                is_outdated: false,
                license_spdx_id: None,
                language_names: vec![],
            }
        }
    }
}

/// 带重试的版本检查
///
/// 顺序尝试直到成功或耗尽重试次数，失败时返回最后一次的错误。
///
/// # 参数
/// - `checker`: 版本检查器实例
/// - `client`: HTTP 客户端
/// - `upstream_url`: 上游仓库 URL
/// - `pkgname`: 软件包名称
/// - `version_extract_regex`: 版本提取正则表达式（可选）
/// - `options`: 检查选项
/// - `retry_count`: 最大重试次数
///
/// # 返回
/// - `Ok(CheckResult)`: 检查成功，包含版本号和 license 信息
/// - `Err(e)`: 所有重试均失败
async fn check_with_retry(
    checker: &dyn checkers::VersionChecker,
    client: &Client,
    upstream_url: &str,
    pkgname: &str,
    version_extract_regex: Option<&str>,
    options: &CheckOptions,
    retry_count: u32,
) -> AppResult<CheckResult> {
    let mut last_error = None;
    for attempt in 0..=retry_count {
        if attempt > 0 {
            // 指数退避延迟：1s, 2s, 4s ...
            let delay_secs = 1u64 << (attempt - 1);
            info!(
                "[重试] 第 {} 次重试 {} (等待 {}s)",
                attempt, pkgname, delay_secs
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
        }
        match checker
            .check(
                client,
                upstream_url,
                pkgname,
                version_extract_regex,
                options,
            )
            .await
        {
            Ok(result) => return Ok(result),
            Err(e) => {
                error!(
                    "检查 {} 失败 (尝试 {}/{}): {}",
                    pkgname,
                    attempt + 1,
                    retry_count + 1,
                    e
                );
                // 永久性错误（DNS 失败、404、403 等）不重试
                if !e.is_retryable() {
                    warn!("[批量检查] {} 错误不可重试，跳过剩余重试", pkgname);
                    return Err(e);
                }
                last_error = Some(e);
            }
        }
    }
    Err(last_error.unwrap_or(AppError::VersionCheckError("检查失败".to_string())))
}
