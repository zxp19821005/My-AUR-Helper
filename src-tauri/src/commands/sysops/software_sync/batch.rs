/**
 * batch.rs - 上游版本批量检查执行引擎
 *
 * 功能：按检查器类型对软件包分类后分组执行，在受控并发的前提下并行检查。
 * - Manual 检查器：跳过网络请求，仅返回需手动更新的包名列表
 * - Browser 检查器：使用独立且更严格的信号量限制并发，避免同时拉起过多
 *   Chrome 进程导致内存耗尽 / 文件描述符耗尽
 * - 其余检查器（GitHub/Gitee/GitLab/HTTP/Redirect）：使用全局并发信号量，
 *   降低对上游的请求压力、规避限流
 *
 * 模块设计原则：
 * - 本模块仅负责「分类 + 分组并发执行」，不含数据库写入逻辑（写库交由调用方）
 * - 单个文件控制在 300 行以内
 */
use std::collections::HashSet;
use std::sync::Arc;

use log::{error, info, warn};
use reqwest::Client;
use tokio::sync::Semaphore;

use crate::checkers::github::graphql_batch::{batch_check_github, GithubBatchItem};
use crate::checkers::utils::extract_owner_repo;
use crate::checkers::{self, CheckOptions, CheckResult, CheckerSettings};
use crate::errors::{AppError, AppResult};
use crate::models::{CheckerType, PackageType};

use super::utils::UpstreamCheckResult;

/// 浏览器检查器最大并发数：每个 headless Chrome 进程内存占用较大，
/// 过高会导致 OOM / 文件描述符耗尽。后续可改为读取设置项覆盖。
const MAX_BROWSER_CONCURRENCY: usize = 4;

/// 其余 HTTP 类检查器全局最大并发数：控制对上游的请求压力，规避限流。
const MAX_NETWORK_CONCURRENCY: usize = 16;

/// 单次批量检查待执行的软件包任务
///
/// 由 SoftwareInfo 映射而来，携带检查器执行所需的全部字段。
pub struct PackageTask {
    /// 软件包名称
    pub pkgname: String,
    /// 软件包数据库 ID
    pub software_id: i64,
    /// 上游仓库 URL
    pub upstream_url: String,
    /// 版本提取正则表达式（可选）
    pub version_extract_regex: Option<String>,
    /// 是否检查测试版本（prerelease）
    pub check_test_versions: bool,
    /// 是否检查二进制文件
    pub check_binary_files: bool,
    /// 检查器类型（决定分类归属）
    pub checker_type: CheckerType,
    /// 软件包类型（透传备用，为后续按包类型做批量优化预留）
    pub package_type: PackageType,
}

/// 批量检查结果
///
/// - checked：通过检查器得到版本（或失败留空）的包，交由调用方写库
/// - manual：使用 Manual 检查器、需用户手动更新的包名列表（不发起网络请求）
pub struct BatchOutcome {
    /// 已执行的检查结果（版本可能为空的失败项）
    pub checked: Vec<UpstreamCheckResult>,
    /// 手动检查器包名列表
    pub manual: Vec<String>,
}

/// 将软件包按检查器类型分为 Manual / Browser / 其余网络检查三类
///
/// # 参数
/// - `tasks`: 待分类的软件包任务列表
///
/// # 返回
/// - `(browser, network, manual)`：浏览器类任务、网络类任务与手动类包名
fn classify(tasks: Vec<PackageTask>) -> (Vec<PackageTask>, Vec<PackageTask>, Vec<String>) {
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
async fn run_one(
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
            info!("[重试] 第 {} 次重试 {}", attempt, pkgname);
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
                last_error = Some(e);
            }
        }
    }
    Err(last_error.unwrap_or(AppError::VersionCheckError("检查失败".to_string())))
}

