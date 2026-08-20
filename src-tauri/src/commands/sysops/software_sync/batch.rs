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

use log::{info, warn};
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::checkers::github::graphql_batch::{batch_check_github, GithubBatchItem};
use crate::checkers::utils::extract_owner_repo;
use crate::checkers::CheckerSettings;
use crate::models::{CheckerType, PackageType};

use super::batch_helpers::{classify, run_one};
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

/// 分类并行检查所有软件包的上游版本
///
/// # 参数
/// - `tasks`: 待检查的软件包任务列表
/// - `client`: 普通 HTTP 客户端（非 GitHub 请求）
/// - `github_client`: 带代理的 HTTP 客户端（GitHub 请求）
/// - `settings`: 检查器配置（各平台 Token）
/// - `retry`: 单包最大重试次数
///
/// # 返回
/// - `BatchOutcome`: 已检查结果与手动检查包名列表
pub async fn batch_check_upstream(
    tasks: Vec<PackageTask>,
    client: Client,
    github_client: Client,
    settings: CheckerSettings,
    retry: u32,
) -> BatchOutcome {
    let (browser_tasks, network_tasks, manual) = classify(tasks);

    // 浏览器类使用更严格的并发上限，避免同时拉起过多 Chrome 进程
    let browser_sem = Arc::new(Semaphore::new(MAX_BROWSER_CONCURRENCY));
    // 其余网络类使用全局并发上限，缓解上游限流
    let network_sem = Arc::new(Semaphore::new(MAX_NETWORK_CONCURRENCY));

    // 单一 JoinSet 承载所有 run_one 任务（浏览器桶 + 必然 REST 桶 + 未命中 GraphQL 的回落桶），
    // 统一用 join_next() 边完成边回收；回落任务在 GraphQL 完成后并入同一集合，
    // 与其他任务共享并发信号量，不再单独开第二组 await。
    let mut handles: JoinSet<UpstreamCheckResult> = JoinSet::new();

    // Browser 桶：独立严格并发
    for task in browser_tasks {
        let client = client.clone();
        let settings = settings.clone();
        let sem = browser_sem.clone();
        handles.spawn(async move {
            // 获取并发许可后再执行，超出上限的任务在此排队等待；
            // 许可随 _permit 在任务结束时自动释放
            let _permit = sem.acquire().await.expect("浏览器并发信号量已被关闭");
            run_one(&client, &settings, task, retry).await
        });
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

    // 诊断：批量引擎分流汇总（便于排查 GraphQL 是否被命中、为何走 REST fallback）
    info!(
        "[批量检查] 分流汇总: GraphQL待处理={} GitHub-可能回落={} 必然REST={}",
        github_items.len(),
        github_origin.len(),
        fallback_definite.len()
    );
    // 并发点：GraphQL 批量查询与「必然 REST」桶同时启动，压平墙钟时间。
    // 成功命中 GraphQL 的包不再发 REST；仅未命中的 github 包后续补回落，
    // 既避免重复请求（保住限流收益），又避免同一包被双重处理。
    // 注：github_handle 返回 Vec<GithubBatchOutcome>，与 run_one 的返回类型异构，
    // 故保留独立 tokio::spawn，不并入下方同构的 JoinSet。
    let github_handle = {
        let client = github_client.clone();
        // 仅克隆 token 字段，settings 仍需用于下方 REST 任务
        let token = settings.github_token.clone();
        let items = github_items;
        tokio::spawn(async move { batch_check_github(&client, items, token.as_deref()).await })
    };

    // 必然走 REST 的包：全局并发（与 GraphQL 并行执行，并入同一 JoinSet）
    for task in fallback_definite {
        let client = client.clone();
        let settings = settings.clone();
        let sem = network_sem.clone();
        handles.spawn(async move {
            let _permit = sem.acquire().await.expect("网络并发信号量已被关闭");
            run_one(&client, &settings, task, retry).await
        });
    }

    // 第一段 drain：先回收浏览器 + 必然 REST（与 GraphQL 并行，互不阻塞）
    let mut checked = Vec::new();
    while let Some(res) = handles.join_next().await {
        // 单个任务 panic 不应拖垮整体批量检查
        if let Ok(r) = res {
            checked.push(r);
        }
    }

    // 回收 GraphQL 结果：命中（无论是否取到版本）的包不再走 REST；
    // 仅未命中的 github 包补回落 REST。
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

    // 未命中的 github 包并入同一 JoinSet（与其他任务共享并发信号量与回收逻辑），
    // 回落任务之间并行执行，无需单独开第二组 await / 第二组 JoinSet。
    for origin in github_origin {
        if !processed.contains(&origin.pkgname) {
            let client = github_client.clone();
            let settings = settings.clone();
            let sem = network_sem.clone();
            let task = origin;
            handles.spawn(async move {
                let _permit = sem.acquire().await.expect("网络并发信号量已被关闭");
                run_one(&client, &settings, task, retry).await
            });
        }
    }

    // 第二段 drain：回收未命中回落任务（第一段已把所有 run_one 任务 drain 完，此处仅追赶回落任务）
    while let Some(res) = handles.join_next().await {
        if let Ok(r) = res {
            checked.push(r);
        }
    }

    // 合并 GraphQL 批量命中结果
    checked.append(&mut checked_from_graphql);

    BatchOutcome { checked, manual }
}