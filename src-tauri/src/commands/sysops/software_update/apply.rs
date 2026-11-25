/**
 * apply.rs - 软件更新的 dry-run 预览与批量写回
 *
 * 功能：先按包名生成「只改 pkgver / pkgrel」的改写计划（纯读、不落盘），
 *      前端展示 diff 并经用户确认后，才执行备份 + 写回。
 *
 * 安全约束：
 * - 目标版本一律从数据库 upstream_info 重新读取，不信任前端传入值，
 *   避免预览与执行之间版本过期导致写错；
 * - 沿用 pkgbuild_io 的包名白名单与路径越界校验；
 * - 逐包独立返回成败，单包失败不中断整批。
 *
 * 设计说明：写回只作用于本地 PKGBUILD，不改动数据库。AUR 现网版本在
 * 推送前保持不变，列表状态由磁盘 pkgver 实时推导，故无需新增表或更新字段。
 *
 * 适用场景：软件更新页面的「预览变更」与「执行更新」。
 */
use std::path::PathBuf;

use log::info;
use tauri::State;

use crate::errors::{AppError, AppResult};
use crate::models::{SoftwareUpdateApplyResult, SoftwareUpdatePreview};
use crate::versions::UpstreamVersion;
use crate::AppState;

use super::pkgbuild_io::{backup_and_write, build_rewrite, read_local_pkgbuild, validate_pkgname};

/// 单个包的改写计划（内存中间产物，不代表已经写盘）
struct RewritePlan {
    /// PKGBUILD 路径
    path: PathBuf,
    /// 改写后的完整文本
    rewritten: String,
    /// 原 pkgver
    old_pkgver: String,
    /// 原 pkgrel
    old_pkgrel: String,
    /// 目标 pkgver
    new_pkgver: String,
    /// 变更摘要
    diff: String,
}

/// 生成改写计划：校验包名 → 重读上游版本 → 读本地 PKGBUILD → 构造新文本
///
/// # 参数
/// - `state`: 应用状态，提供数据库连接与设置项
/// - `pkgname`: 软件包名称
///
/// # 返回
/// - `RewritePlan`: 可预览、可执行的改写计划
/// - `Err(...)`: 包名非法、缺少上游版本、PKGBUILD 缺失或格式异常
async fn plan_rewrite(state: &AppState, pkgname: &str) -> AppResult<RewritePlan> {
    validate_pkgname(pkgname)?;
    let (root, upstream_opt) = {
        let db = state.db.lock()?;
        let dir = db
            .get_setting("aur_packages_dir")?
            .map(|s| s.value)
            .unwrap_or_default();
        let upstream = db.get_upstream_version_for(pkgname)?;
        (PathBuf::from(dir), upstream)
    };
    let upstream_raw = upstream_opt.ok_or_else(|| {
        AppError::VersionCheckError(format!("{pkgname} 无上游版本信息，请先执行上游检查"))
    })?;
    // 与列表的 target_version 共用静默归一化，保证写回后状态稳定变为 local_updated
    let new_pkgver = UpstreamVersion::normalized(&upstream_raw);
    let local = read_local_pkgbuild(&root, pkgname)
        .await?
        .ok_or_else(|| AppError::FileNotFound(format!("{pkgname} 缺少 PKGBUILD")))?;
    let (rewritten, diff, old_pkgver, old_pkgrel) = build_rewrite(&local.content, &new_pkgver)?;

    Ok(RewritePlan {
        path: local.path,
        rewritten,
        old_pkgver,
        old_pkgrel,
        new_pkgver,
        diff,
    })
}

/// 由改写计划构造可展示的预览条目
fn preview_from_plan(pkgname: &str, plan: &RewritePlan) -> SoftwareUpdatePreview {
    SoftwareUpdatePreview {
        pkgname: pkgname.to_string(),
        pkgbuild_path: plan.path.to_string_lossy().to_string(),
        old_pkgver: plan.old_pkgver.clone(),
        new_pkgver: plan.new_pkgver.clone(),
        old_pkgrel: plan.old_pkgrel.clone(),
        new_pkgrel: "1".to_string(),
        diff: plan.diff.clone(),
        error: None,
    }
}

/// 由错误构造不可预览的占位条目（保证返回顺序与入参一致）
fn preview_from_error(pkgname: &str, err: &AppError) -> SoftwareUpdatePreview {
    SoftwareUpdatePreview {
        pkgname: pkgname.to_string(),
        pkgbuild_path: String::new(),
        old_pkgver: String::new(),
        new_pkgver: String::new(),
        old_pkgrel: String::new(),
        new_pkgrel: String::new(),
        diff: String::new(),
        error: Some(err.to_string()),
    }
}

/// 预览一批软件包的 PKGBUILD 变更（dry-run，绝不写文件）
///
/// # 参数
/// - `state`: Tauri 应用状态
/// - `pkgnames`: 待预览的包名列表
///
/// # 返回
/// - `Vec<SoftwareUpdatePreview>`: 与入参同序；不可预览的条目带 `error`
#[tauri::command]
#[specta::specta]
pub async fn preview_software_updates(
    state: State<'_, AppState>,
    pkgnames: Vec<String>,
) -> AppResult<Vec<SoftwareUpdatePreview>> {
    let mut previews = Vec::with_capacity(pkgnames.len());
    for pkgname in pkgnames {
        match plan_rewrite(&state, &pkgname).await {
            Ok(plan) => previews.push(preview_from_plan(&pkgname, &plan)),
            Err(e) => previews.push(preview_from_error(&pkgname, &e)),
        }
    }
    Ok(previews)
}

/// 执行一批软件包的 PKGBUILD 写回（前端须先展示预览并取得确认）
///
/// # 参数
/// - `state`: Tauri 应用状态
/// - `pkgnames`: 待写回的包名列表
///
/// # 返回
/// - `Vec<SoftwareUpdateApplyResult>`: 与入参同序的逐包结果，含备份路径
#[tauri::command]
#[specta::specta]
pub async fn apply_software_updates(
    state: State<'_, AppState>,
    pkgnames: Vec<String>,
) -> AppResult<Vec<SoftwareUpdateApplyResult>> {
    let mut results = Vec::with_capacity(pkgnames.len());
    for pkgname in pkgnames {
        let result = match plan_rewrite(&state, &pkgname).await {
            Ok(plan) => match backup_and_write(&plan.path, &plan.rewritten).await {
                Ok(backup) => {
                    info!(
                        "[软件更新] {} 写回成功: {} -> {}, pkgrel {} -> 1",
                        pkgname, plan.old_pkgver, plan.new_pkgver, plan.old_pkgrel
                    );
                    SoftwareUpdateApplyResult {
                        pkgname,
                        success: true,
                        message: format!(
                            "{} → {}（pkgrel {} → 1）",
                            plan.old_pkgver, plan.new_pkgver, plan.old_pkgrel
                        ),
                        backup_path: Some(backup),
                    }
                }
                Err(e) => SoftwareUpdateApplyResult {
                    success: false,
                    message: e.to_string(),
                    backup_path: None,
                    pkgname,
                },
            },
            Err(e) => SoftwareUpdateApplyResult {
                success: false,
                message: e.to_string(),
                backup_path: None,
                pkgname,
            },
        };
        results.push(result);
    }
    Ok(results)
}
