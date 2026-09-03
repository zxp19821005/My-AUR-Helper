/**
 * cache.rs - GitHub tags 缓存增量校验辅助函数
 *
 * 功能：对 GitHub 仓库列表执行缓存状态校验，决定哪些包可以直接从缓存获取版本。
 *
 * 工作流程：
 * 1. 遍历每个 (owner, repo)，调 check_and_extend_cache 判断缓存状态
 * 2. Hit/Extend：加入 skip_keys（后续跳过 batch 检查），并从缓存重算版本生成结果条目
 * 3. Recalculate/Deleted/Miss：不跳过，进入后续网络检查
 *
 * 设计要点：
 * - 此函数为同步函数（含 blocking HTTP + DB 操作），调用方需在 lock 内串行执行
 * - GitHub 仓库数量有限，串行校验完全可接受
 * - 缓存命中结果提前生成，避免这些包被 batch 引擎遗漏（不产生任何网络请求）
 */
use log::{info, warn};
use std::collections::HashSet;

use super::batch::PackageTask;
use super::utils::UpstreamCheckResult;
use crate::checkers::utils::extract_owner_repo;
use crate::db::github_tag_cache::{
    decode_tags, recompute_version_from_tags, CacheCheckResult,
};
use crate::models::{CheckerType, PackageType};
use crate::db::Database;

/// 缓存校验结果汇总
pub struct CacheHitSummary {
    /// 需要跳过的 (owner, repo) 集合（缓存命中或顺延，无需网络请求）
    pub skip_keys: HashSet<(String, String)>,
    /// 直接从缓存生成的检查结果（已包含上游版本，is_outdated 由调用方统一计算）
    pub cache_hit_results: Vec<UpstreamCheckResult>,
}

/// 对 GitHub 仓库列表执行增量缓存校验。
///
/// # 参数
/// - `db`: 数据库连接（不可 Send，必须在同步上下文中使用）
/// - `tasks`: 待检查的任务列表（用于按 (owner, repo) 反查属于该仓库的包）
/// - `github_repos`: 去重后的仓库列表
///
/// # 返回
/// - `CacheHitSummary`: skip_keys 和缓存命中的结果条目
pub fn check_github_cache(
    db: &Database,
    tasks: &[PackageTask],
    github_repos: &[(String, String)],
) -> CacheHitSummary {
    let mut skip: HashSet<(String, String)> = HashSet::new();
    let mut hit_results: Vec<UpstreamCheckResult> = Vec::new();

    for (owner, repo) in github_repos {
        // 找该仓库对应包的任务列表（用于生成命中结果）
        let repo_tasks: Vec<&PackageTask> = tasks
            .iter()
            .filter(|t| {
                matches!(t.checker_type, CheckerType::GitHubTags | CheckerType::GitHubAPI)
                    && t.package_type != PackageType::Git
                    && extract_owner_repo(&t.upstream_url)
                        .is_some_and(|(o, r)| o == *owner && r == *repo)
            })
            .collect();

        // check_and_extend_cache 的 regex 参数仅用于过期时的 Recalculate 路径
        // Hit 路径不使用 regex；此处传 None，因为 cached_version 为 None 时
        // Recalculate 会用 None 取第一个 tag 作为 cached_version（仓库级最新）
        match db.check_and_extend_cache(owner, repo, None) {
            Ok(result) => match result {
                CacheCheckResult::Hit | CacheCheckResult::Extend => {
                    // 从缓存提取 tags，为每个包用各自正则提取版本
                    match db.get_cache(owner, repo) {
                        Ok(Some(cache_row)) => {
                            let tags = decode_tags(&cache_row.data_json);
                            if tags.is_empty() {
                                warn!(
                                    "[GitHub Cache] {}:{} 缓存有效但 tags 解析为空，{} 个包进入网络检查",
                                    owner, repo, repo_tasks.len()
                                );
                                // 不加入 skip，回退网络检查
                            } else {
                                skip.insert((owner.clone(), repo.clone()));
                                for task in &repo_tasks {
                                    let pkg_regex = task.version_extract_regex.as_deref();
                                    let version =
                                        recompute_version_from_tags(&tags, pkg_regex);
                                    if let Some(v) = version {
                                        hit_results.push(UpstreamCheckResult {
                                            pkgname: task.pkgname.clone(),
                                            software_id: task.software_id,
                                            upstream_version: v,
                                            is_outdated: false,
                                            license_spdx_id: None,
                                            language_names: Vec::new(),
                                            _all_tags: None,
                                            _owner: String::new(),
                                            _repo: String::new(),
                                        });
                                    }
                                }
                                let result_type = if matches!(result, CacheCheckResult::Hit) {
                                    "缓存命中"
                                } else {
                                    "缓存顺延"
                                };
                                info!(
                                    "[GitHub Cache] {}:{} {}，{} 个包从缓存获取版本",
                                    owner, repo, result_type, repo_tasks.len()
                                );
                            }
                        }
                        _ => warn!("[GitHub Cache] {}:{} 缓存读取失败", owner, repo),
                    }
                }
                CacheCheckResult::Recalculate { ref new_version } => {
                    // 版本有变化，tags 可能已过期，需要重新拉取
                    if !repo_tasks.is_empty() {
                        info!(
                            "[GitHub Cache] {}:{} 版本变化({})，需要重新拉取，{} 个包进入检查",
                            owner, repo, new_version, repo_tasks.len()
                        );
                    }
                }
                CacheCheckResult::Deleted | CacheCheckResult::Miss => {
                    // 无缓存或已清除，不跳过
                    if !repo_tasks.is_empty() {
                        info!(
                            "[GitHub Cache] {}:{} 无有效缓存，{} 个包直接进入网络检查",
                            owner, repo, repo_tasks.len()
                        );
                    }
                }
            },
            Err(e) => {
                warn!("[GitHub Cache] {}:{} 校验失败: {}", owner, repo, e);
            }
        }
    }

    info!(
        "[GitHub Cache] 跳过 {} 个仓库（缓存有效或已顺延），{} 个包直接从缓存获取版本",
        skip.len(),
        hit_results.len()
    );
    CacheHitSummary {
        skip_keys: skip,
        cache_hit_results: hit_results,
    }
}