/// 分类并行检查所有软件包的上游版本
///
/// 工作流程：
/// 1. 按检查器类型分类为 Manual / Browser / 网络三类
/// 2. Browser 走独立严格并发信号量，网络类走全局并发信号量
/// 3. 收集结果，Manual 仅返回包名列表（不发起网络请求）
///
/// # 参数
/// - `tasks`: 待检查的软件包任务列表
/// - `client`: 共享 HTTP 客户端
/// - `settings`: 检查器配置（各平台 Token）
/// - `retry`: 单包最大重试次数
///
/// # 返回
/// - `BatchOutcome`: 已检查结果与手动检查包名列表
pub async fn batch_check_upstream(
    tasks: Vec<PackageTask>,
    client: Client,
    settings: CheckerSettings,
    retry: u32,
) -> BatchOutcome {
    let (browser_tasks, network_tasks, manual) = classify(tasks);

    // 浏览器类使用更严格的并发上限，避免同时拉起过多 Chrome 进程
    let browser_sem = Arc::new(Semaphore::new(MAX_BROWSER_CONCURRENCY));
    // 其余网络类使用全局并发上限，缓解上游限流
    let network_sem = Arc::new(Semaphore::new(MAX_NETWORK_CONCURRENCY));

    let mut handles = Vec::new();

    // Browser 桶：独立严格并发
    for task in browser_tasks {
        let client = client.clone();
        let settings = settings.clone();
        let sem = browser_sem.clone();
        handles.push(tokio::spawn(async move {
            // 获取并发许可后再执行，超出上限的任务在此排队等待；
            // 许可随 _permit 在任务结束时自动释放
            let _permit = sem.acquire().await.expect("浏览器并发信号量已被关闭");
            run_one(&client, &settings, task, retry).await
        }));
    }

    // ---- 拆分网络桶：GitHub 与必然 REST ----
    // github_items/origin：可走 GraphQL 的包（URL 解析成功、非 git、有 Token）
    // fallback_definite：git 包 / URL 解析失败 / 非 GitHub 检查器 —— 必然走 REST
    let mut github_items: Vec<GithubBatchItem> = Vec::new();
    let mut github_origin: Vec<PackageTask> = Vec::new();
    let mut fallback_definite: Vec<PackageTask> = Vec::new();

    for task in network_tasks {
        let is_github = matches!(
            task.checker_type,
            CheckerType::GitHubTags | CheckerType::GitHubAPI
        );
        // git 包需 commit 计数（无廉价 GraphQL 等价）；无 Token 时 GraphQL 基本不可用
        if is_github && task.package_type != PackageType::Git && settings.github_token.is_some() {
            if let Some((owner, repo)) = extract_owner_repo(&task.upstream_url) {
                github_items.push(GithubBatchItem {
                    pkgname: task.pkgname.clone(),
                    software_id: task.software_id,
                    owner,
                    repo,
                    package_type: task.package_type.clone(),
                    check_test_versions: task.check_test_versions,
                    check_binary_files: task.check_binary_files,
                    version_extract_regex: task.version_extract_regex.clone(),
                });
                github_origin.push(task);
                continue;
            }
        }
        fallback_definite.push(task);
    }

    // 并发点：GraphQL 批量查询与「必然 REST」桶同时启动，压平墙钟时间。
    // 成功命中 GraphQL 的包不再发 REST；仅未命中的 github 包后续补回落，
    // 既避免重复请求（保住限流收益），又避免同一包被双重处理。
    let github_handle = {
        let client = client.clone();
        // 仅克隆 token 字段，settings 仍需用于下方 REST 任务
        let token = settings.github_token.clone();
        let items = github_items;
        tokio::spawn(async move { batch_check_github(&client, items, token.as_deref()).await })
    };

    // 必然走 REST 的包：全局并发（与 GraphQL 并行执行）
    for task in fallback_definite {
        let client = client.clone();
        let settings = settings.clone();
        let sem = network_sem.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("网络并发信号量已被关闭");
            run_one(&client, &settings, task, retry).await
        }));
    }

    // 等待浏览器 + 必然 REST 任务完成（与 GraphQL 并行，不互相等待）
    let mut checked = Vec::new();
    for h in handles {
        // 单个任务 panic 不应拖垮整体批量检查
        if let Ok(r) = h.await {
            checked.push(r);
        }
    }

    // 回收 GraphQL 结果：命中（无论是否取到版本）的包不再走 REST；
    // 仅未命中的 github 包补回落 REST（此时 GraphQL 已完成，该集合通常很小）
    let github_results = match github_handle.await {
        Ok(r) => r,
        Err(e) => {
            warn!("[批量检查] GitHub GraphQL 任务失败: {}", e);
            Vec::new()
        }
    };
    let processed: HashSet<String> = github_results.iter().map(|o| o.pkgname.clone()).collect();

    let mut checked_from_graphql: Vec<UpstreamCheckResult> = github_results
        .into_iter()
        .map(|o| UpstreamCheckResult {
            pkgname: o.pkgname,
            software_id: o.software_id,
            // 检查失败 / 无可解析版本（含仓库存在但无 tag/release）留空，交由调用方置 is_outdated=false
            upstream_version: o.version.unwrap_or_default(),
            is_outdated: false,
            license_spdx_id: o.license_spdx_id,
            language_names: o.language_names,
        })
        .collect();

    // 仅未命中的 github 包补 REST 回落（数量少，单独并发收集）
    let mut miss_handles = Vec::new();
    for origin in github_origin {
        if !processed.contains(&origin.pkgname) {
            let client = client.clone();
            let settings = settings.clone();
            let sem = network_sem.clone();
            let task = origin;
            miss_handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.expect("网络并发信号量已被关闭");
                run_one(&client, &settings, task, retry).await
            }));
        }
    }
    for h in miss_handles {
        if let Ok(r) = h.await {
            checked.push(r);
        }
    }

    // 合并 GraphQL 批量命中结果
    checked.append(&mut checked_from_graphql);

    BatchOutcome { checked, manual }
}
