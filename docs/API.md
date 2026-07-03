<!-- ================================================================ -->
<!-- My-AUR-Helper Tauri Command API 文档                              -->
<!-- 本文档列出了所有前端可调用的 Tauri IPC 命令、参数和返回值类型      -->
<!-- 供前端开发者和后端贡献者参考，确保前后端接口一致                  -->
<!-- ================================================================ -->

# Tauri Command API

<!-- 通信机制说明：前端通过 @tauri-apps/api/core 的 invoke 函数调用后端命令 -->
前后端通过 `@tauri-apps/api/core` 的 `invoke()` 通信。

<!-- ========== 包管理：软件包增删改查及版本检查相关命令 ========== -->
## 包管理

### list_packages
获取所有软件包列表。
- 参数: 无
- 返回: `Package[]`
- 命令: `commands::packages::list_packages`

### get_package
获取单个软件包详情。
- 参数: `{ pkgname: string }`
- 返回: `Package | null`

### check_upstream_version
检查某个包的上游版本。
- 参数: `{ pkgname: string }`
- 返回: `string` (上游版本号)
- 错误: 检查失败返回错误信息

### update_package
更新软件包（待实现完整工作流）。
- 参数: `{ pkgname: string }`
- 返回: `string`

<!-- ========== 备份管理：备份配置的增删改查及执行备份 ========== -->
## 备份管理

### get_backup_configs
获取所有备份配置。
- 参数: 无
- 返回: `BackupConfig[]`

### save_backup_config
保存备份配置。
- 参数: `{ config: BackupConfig }`
- 返回: `void`

### run_backup
执行指定配置的备份。
- 参数: `{ configId: number }`
- 返回: `BackupResult`

<!-- ========== 代理管理：代理源的获取、测试和启用/禁用 ========== -->
## 代理管理

### get_proxies
获取所有代理源。
- 参数: 无
- 返回: `ProxySource[]`

### fetch_proxy_sources
从 Greasyfork userscript 获取代理列表。
- 参数: 无
- 返回: `number` (获取到的代理数量)

### test_proxy
测试代理延迟。
- 参数: `{ proxyUrl: string }`
- 返回: `number` (延迟 ms)

### set_proxy
启用/禁用代理（待实现）。
- 参数: `{ proxyId: number, isActive: boolean }`
- 返回: `void`

<!-- ========== 日志管理：日志查询和清理命令 ========== -->
## 日志管理

### get_logs
获取日志列表。
- 参数: `{ limit?: number }`
- 返回: `LogEntry[]`

### clear_logs
清空日志。
- 参数: 无
- 返回: `void`
