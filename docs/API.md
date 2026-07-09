<!-- ================================================================ -->
<!-- My-AUR-Helper Tauri Command API 文档                              -->
<!-- 本文档列出了所有前端可调用的 Tauri IPC 命令、参数和返回值类型      -->
<!-- 供前端开发者和后端贡献者参考，确保前后端接口一致                  -->
<!-- ================================================================ -->

# Tauri Command API

前后端通过 `@tauri-apps/api/core` 的 `invoke()` 通信。

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

### search_software
搜索软件包。
- 参数: `{ keyword: string }`
- 返回: `SoftwareInfo[]`

### add_software
添加新的软件包。
- 参数: `{ pkgname, upstream_url?, package_type, checker_type, check_test_versions, check_binary_files, auto_check_enabled, license_id?, language_id? }`
- 返回: `number` (software_id)

### update_software
更新软件包信息。
- 参数: `{ software_id, pkgname, upstream_url?, package_type, checker_type, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, license_id?, language_id? }`
- 返回: `void`

### delete_software
删除单个软件包。
- 参数: `{ software_id: number }`
- 返回: `void`

### batch_delete_software
批量删除软件包。
- 参数: `{ ids: number[] }`
- 返回: `number` (删除数量)

### set_software_license
设置软件包的 License。
- 参数: `{ software_id: number, license_id: number | null }`
- 返回: `void`

### set_software_language
设置软件包的编程语言。
- 参数: `{ software_id: number, language_id: number | null }`
- 返回: `void`

## 软件包同步 (commands/software_sync.rs)

### sync_from_aur
从 AUR 批量同步软件包信息（使用批量查询 API）。
- 参数: 无
- 返回: `number` (同步数量)
- 说明: 使用 `get_packages_info` 批量查询所有本地软件包的 AUR 信息，一次性获取后逐个更新

### sync_from_pkgbuild
从 PKGBUILD 文件同步软件包。
- 参数: `{ pkgname?: string }`
- 返回: `number` (同步数量)
- 事件: `sync-progress` — 同步进度

### update_aur_info
更新指定软件包的 AUR 信息。
- 参数: `{ pkgname_list?: string[] }`
- 返回: `number` (更新数量)
- 说明: 支持批量更新，传入多个包名可一次性更新多个包的 AUR 信息

## 版本检查 (commands/software_check.rs)

### check_upstream_version
检查单个软件包的上游版本。
- 参数: `{ pkgname: string }`
- 返回: `string` (上游版本号)

### check_all_upstream
检查所有软件包的上游版本。
- 参数: 无
- 返回: `[string, string][]` (包名, 版本) 数组

### check_selected_upstream
检查选中的软件包上游版本。
- 参数: `{ pkgname_list: string[] }`
- 返回: `[string, string][]` (包名, 版本) 数组

## 文件操作 (commands/files.rs)

### copy_file
复制文件或目录。
- 参数: `{ src: string, dst: string }`
- 返回: `void`

### move_file
移动文件或目录。
- 参数: `{ src: string, dst: string }`
- 返回: `void`

### delete_file
删除文件或目录。
- 参数: `{ path: string }`
- 返回: `void`

### delete_directory
删除目录（仅限目录）。
- 参数: `{ path: string }`
- 返回: `void`

### create_directory
创建目录（支持递归创建）。
- 参数: `{ path: string }`
- 返回: `void`

### read_file
读取文件内容为字符串。
- 参数: `{ path: string }`
- 返回: `string`

### list_directory
列出目录内容。
- 参数: `{ path: string }`
- 返回: `DirEntry[]`

### file_exists
检查文件是否存在。
- 参数: `{ path: string }`
- 返回: `boolean`

### file_metadata
获取文件元信息。
- 参数: `{ path: string }`
- 返回: `FileMetadata`

### batch_delete
批量删除文件或目录。
- 参数: `{ paths: string[] }`
- 返回: `BatchDeleteResult`

## 包文件扫描 (commands/files_scan.rs)

### scan_pkg_files
扫描目录中的 .pkg.tar 文件。
- 参数: `{ directory: string }`
- 返回: `PkgFileInfo[]`

## 目录扫描 (commands/scan.rs)

### scan_directory
扫描指定目录（单层）。
- 参数: `{ path: string }`
- 返回: 目录内容

### scan_directory_recursive
递归扫描目录树。
- 参数: `{ path: string }`
- 返回: 目录树结构

### scan_pkg_files_cmd
扫描 .pkg.tar.zst 包文件。
- 参数: `{ directory: string }`
- 返回: `PkgFileInfo[]`

## 系统命令 (commands/sys_command.rs)

### run_command
执行任意系统命令。
- 参数: `{ command: string, args?: string[] }`
- 返回: `{ stdout: string, stderr: string, code: number }`

### install_package
安装软件包。
- 参数: `{ packages: string[] }`
- 返回: 执行结果

### remove_package
卸载软件包。
- 参数: `{ packages: string[] }`
- 返回: 执行结果

### clean_cache
清理 pacman 缓存。
- 参数: 无
- 返回: 执行结果

### get_package_version
获取已安装包的版本。
- 参数: `{ pkgname: string }`
- 返回: `string`

### list_installed_packages
列出所有已安装包。
- 参数: 无
- 返回: `string[]`

### sync_database
同步 pacman 数据库。
- 参数: 无
- 返回: 执行结果

### makepkg
运行 makepkg 构建。
- 参数: `{ path: string, args?: string[] }`
- 返回: 执行结果

## 备份管理 (commands/backup.rs)

### run_backup
执行备份操作。
- 参数: 无
- 返回: 执行结果

## 代理管理 (commands/proxy.rs)

### get_proxies
获取所有代理列表。
- 参数: 无
- 返回: `ProxyInfo[]`

### fetch_proxy_sources
从 Greasyfork 获取代理源。
- 参数: 无
- 返回: `number` (获取数量)

### test_proxy
测试代理延迟。
- 参数: `{ proxy_id: number }`
- 返回: `ProxyTestResult`

### set_proxy_active
设置代理启用状态。
- 参数: `{ proxy_id: number, is_active: boolean }`
- 返回: `void`

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

## 枚举值管理 (commands/enums.rs)

### get_licenses
获取所有 License。
- 参数: 无
- 返回: `License[]`

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
- 返回: `Language[]`

### upsert_language
添加或更新编程语言。
- 参数: `{ name, description?, file_extensions?, ... }`
- 返回: `number` (id)

### delete_language
删除编程语言。
- 参数: `{ id: number }`
- 返回: `void`