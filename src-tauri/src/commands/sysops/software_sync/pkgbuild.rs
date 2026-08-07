/**
 * pkgbuild.rs - PKGBUILD 文件同步命令
 *
 * 功能：扫描 AUR 软件包目录中的 PKGBUILD 文件，解析并同步软件包信息到数据库。
 * 支持进度事件推送，前端可实时显示同步进度。
 *
 * 工作流程：
 * 1. 从配置读取 AUR 软件包目录路径
 * 2. 遍历目录下的所有子目录（每个子目录代表一个软件包）
 * 3. 对每个子目录调用 aur::read_pkgbuild 解析 PKGBUILD 文件
 * 4. 查询数据库中已有的记录，保留用户手动设置的字段
 * 5. 将解析结果写入数据库
 * 6. 通过 Tauri emit 推送进度事件到前端
 *
 * 注意：以下字段始终使用 PKGBUILD 解析值（可从 PKGBUILD 准确推断）：
 * - package_type_id（包类型）
 * - checker_type_id（检查器类型）
 * - check_test_versions（检查测试版本）
 * - check_binary_files（检查二进制文件）
 * - auto_check_enabled（自动检查）
 *
 * 以下字段优先保留用户设置，仅在为空时用 PKGBUILD 解析值填充：
 * - upstream_url（上游 URL）
 * - version_extract_regex（版本提取正则）
 * - language_ids（编程语言列表）
 */
use log::{error, info};
use tauri::{Emitter, State};

use crate::aur;
use crate::errors::{AppError, AppResult};
use crate::AppState;

/// 从 PKGBUILD 文件同步软件包信息
///
/// # 参数
/// - `state`: Tauri 应用状态，包含数据库连接
/// - `app`: Tauri AppHandle，用于发送进度事件
/// - `pkgname`: 可选的软件包名称过滤器，如果指定则只同步该软件包
///
/// # 返回
/// - `Ok(count)`: 成功同步的软件包数量
/// - `Err(e)`: 同步过程中发生错误
#[tauri::command]
pub async fn sync_from_pkgbuild(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    pkgname: Option<String>,
) -> AppResult<i64> {
    info!("正在从 PKGBUILD 文件同步软件包");
    let aur_dir = {
        let db = state.db.lock()?;
        db.get_setting("aur_packages_dir")?
            .map(|s| s.value)
            .unwrap_or_default()
    };
    if aur_dir.is_empty() {
        return Err(AppError::ConfigError("AUR 文件目录未配置".to_string()));
    }
    let path = std::path::Path::new(&aur_dir);

    let mut dir_entries = Vec::new();
    let mut entries = tokio::fs::read_dir(&path).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            if let Some(ref filter_name) = pkgname {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if dir_name != *filter_name {
                    continue;
                }
            }
            dir_entries.push(entry);
        }
    }

    let total = dir_entries.len();
    info!("找到 {} 个软件包目录待同步", total);

    let _ = app.emit(
        "sync-progress",
        serde_json::json!({
            "current": 0,
            "total": total,
            "pkgname": "",
            "message": format!("开始同步，共 {} 个包", total),
        }),
    );

    let mut count = 0i64;
    for (i, entry) in dir_entries.iter().enumerate() {
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let pkg_path = entry.path();

        let _ = app.emit(
            "sync-progress",
            serde_json::json!({
                "current": i,
                "total": total,
                "pkgname": dir_name,
                "message": format!("[{}/{}] 正在处理: {}", i + 1, total, dir_name),
            }),
        );

        match aur::read_pkgbuild(&pkg_path).await {
            Ok(Some((parsed_sw, _))) => {
                let pkg_type = match parsed_sw.package_type_id.as_id() {
                    2 => "二进制包",
                    3 => "Git",
                    4 => "AppImage",
                    _ => "编译安装",
                };
                info!(
                    "[{}/{}] {} - 类型: {}, 自动检查: {}, 检查测试版: {}, 检查二进制: {}",
                    i + 1,
                    total,
                    parsed_sw.pkgname,
                    pkg_type,
                    parsed_sw.auto_check_enabled,
                    parsed_sw.check_test_versions,
                    parsed_sw.check_binary_files
                );

                let db = state.db.lock()?;
                // 查询数据库中已有的记录，保留用户手动设置的字段
                let final_sw = match db.get_software_by_name(&parsed_sw.pkgname)? {
                    Some(mut existing) => {
                        // 保留用户手动设置的字段，仅在字段为空时用 PKGBUILD 解析值填充
                        if existing.upstream_url.is_none() {
                            existing.upstream_url = parsed_sw.upstream_url;
                        }
                        if existing.version_extract_regex.is_none() {
                            existing.version_extract_regex = parsed_sw.version_extract_regex;
                        }
                        if existing.language_ids.is_empty() {
                            existing.language_ids = parsed_sw.language_ids;
                        }
                        // 保留用户手动设置的检查选项
                        // package_type_id、checker_type_id、check_test_versions、check_binary_files、auto_check_enabled
                        // 始终使用 PKGBUILD 解析的值（因为这些值可以从 PKGBUILD 中准确推断）
                        existing.package_type_id = parsed_sw.package_type_id;
                        existing.checker_type_id = parsed_sw.checker_type_id;
                        existing.check_test_versions = parsed_sw.check_test_versions;
                        existing.check_binary_files = parsed_sw.check_binary_files;
                        existing.auto_check_enabled = parsed_sw.auto_check_enabled;
                        existing
                    }
                    None => parsed_sw, // 新包，直接使用解析结果
                };

                if let Err(e) = db.upsert_software(&final_sw) {
                    error!(
                        "[{}/{}] {} - 写入数据库失败: {}",
                        i + 1,
                        total,
                        final_sw.pkgname,
                        e
                    );
                }
                count += 1;

                let _ = app.emit(
                    "sync-progress",
                    serde_json::json!({
                        "current": i + 1,
                        "total": total,
                        "pkgname": final_sw.pkgname,
                        "message": format!("[{}/{}] 已完成: {}", i + 1, total, dir_name),
                    }),
                );
            }
            Ok(None) => {
                info!(
                    "[{}/{}] {} - 跳过（无 PKGBUILD 文件）",
                    i + 1,
                    total,
                    dir_name
                );
                let _ = app.emit("sync-progress", serde_json::json!({
                    "current": i + 1,
                    "total": total,
                    "pkgname": dir_name,
                    "message": format!("[{}/{}] 跳过: {} (无 PKGBUILD)", i + 1, total, dir_name),
                }));
            }
            Err(e) => {
                error!("[{}/{}] {} - 解析失败: {}", i + 1, total, dir_name, e);
                let _ = app.emit(
                    "sync-progress",
                    serde_json::json!({
                        "current": i + 1,
                        "total": total,
                        "pkgname": dir_name,
                        "message": format!("[{}/{}] 失败: {} ({})", i + 1, total, dir_name, e),
                    }),
                );
            }
        }
    }

    let _ = app.emit(
        "sync-progress",
        serde_json::json!({
            "current": total,
            "total": total,
            "pkgname": "",
            "message": format!("同步完成，成功 {} 个", count),
        }),
    );

    info!("已从 PKGBUILD 文件同步 {} 个软件包", count);
    Ok(count)
}
