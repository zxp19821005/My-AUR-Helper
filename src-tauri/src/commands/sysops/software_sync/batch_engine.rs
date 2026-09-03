/**
 * batch_engine.rs - 上游版本批量检查：常量、结构体与辅助函数
 *
 * 从 batch.rs 拆分而来，包含：
 * - PackageTask: 单次批量检查的任务结构体
 * - BatchOutcome: 批量检查结果
 * - MAX_BROWSER_CONCURRENCY / MAX_NETWORK_CONCURRENCY: 并发控制常量
 * - drain_fallback_results: 第二段 drain（回收回填结果 + 回落 REST）
 *
 * 模块设计原则：
 * - 本文件仅定义类型常量和辅助函数
 * - 核心调度逻辑 (batch_check_upstream) 保留在 batch.rs
 * - 所有文件严格控制在 300 行以内
 */
use std::collections::HashMap;

use log::warn;
use tokio::task::JoinSet;

use super::utils::UpstreamCheckResult;
use crate::checkers::github::graphql_batch::{GithubBatchItem, RepoCache};

/// 浏览器检查器最大并发数：每个 headless Chrome 进程内存占用较大，
/// 过高会导致 OOM / 文件描述符耗尽。后续可改为读取设置项覆盖。
pub const MAX_BROWSER_CONCURRENCY: usize = 4;

/// 其余 HTTP 类检查器全局最大并发数：控制对上游的请求压力，规避限流。
pub const MAX_NETWORK_CONCURRENCY: usize = 16;

/// 单次批量检查待执行的软件包任务
///
/// 由 SoftwareInfo 映射而来，携带检查器执行所需的全部字段。
#[derive(Clone)]
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
    pub checker_type: crate::models::CheckerType,
    /// 软件包类型（透传备用，为后续按包类型做批量优化预留）
    pub package_type: crate::models::PackageType,
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
    /// GitHub tags 快照缓存，供调用方写盘（避免跨 await 持有 db 引用）
    pub github_cache_map: HashMap<(String, String), RepoCache>,
}

/// 第二段 drain：回收回填结果 + 回落 REST 任务
///
/// 将 JoinSet 中的任务逐个取出，区分「tags 回填结果」与「普通 REST 结果」，
/// 回填结果会触发 select_version 重新计算版本并写入 checked。
///
/// # 参数
/// - `handles`: 承载所有 run_one/fill_tags 任务的 JoinSet
/// - `checked`: 已收集的检查结果（追加写入）
/// - `cache_map`: GitHub tags 缓存映射（回填后更新 all_tags 字段）
/// - `github_items`: GraphQL 批量检查输入列表，用于按仓库过滤包
///
/// # 返回
/// 一个临时的 BatchOutcome，仅包含更新后的 github_cache_map
pub(crate) async fn drain_fallback_results(
    handles: &mut JoinSet<UpstreamCheckResult>,
    checked: &mut Vec<UpstreamCheckResult>,
    cache_map: &mut HashMap<(String, String), RepoCache>,
    github_items: &[GithubBatchItem],
) -> BatchOutcome {
    use crate::checkers::github::graphql_batch_helpers::select_version;

    while let Some(res) = handles.join_next().await {
        match res {
            Ok(result) => {
                // 回填任务结果：提取 all_tags 并产出各包版本
                if let Some(tags_result) = result._all_tags {
                    let owner = result._owner;
                    let repo = result._repo;
                    if let Some(cache) = cache_map.get_mut(&(owner.clone(), repo.clone())) {
                        cache.all_tags = tags_result;
                        // 找出属于本仓库且需要回填的包（github_items 中）
                        let pkgs: Vec<_> = github_items
                            .iter()
                            .filter(|t| t.owner == owner && t.repo == repo)
                            .collect();
                        for task in pkgs {
                            let version =
                                select_version(&cache.snapshot, task, Some(&cache.all_tags));
                            checked.push(UpstreamCheckResult {
                                pkgname: task.pkgname.clone(),
                                software_id: task.software_id,
                                upstream_version: version.unwrap_or_default(),
                                is_outdated: false,
                                license_spdx_id: cache.snapshot.license.clone(),
                                language_names: cache.snapshot.languages.clone(),
                                _all_tags: None,
                                _owner: String::new(),
                                _repo: String::new(),
                            });
                        }
                    }
                } else {
                    // 普通 REST 任务结果
                    checked.push(result);
                }
            }
            Err(e) => {
                warn!("[批量检查] 回填或回落任务失败: {}", e);
            }
        }
    }

    BatchOutcome {
        checked: Vec::new(),
        manual: Vec::new(),
        github_cache_map: cache_map.clone(),
    }
}
