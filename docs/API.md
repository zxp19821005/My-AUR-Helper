<!-- ================================================================ -->
<!-- My-AUR-Helper Tauri Command API 文档                              -->
<!-- 本文档列出了所有前端可调用的 Tauri IPC 命令、参数和返回值类型      -->
<!-- 供前端开发者和后端贡献者参考，确保前后端接口一致                  -->
<!-- 命令注册位置：src-tauri/src/lib.rs 的 invoke_handler              -->
<!-- ================================================================ -->

# Tauri Command API

前后端通过 `@tauri-apps/api/core` 的 `invoke()` 通信。
所有命令均在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册。

## 软件包管理 (commands/software.rs)

### list_software
获取所有软件包列表。
- 参数: 无
- 返回: `SoftwareInfo[]`

### list_software_view
获取软件包列表展示数据（含 AUR + Upstream 信息）。
- 参数: 无
- 返回: `SoftwareListEntry[]`

### get_software
根据包名获取单个软件包信息。
- 参数: `{ pkgname: string }`
- 返回: `SoftwareInfo | null`

### get_software_detail
获取软件包完整详情（基本信息 + AUR + 上游）。
- 参数: `{ pkgname: string }`
- 返回: `SoftwareDetail | null`

### get_prev_next_software
获取上一个/下一个软件包包名（详情页导航用）。
- 参数: `{ pkgname: string }`
- 返回: `[string | null, string | null]`

### search_software
搜索软件包。
- 参数: `{ keyword: string }`
- 返回: `SoftwareInfo[]`

### add_software
添加新的软件包。
- 参数: `{ pkgname, upstream_url?, package_type, checker_type, check_test_versions, check_binary_files, auto_check_enabled, language_ids, version_extract_regex? }`
- 返回: `number` (software_id)

### update_software
更新软件包信息。
- 参数: 同 `add_software`，另含 `software_id` 和 `is_outdated`
- 返回: `void`

### set_software_license
设置软件包的 License。
- 参数: `{ software_id: number, license_id?: string }`
- 返回: `void`

### set_software_language
设置软件包的编程语言。
- 参数: `{ software_id: number, language_id?: number }`
- 返回: `void`

### delete_software
删除软件包。
- 参数: `{ software_id: number }`
- 返回: `void`

### batch_delete_software
批量删除软件包。
- 参数: `{ software_ids: number[] }`
- 返回: `number` (删除数量)

## 软件包同步 (commands/sysops/software_sync/)

### sync_from_aur
从 AUR 同步当前用户维护的软件包列表（并行执行）。
- 参数: 无
- 返回: `number` (同步数量)
- 说明: 只更新 `aur_info` 表（描述、版本、依赖等），不更新 `software_info` 表。用户手动设置的上游URL、检查器类型、包类型等字段不会被覆盖。

### update_aur_info
更新软件包的 AUR 信息（批量查询，遵守 aur_batch_size / aur_batch_interval 设置）。
- 参数: `{ pkgname_list?: string[] }` (为空时更新所有)
- 返回: `number` (更新数量)
- 说明: 只更新 `aur_info` 表，不更新 `software_info` 表。

### sync_from_pkgbuild
从本地 PKGBUILD 文件同步软件包信息。
- 参数: `{ pkgname?: string }` (为空时同步所有)
- 返回: `number` (同步数量)
- 说明: 保留用户手动设置的字段（`upstream_url`、`version_extract_regex`、`language_ids`），仅在字段为空时用 PKGBUILD 解析值填充。`package_type_id`、`checker_type_id`、`check_test_versions`、`check_binary_files`、`auto_check_enabled` 始终使用 PKGBUILD 解析值。

### check_all_upstream
并行检查所有软件包的上游版本。
- 参数: 无
- 返回: `[string, string][]` (包名与检查结果)
- 说明: 只有当用户没有手动设置语言列表时，才用自动检测到的语言列表填充 `language_ids` 字段。

## 版本检查 (commands/sysops/software_check.rs)

### check_upstream_version
检查单个软件包的上游版本。
- 参数: `{ pkgname: string }`
- 返回: `string` (检查结果消息)
- 说明: 只有当用户没有手动设置语言列表时，才用自动检测到的语言列表填充 `language_ids` 字段。

### check_selected_upstream
检查选中的软件包上游版本。
- 参数: `{ pkgname_list: string[] }`
- 返回: `[string, string][]` (包名与检查结果)
- 说明: 只有当用户没有手动设置语言列表时，才用自动检测到的语言列表填充 `language_ids` 字段。

## 上游 URL 验证 (commands/sysops/upstream_validate.rs)

### validate_upstream_urls
批量验证软件包的上游 URL 可达性。
- 参数: `{ pkgname_list?: string[] }` (可选，为空时验证所有有上游 URL 的包)
- 返回: `ValidateResult[]` (验证结果数组)
- 事件: `validate-upstream-progress` — 验证进度
- 说明: 并发数 10，超时 10 秒，更新 upstream_info 表的 upstream_url_status 字段

## 扫描和缓存管理 (commands/fileops/)

### scan_pkg_files_cmd (fileops/scan.rs)
扫描目录中的 .pkg.tar.zst 包文件。
- 参数: `{ directory: string }`
- 返回: `PkgFileInfo[]`

### list_cache_software (fileops/cache_scan.rs)
直接读取 cache_software 表（页面打开时）。
- 参数: 无
- 返回: `CacheSoftwareEntry[]`

