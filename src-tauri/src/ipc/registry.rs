/**
 * ipc/registry.rs - Tauri-specta IPC 命令注册表与 bindings 导出
 *
 * 功能：
 * - build_specta_router 是 88 个 Tauri 命令的唯一注册点，collect_commands!
 *   统一收集命令签名/参数/返回值，供 invoke_handler 分发与 TS bindings 生成共用
 * - export_bindings 在 debug 构建或测试时把类型生成到前端 src/bindings.ts
 *
 * 工作流程：
 * 1. Builder::new().dangerously_cast_bigints_to_number().commands(collect_commands![...])
 * 2. run() 复用同一 router 的 invoke_handler，测试调用 export_bindings 刷新生成物
 *
 * 适用场景：IPC 铁律闭环——命令双宏标注、唯一注册点、bindings 禁手改。
 * 注意：88 条命令整体搬迁，顺序与内容零改动，确保 bindings diff 为空。
 */
use tauri_specta::{collect_commands, Builder};

// collect_commands! 中的 `commands::...` 原在 lib.rs（crate root）解析；
// 本模块下沉到 ipc/ 后需显式引入，确保 88 条命令路径写法与顺序零改动。
use crate::commands;

/// 导出 TypeScript bindings 到前端 src 目录。
///
/// 路径基于 Cargo manifest 定位，避免依赖运行时工作目录；
/// 仅在 debug 构建自动触发，release 构建可通过同名测试手动刷新。
#[cfg(any(debug_assertions, test))]
pub(crate) fn export_bindings() {
    // PathBuf 仅在本 debug/test 函数内使用；置于文件顶层会在 release 构建
    // 因该 cfg 分支不编译而触发 unused_imports 告警，故就近引入。
    use std::path::PathBuf;

    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts");
    build_specta_router()
        .export(specta_typescript::Typescript::default(), &out)
        .unwrap_or_else(|e| panic!("生成 TypeScript bindings 失败: {:?}", e));
}

/// 构建 tauri-specta 的 IPC 路由与类型收集器。
///
/// 命令需同时标注 `#[tauri::command]` 与 `#[specta::specta]`，
/// 由 collect_commands! 统一收集命令签名、参数与返回值类型，
/// 供 invoke_handler 分发与 TypeScript bindings 生成共用同一份定义。
pub(crate) fn build_specta_router() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        // 项目内 i64 均为 ID/计数，值域远小于 2^53，经 JSON 传输不会丢精度，
        // 故按官方内置开关把 BigInt 风格整数导出为 TS number。
        .dangerously_cast_bigints_to_number()
        .commands(collect_commands![
            commands::software::add_software,
            commands::software::update_software,
            commands::software::list_software,
            commands::software::list_software_view,
            commands::software::get_software_list_entry,
            commands::software::get_software,
            commands::software::get_software_detail,
            commands::software::get_prev_next_software,
            commands::software::search_software,
            commands::software::delete_software,
            commands::software::batch_delete_software,
            commands::software::set_software_license,
            commands::software::set_software_language,
            commands::sysops::software_sync::aur::sync_from_aur,
            commands::sysops::software_sync::aur::update_aur_info,
            commands::sysops::software_sync::maintainer_sync::sync_packages_by_maintainer,
            commands::sysops::software_sync::import_aur::search_aur_packages,
            commands::sysops::software_sync::import_aur::import_aur_package,
            commands::sysops::software_sync::pkgbuild::sync_from_pkgbuild,
            commands::sysops::software_sync::upstream::check_all_upstream,
            commands::sysops::software_sync::cache_preload::preload_github_tag_cache,
            commands::sysops::software_check_single::check_upstream_version,
            commands::sysops::software_check::selected::check_selected_upstream,
            commands::sysops::software_update::list::list_software_updates,
            commands::sysops::software_update::apply::preview_software_updates,
            commands::sysops::software_update::apply::apply_software_updates,
            commands::sysops::upstream_validate::validate_upstream_urls,
            commands::fileops::scan::scan_pkg_files_cmd,
            commands::fileops::cache_scan::list_cache_software,
            commands::fileops::cache_scan::scan_all_cache_dirs,
            commands::fileops::cache_scan::clear_cache_software,
            commands::fileops::cache_backup::backup_cache_to_existing,
            commands::fileops::cache_backup::backup_cache_to_subdirectory,
            commands::fileops::backup_scan::scan_backup_directory,
            commands::fileops::backup_scan::list_backup_subdirectories,
            commands::fileops::backup_dedup::deduplicate_backups,
            commands::sysops::backup_basic::list_backup_software,
            commands::sysops::backup_basic::clear_backup_software,
            commands::sysops::backup_basic::delete_backup,
            commands::sysops::backup_install::get_package_file_info,
            commands::sysops::backup_install::check_sudoers_config,
            commands::sysops::backup_install::get_sudoers_command,
            commands::sysops::backup_install::install_backup_package,
            commands::proxy::get_proxies,
            commands::proxy::fetch_proxy_sources,
            commands::proxy::download_proxy_file,
            commands::proxy::parse_proxy_file,
            commands::proxy::test_proxy,
            commands::proxy::test_proxies_batch,
            commands::proxy::test_proxy_single,
            commands::proxy::set_proxy_active,
            commands::proxy::update_proxy,
            commands::proxy::delete_proxy,
            commands::proxy::clear_proxy_tables,
            commands::sysops::sys_command::get_package_version,
            commands::sysops::sys_command::list_installed_packages,
            commands::sysops::cache_cleanup::clean_system_cache,
            commands::sysops::cache_cleanup::clean_custom_cache_dirs,
            commands::sysops::cache_cleanup::check_cache_cleanup_sudoers,
            commands::sysops::cache_cleanup::get_cache_cleanup_sudoers_command,
            commands::sysops::cache_cleanup::clear_github_tag_cache,
            commands::sysops::cache_cleanup::clear_expired_github_tag_cache,
            commands::sysops::cache_cleanup::get_github_tag_cache_stats,
            commands::sysops::cache_install::get_cache_package_info,
            commands::sysops::cache_install::install_cache_package,
            commands::sysops::cache_install::check_cache_install_sudoers,
            commands::sysops::cache_install::get_cache_install_sudoers_command,
            commands::logs::get_logs,
            commands::logs::get_new_logs,
            commands::logs::clear_logs,
            commands::settings::get_settings,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::apply_log_settings,
            commands::memory_cache::get_memory_cache_stats,
            commands::memory_cache::flush_memory_cache,
            commands::memory_cache::clear_memory_cache,
            commands::enums::get_licenses,
            commands::enums::get_license,
            commands::enums::update_license,
            commands::enums::delete_license,
            commands::enums::sync_licenses_from_spdx,
            commands::enums::add_license,
            commands::enums::get_languages,
            commands::enums::upsert_language,
            commands::enums::delete_language,
            commands::dashboard::get_dashboard_stats,
            commands::fe_log::frontend_log,
        ])
}