### scan_all_cache_dirs (fileops/cache_scan.rs)
扫描所有启用的缓存目录并写入数据库。
- 参数: 无
- 返回: `PkgFileInfo[]`

### clear_cache_software (fileops/cache_scan.rs)
清空 cache_software 表。
- 参数: 无
- 返回: `number` (删除数量)

### backup_cache_to_existing (fileops/cache_backup.rs)
备份缓存包到已有备份位置（按包名匹配已有子目录）。
- 参数: `{ filenames: string[], backup_path: string }`
- 返回: `[number, string[]]` (成功数量与失败列表)

### backup_cache_to_subdirectory (fileops/cache_backup.rs)
备份缓存包到指定子目录。
- 参数: `{ filenames: string[], backup_path: string, subdirectory: string }`
- 返回: `[number, string[]]` (成功数量与失败列表)

## 备份管理 (commands/fileops/ + commands/sysops/)

### scan_backup_directory (fileops/backup_scan.rs)
扫描备份目录并写入数据库。
- 参数: `{ backup_path: string }`
- 返回: `number` (新增记录数)

### list_backup_subdirectories (fileops/backup_scan.rs)
获取备份目录的子目录列表。
- 参数: 无
- 返回: `string[]`

### deduplicate_backups (fileops/backup_dedup.rs)
软件去重（保留最新版本，删除旧文件和记录）。
- 参数: `{ backup_path: string }`
- 返回: `DeduplicateResult`

### list_backup_software (sysops/backup_basic.rs)
列出所有备份记录（含软件包名称）。
- 参数: 无
- 返回: `BackupSoftwareEntry[]`

### clear_backup_software (sysops/backup_basic.rs)
清空备份表（仅删除数据库记录，不删除磁盘文件）。
- 参数: 无
- 返回: `number` (删除数量)

### delete_backup (sysops/backup_basic.rs)
删除单个备份记录（及对应磁盘文件）。
- 参数: `{ id: number, backup_path: string }`
- 返回: `void`

### get_package_file_info (sysops/backup_install.rs)
获取备份包文件信息（pacman -Qip）。
- 参数: `{ full_path: string }`
- 返回: `string` (包信息文本)

### check_sudoers_config (sysops/backup_install.rs)
检测是否已配置 pacman 免密 sudoers。
- 参数: 无
- 返回: `boolean`

### get_sudoers_command (sysops/backup_install.rs)
获取配置 sudoers 的命令文本。
- 参数: 无
- 返回: `string`

### install_backup_package (sysops/backup_install.rs)
安装备份包（pacman -U）。
- 参数: `{ full_path: string }`
- 返回: `string` (安装输出)

## 代理管理 (commands/proxy.rs)

### get_proxies
获取所有代理列表。
- 参数: 无
- 返回: `ProxyInfo[]`

### fetch_proxy_sources
从 Greasyfork 获取代理源。
- 参数: 无
- 返回: `number` (获取数量)

### download_proxy_file
下载代理配置文件。
- 参数: 无
- 返回: `number` (下载字节数/数量)

### parse_proxy_file
解析已下载的代理文件。
- 参数: 无
- 返回: `number` (解析数量)

### test_proxy
测试代理延迟。
- 参数: `{ proxy_url: string }`
- 返回: `number` (延迟毫秒)

### test_proxies_batch
批量测试代理。
- 参数: 见 `proxy.rs`（代理 ID 列表）
- 返回: `ProxyTestResult[]`

### test_proxy_single
单个测试代理并写入结果。
- 参数: `{ proxy_id: number }`
- 返回: `ProxyTestResult`

### set_proxy_active
设置代理启用状态。
- 参数: `{ proxy_id: number, is_active: boolean }`
- 返回: `void`

### delete_proxy
删除代理。
- 参数: `{ proxy_id: number }`
- 返回: `void`

## 系统命令 (commands/sysops/sys_command.rs)

### get_package_version
获取已安装包的版本。
- 参数: `{ pkgname: string }`
- 返回: `string`

### list_installed_packages
列出所有已安装包。
- 参数: 无
- 返回: `string[]`

## 日志管理 (commands/logs.rs)

### get_logs
获取日志列表。
- 参数: `{ limit?: number }`
- 返回: `LogEntry[]`

### clear_logs
清空日志。
- 参数: 无
- 返回: `void`

## 设置管理 (commands/settings.rs)

### get_settings
获取所有设置。
- 参数: 无
- 返回: `Setting[]`

### get_setting
获取单个设置。
- 参数: `{ key: string }`
- 返回: `Setting | null`

### set_setting
设置配置值。
- 参数: `{ key: string, value: string }`
- 返回: `void`

### apply_log_settings
应用日志轮转设置。
- 参数: `{ max_size: number, max_files: number }`
- 返回: `void`

## 枚举值管理 (commands/enums.rs)

### get_licenses
获取所有 License。
- 参数: 无
- 返回: `EnumLicense[]`

### sync_licenses_from_spdx
从 SPDX 同步 License。
- 参数: 无
- 返回: `number` (同步数量)

### add_license
添加 License。
- 参数: `{ spdx_id, full_name, url?, ... }`
- 返回: `number` (id)

### get_languages
获取所有编程语言。
- 参数: 无
- 返回: `EnumProgrammingLanguage[]`

### upsert_language
添加或更新编程语言。
- 参数: `{ name, description?, file_extensions?, ... }`
- 返回: `number` (id)

### delete_language
删除编程语言。
- 参数: `{ name: string }`
- 返回: `void`

---

**最后更新**: 2026-07-30
