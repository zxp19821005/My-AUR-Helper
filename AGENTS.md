<!-- ================================================================ -->
<!-- AI 助手配置指南 / AGENTS.md                                       -->
<!-- 本文件为 AI 编码助手提供项目上下文信息，                          -->
<!-- 包括技术栈、代码规范、关键文件位置和开发命令等。                  -->
<!-- AI 在生成代码时应参考本文件以确保符合项目约定。                  -->
<!-- ================================================================ -->

# AI 助手配置指南

<!-- ========== 项目概述 ========== -->
## 项目概述
My-AUR-Helper 是一个基于 Tauri 的跨平台桌面应用，主要用于：
- **AUR 软件包管理**：搜索、安装、更新和卸载 AUR 软件包
- **版本检查**：支持多种上游版本源（GitHub、Gitee、GitLab、HTTP等）
- **本地备份**：管理软件包的备份和恢复
- **代理配置**：支持 HTTP/SOCKS 代理设置和测试

**架构分层**：
- 前端：Vue 3 + TypeScript（用户界面）
- 后端：Rust + Tauri（系统操作和业务逻辑）
- 数据层：SQLite（持久化存储）

<!-- ========== 技术栈：列出项目核心技术 ========== -->
## 技术栈
- 后端: Rust + Tauri v2
  - HTTP 客户端: reqwest
  - 序列化: serde (JSON)
  - 数据库: rusqlite + diesel_migrations
  - 日志: tracing + tracing-subscriber
  - AUR RPC: 自定义实现
- 前端: Vue 3 + TypeScript + Vite
  - 状态管理: Pinia
  - 路由: Vue Router
  - 样式: TailwindCSS 3
  - UI 组件: 自定义组件库
- 构建工具: pnpm + cargo

<!-- ========== 代码规范：AI 编码时必须遵守的规则 ========== -->
## 代码规范（强制）

<!-- 文件组织原则：确保代码可维护性和模块化 -->
### 文件组织原则
1. **单一职责**: 每个文件只负责一个功能模块
2. **行数限制**: 单个文件不超过 300 行
3. **模块拆分**: 超过 300 行的文件必须拆分为多个独立文件
4. **代码重用**: 优先提取通用组件和工具函数，避免重复代码
5. **命名一致**: 文件名、函数名、组件名需与功能模块统一

<!-- 拆分规则：具体模块的拆分方法 -->
### 拆分规则
- `db/mod.rs` 按表拆分: `db/packages.rs`, `db/proxies.rs` 等
- Vue 组件拆分: 通用组件提取到 `src/components/`

### Rust 编码规范
- 使用 `cargo fmt` 进行代码格式化
- 使用 `cargo clippy` 进行 lint 检查
- 函数命名：snake_case
- 结构体/枚举命名：PascalCase
- 使用 `Result<T, Error>` 进行错误处理
- 异步代码使用 `async/await`
- **模块设计原则（强制）**：
  - `mod.rs` 仅负责模块声明和导出，不包含任何具体实现
  - 每个子文件负责单一功能，保持代码可维护性
  - 所有文件严格控制在 300 行以内
- **注释规范（强制）**：
  - 所有文件必须在开头添加文件级注释，说明功能、工作流程适用场景
  - 所有公开函数（`pub fn`）必须添加文档注释（`///`），包含参数、返回值说明
  - 所有结构体/枚举必须添加注释说明用途
  - 复杂逻辑必须添加行内注释（`//`）解释实现思路
  - 注释语言：中文（与项目文档保持一致）
- **安全规范（强制）**：
  - 禁止注册任意命令执行函数（如 `run_command`）
  - 文件操作必须有路径验证和沙箱限制
  - SQL 查询禁止使用 `format!` 拼接用户输入，必须使用参数化查询
  - 前端 IPC 命令仅暴露必要功能，未使用的危险命令必须移除
  - 敏感信息（凭据、密钥、代理 URL）禁止写入日志
  - Tauri 配置必须设置 CSP 内容安全策略

### Vue/TypeScript 编码规范
- 组件命名：PascalCase（文件名和组件名一致）
- 变量命名：camelCase
- 使用组合式 API（Composition API）
- 类型定义放在 `src/types/` 目录
- 状态管理使用 Pinia store

<!-- ========== 关键文件：项目入口和核心模块位置 ========== -->
## 关键文件

<!-- Rust 后端关键文件列表 -->
### Rust 后端
| 文件 | 说明 |
|------|------|
| `src-tauri/src/lib.rs` | Tauri 命令注册和应用初始化 |
| `src-tauri/src/tray.rs` | 系统托盘创建（菜单、点击事件，由 lib.rs 提取） |
| `src-tauri/src/main.rs` | Tauri 应用入口点 |
| `src-tauri/src/logger.rs` | 日志系统配置（tracing） |
| `src-tauri/src/db/` | 数据库层 |
| `src-tauri/src/db/mod.rs` | 数据库模块入口和导出 |
| `src-tauri/src/db/connection.rs` | Database 结构体、连接创建、表初始化和迁移 |
| `src-tauri/src/db/schema.rs` | 数据库 Schema 定义 |
| `src-tauri/src/db/migration_aur.rs` | aur_info 表迁移 |
| `src-tauri/src/db/migration_software.rs` | software_info 表迁移 |
| `src-tauri/src/db/migration_upstream.rs` | upstream_info 表迁移 |
| `src-tauri/src/db/migration_enum.rs` | 枚举表迁移（licenses + languages） |
| `src-tauri/src/db/seed.rs` | 初始数据填充 |
| `src-tauri/src/db/software_info.rs` | 软件包信息表 |
| `src-tauri/src/db/aur_info.rs` | AUR 信息表；含 `get_aur_versions_map` 批量读取全部 AUR 版本（供上游批量检查消除 N+1 查询） |
| `src-tauri/src/db/upstream_info.rs` | 上游版本信息表 |
| `src-tauri/src/db/proxies_info.rs` | 代理配置表 |
| `src-tauri/src/db/backup_software.rs` | 备份软件表 |
| `src-tauri/src/db/cache_software.rs` | 缓存软件表 |
| `src-tauri/src/db/logs.rs` | 日志表 |
| `src-tauri/src/db/settings.rs` | 设置表（含缓存目录配置、启用/禁用） |
| `src-tauri/src/db/stats.rs` | 仪表盘统计聚合查询（各模块 COUNT 汇总） |
| `src-tauri/src/commands/` | Tauri IPC 命令（software/sys_command/enums 等） |
| `src-tauri/src/commands/dashboard.rs` | 仪表盘统计命令（get_dashboard_stats，聚合各模块计数） |
| `src-tauri/src/commands/fe_log.rs` | 前端诊断日志命令（frontend_log，仅 println! 到终端、不写文件） |
| `src-tauri/src/commands/upstream_validate.rs` | 上游 URL 验证命令 |
| `src-tauri/src/commands/backup/` | 备份管理命令模块（目录结构） |
| `src-tauri/src/commands/backup/mod.rs` | 模块声明和导出 |
| `src-tauri/src/commands/backup/backup_basic.rs` | 备份基础操作（查询、扫描、去重、删除） |
| `src-tauri/src/commands/backup/backup_install.rs` | 备份包安装和信息查询（pacman -Qip、sudoers、install） |
| `src-tauri/src/commands/backup/dedup.rs` | 去重逻辑（文件名解析、版本比较） |
| `src-tauri/src/commands/proxy/` | 代理管理命令模块（由单文件 commands/proxy.rs 拆分，保持 commands::proxy::* 注册路径） |
| `src-tauri/src/commands/proxy/mod.rs` | 模块声明和导出（basic/test 子模块） |
| `src-tauri/src/commands/proxy/basic.rs` | 代理基础命令（获取/下载/解析/增删改查） |
| `src-tauri/src/commands/proxy/test.rs` | 代理连通性测试命令与辅助 |
| `src-tauri/src/commands/fileops/cache_backup/` | 缓存包备份命令模块（由 fileops 下原单文件拆分） |
| `src-tauri/src/commands/fileops/cache_backup/mod.rs` | 模块声明和导出（existing/subdirectory 子模块） |
| `src-tauri/src/commands/fileops/cache_backup/existing.rs` | 备份到已有备份记录所在子目录（版本比较 + 复制） |
| `src-tauri/src/commands/fileops/cache_backup/subdirectory.rs` | 备份到指定子目录 |
| `src-tauri/src/commands/sysops/cache_cleanup.rs` | 缓存清理命令（系统缓存、自定义缓存目录、sudoers 配置） |
| `src-tauri/src/commands/sysops/cache_install.rs` | 缓存包安装和信息查询命令（pacman -Qip、sudoers 免密检测、install） |
| `src-tauri/src/commands/sysops/backup_install.rs` | 备份包安装和信息查询 Tauri 命令（pacman -Qip、sudoers、install） |
| `src-tauri/src/commands/sysops/backup_install_helpers.rs` | 备份安装的路径校验与 sudoers 规则辅助函数（被 cache_install/cache_cleanup 复用） |
| `src-tauri/src/commands/sysops/pacman_lock.rs` | pacman 数据库锁检查命令（检测 /var/lib/pacman/db.lck 是否存在） |
| `src-tauri/src/commands/sysops/software_sync/` | 软件包同步命令模块（目录结构） |
| `src-tauri/src/commands/sysops/software_sync/mod.rs` | 模块声明和导出（不含具体实现） |
| `src-tauri/src/commands/sysops/software_sync/aur.rs` | AUR 信息同步命令（只更新 aur_info 表，不更新 software_info 表） |
| `src-tauri/src/commands/sysops/software_sync/upstream.rs` | 上游版本批量检查命令（`check_all_upstream`）：映射任务交给 batch 引擎分类并行检查；Manual 包跳过网络仅回传标记；一次性批量读取 AUR 版本 + 内存映射语言 ID，消除写库阶段 N+1 查询与反复加锁 |
| `src-tauri/src/commands/sysops/software_sync/batch.rs` | 上游批量分类并发执行引擎：按检查器类型分桶（Manual 跳过 / Browser 严格限并发 / 网络类全局限并发）；单一 `JoinSet<UpstreamCheckResult>` 承载所有 `run_one` 任务（浏览器桶 + 必然 REST 桶 + 未命中 GraphQL 的回落桶），分两段 `join_next()` drain（先回收浏览器+必然 REST，待 GraphQL 完成后把未命中回落任务并入同一集合再 drain）；GitHub 批量查询与「必然 REST」桶并发启动，仅未命中者补回落 REST（避免重复请求与重复处理）；迁入 check_with_retry、PackageTask、BatchOutcome |
| `src-tauri/src/commands/sysops/software_sync/pkgbuild.rs` | PKGBUILD 文件同步命令（保留用户手动设置的字段） |
| `src-tauri/src/commands/sysops/software_sync/utils.rs` | 同步工具函数（AurParsedFields、parse_aur_fields 通用 AUR JSON 解析） |
| `src-tauri/src/checkers/` | 版本检查器模块 |
| `src-tauri/src/checkers/mod.rs` | 检查器模块入口和导出 |
| `src-tauri/src/checkers/factory.rs` | 检查器工厂函数（get_checker） |
| `src-tauri/src/checkers/trait_def.rs` | VersionChecker trait 定义 |
| `src-tauri/src/checkers/utils.rs` | 检查器工具函数（含版本正则提取） |
| `src-tauri/src/checkers/redirect.rs` | HTTP 重定向检查器（跟踪 Location / meta-refresh / JS 重定向） |
| `src-tauri/src/checkers/redirect_parse.rs` | 重定向检查器的 URL 解析与脚本扫描辅助函数 |
| `src-tauri/src/checkers/browser.rs` | 浏览器（JS 渲染）检查器（BrowserChecker），调用本机 Chromium/Chrome 渲染后提取版本；`spawn()` + `kill_on_drop(true)`，超时自动回收子进程，修复进程/内存泄漏；HTML 清洗正则用 `OnceLock` 懒加载，进程内仅编译一次 |
| `src-tauri/src/checkers/github/` | GitHub 检查器模块（目录结构） |
| `src-tauri/src/checkers/github/mod.rs` | 模块声明和导出（不含具体实现） |
| `src-tauri/src/checkers/github/tags_checker.rs` | GitHubTagsChecker 检查器实现 |
| `src-tauri/src/checkers/github/api_checker.rs` | GitHubAPIChecker 检查器实现 |
| `src-tauri/src/checkers/github/tags.rs` | GitHub Tags 分页获取和版本比较逻辑 |
| `src-tauri/src/checkers/github/release.rs` | GitHub latest release 路径版本提取（二进制检查 + 正则回退） |
| `src-tauri/src/checkers/github/release_history.rs` | GitHub Releases 历史遍历扫描（分页 + 资产过滤回退） |
| `src-tauri/src/checkers/github/git_describe.rs` | Git Describe 格式化（-git 包专用） |
| `src-tauri/src/checkers/github/graphql_batch.rs` | GitHub GraphQL 批量检查器：`batch_check_github` 用 alias 在单次请求批量查多仓库 tags/releases+license/languages；按 `owner/repo` 构建哈希索引一次性完成去重与按仓库匹配（O(n)，非 O(n²)）；分块用 `JoinSet` 并行发送请求；git 包/无 Token/仓库缺失回落逐包 REST；select_version 镜像 REST 路径 |
| `src-tauri/src/checkers/github/graphql_batch_parse.rs` | GitHub GraphQL 快照解析（RepoSnapshot / ReleaseData / parse_snapshot） |
| `src-tauri/src/versions/` | 版本处理模块（解析、标准化、比较） |
| `src-tauri/src/versions/mod.rs` | versions 模块入口 |
| `src-tauri/src/versions/utils.rs` | 版本处理工具函数（比较、排序、查找最新版本） |
| `src-tauri/src/versions/aur.rs` | AUR 版本解析和标准化 |
| `src-tauri/src/versions/upstream.rs` | 上游版本清洗和标准化 |
| `src-tauri/src/versions/comparison.rs` | 版本比较算法（vercmp） |
| `src-tauri/src/versions/rules.rs` | 版本清洗规则配置 |
| `src-tauri/src/aur/mod.rs` | AUR RPC API 交互 |
| `src-tauri/src/aur/rpc.rs` | AUR RPC 请求封装 |
| `src-tauri/src/aur/pkgbuild.rs` | PKGBUILD 文件解析 |
| `src-tauri/src/proxy/mod.rs` | 代理管理 |
| `src-tauri/src/proxy/fetch.rs` | 代理请求封装 |
| `src-tauri/src/proxy/test.rs` | 代理连通性测试 |
| `src-tauri/src/proxy/parse.rs` | 代理文件解析（公共 API，数组提取逻辑已拆出） |
| `src-tauri/src/proxy/parse/parse_array.rs` | 代理 JS 数组字节级状态机提取 |
| `src-tauri/src/proxy/parse/parse_tests.rs` | 代理解析单元测试 |
| `src-tauri/src/backup/mod.rs` | 备份管理 |
| `src-tauri/src/backup/execute.rs` | 备份执行逻辑 |
| `src-tauri/src/models/` | 数据模型定义 |
| `src-tauri/src/models/upstream_info.rs` | 上游版本信息模型（含 UpstreamUrlStatus 枚举） |
| `src-tauri/src/models/software_list_entry.rs` | 软件包列表展示模型 |
| `src-tauri/src/models/backup_software_entry.rs` | 备份软件包列表展示模型 |
| `src-tauri/src/models/cache_dir.rs` | 缓存目录配置模型 |
| `src-tauri/src/models/dashboard_stats.rs` | 仪表盘统计返回模型（DashboardStats） |

<!-- Vue 前端关键文件列表 -->
### Vue 前端
| 文件 | 说明 |
|------|------|
| `src/main.ts` | 应用入口，初始化 Vue 实例 |
| `src/App.vue` | 根组件，布局容器 |
| `src/router/index.ts` | Vue Router 路由配置 |
| `src/views/` | 页面组件（每个页面一个文件） |
| `src/components/base/` | 基础UI组件（Button、Card、Input、Select、Message、StatCard、Badge） |
| `src/components/common/` | 通用组件（StandardizedTable、StandardizedModal、PaginationControls、ProgressBar、PageToolbar等） |
| `src/components/layout/` | 布局组件（Sidebar、TabBar、BottomToolbar、PopupLayout、LogPanel等） |
| `src/components/package/` | 软件包相关组件（SoftwareDetailModal、SoftwareFormModal、PackageRowActions、PackageTable、各InfoCard等） |
| `src/components/backup/` | 备份管理组件（RowActions、InfoDialog、SudoersDialog等） |
| `src/components/cache/` | 缓存管理组件（CacheRowActions） |
| `src/components/dashboard/` | 仪表盘组件（ModuleCard 模块卡片） |
| `src/components/proxy/` | 代理管理组件（ProxyRowActions） |
| `src/components/settings/` | 设置页面组件（SettingsCard、SettingRow、各配置Section） |
| `src/components/filter/` | 筛选组件（FilterBar） |
| `src/components/enum/` | 枚举管理组件（LanguageFormModal、LicenseFormModal） |
| `src/utils/enums.ts` | 枚举常量和共享选项（packageTypes/checkerTypes/filterOptions） |
| `src/utils/format.ts` | 通用格式化工具（时间戳/License/JSON列表/枚举名称/语言名称） |
| `src/utils/felog.ts` | 前端诊断日志工具（feDebug/feInfo/feWarn/feError，经 IPC 转发到终端） |
| `src/composables/` | 组合式函数（hooks） |
| `src/composables/useCacheDirs.ts` | 缓存目录管理 composable（SettingsCacheSection 和 CacheManager 复用） |
| `src/composables/footer.ts` | 底部状态栏状态管理 |
| `src/composables/packageActions.ts` | 软件包操作逻辑（同步、检查、删除） |
| `src/composables/usePackageList.ts` | 软件包列表页逻辑（分页、搜索、选择） |
| `src/composables/useBackupList.ts` | 备份管理列表页逻辑（分页、搜索、选择） |
| `src/composables/useBackupInstall.ts` | 备份包安装逻辑（sudoers 检测、安装、包信息查询） |
| `src/composables/useCacheList.ts` | 缓存管理列表页逻辑（分页、搜索、选择） |
| `src/composables/useCacheBackupActions.ts` | 缓存备份操作逻辑（去重、备份新版本、备份到目录） |
| `src/composables/useCacheCleanup.ts` | 缓存清理操作逻辑（系统缓存、自定义缓存目录、sudoers 检测） |
| `src/composables/useCacheInfoNav.ts` | 缓存详情弹窗导航与选择逻辑（从 CacheManager 抽取） |
| `src/composables/useBackupInfoNav.ts` | 备份详情弹窗导航与选择逻辑（从 BackupManager 抽取） |
| `src/composables/useSoftwareForm.ts` | 软件包表单逻辑（验证、自动检测） |
| `src/composables/useLicenseSelect.ts` | License 可搜索下拉框逻辑 |
| `src/stores/` | Pinia 状态管理 |
| `src/types/index.ts` | TypeScript 类型定义 |
| `src/assets/styles/` | 样式文件目录（集中管理所有组件样式） |
| `src/assets/styles/base-components.css` | 基础组件样式索引（`@import` base/ 下 6 个子文件） |
| `src/assets/styles/layout-components.css` | 布局组件样式索引（`@import` layout/ 下 4 个子文件） |
| `src/assets/styles/modal-styles.css` | 模态框样式 |
| `src/assets/styles/table-styles.css` | 表格样式 |
| `src/assets/styles/settings-styles.css` | 设置页面样式 |
| `src/assets/styles/toolbar-buttons.css` | 工具栏按钮通用样式（toolbar-btn 及颜色变体） |
| `src/assets/styles/filter-styles.css` | 筛选面板样式（FilterBar） |

<!-- ========== 文件拆分记录：超限文件拆分历史 ========== -->
### 文件拆分记录

| 原文件 | 原行数 | 拆分后文件 | 新行数 | 拆分日期 | 状态 |
|--------|--------|-----------|--------|---------|------|
| `src/components/common/StandardizedTable.vue` | 718 | `StandardizedTable.vue` | 282 | 2026-07-29 | ✅ 完成 |
| | | `StandardizedTableHeader.vue` | 104 | | 新文件 |
| | | `StandardizedTableRow.vue` | 108 | | 新文件 |
| | | `StandardizedTablePagination.vue` | 150 | | 新文件 |
| | | `composables/useTableState.ts` | 273 | | 新文件 |
| | | `assets/styles/table-styles.css` | 287 | | 全局样式 |
| `src/views/PackageDetail.vue` | 643 | `PackageDetail.vue` | 277 | 2026-07-29 | ✅ 完成 |
| | | `PackageBasicInfoCard.vue` | 145 | | 新文件 |
| | | `PackageAurInfoCard.vue` | 101 | | 新文件 |
| | | `PackageUpstreamInfoCard.vue` | 113 | | 新文件 |
| | | `PackageDetailFooter.vue` | 175 | | 新文件 |
| `src/views/BackupManager.vue` | 436 | `BackupManager.vue` | 257 | 2026-07-29 | ✅ 完成 |
| | | `BackupRowActions.vue` | 54 | | 新文件 |
| | | `BackupInfoDialog.vue` | 58 | | 新文件 |
| | | `BackupSudoersDialog.vue` | 67 | | 新文件 |
| `src/views/CacheManager.vue` | 435 | `CacheManager.vue` | 233 | 2026-07-30 | ✅ 完成 |
| | | `CacheRowActions.vue` | 29 | | 新文件 |
| | | `composables/useCacheBackupActions.ts` | 124 | | 新文件 |
| `src/views/ProxySettings.vue` | 428 | `ProxySettings.vue` | 276 | 2026-07-29 | ✅ 完成 |
| | | `ProxyRowActions.vue` | 52 | | 新文件 |
| `src/components/BottomToolbar.vue` | 422 | `BottomToolbar.vue` | 160 | 2026-07-29 | ✅ 完成 |
| | | `LogPanel.vue` | 208 | | 新文件 |
| | | `PaginationControls.vue` | 92 | | 新文件 |
| | | `ProgressBar.vue` | 60 | | 新文件 |
| `src/components/SoftwareDetailModal.vue` | 417 | `SoftwareDetailModal.vue` | 283 | 2026-07-29 | ✅ 完成 |
| | | `SoftwareInfoTable.vue` | 58 | | 新文件 |
| | | `SoftwareStatusRow.vue` | 88 | | 新文件 |
| | | `SoftwareSideCards.vue` | 91 | | 新文件 |
| `src/components/common/StandardizedModal.vue` | 391 | `StandardizedModal.vue` | 189 | 2026-07-29 | ✅ 完成 |
| | | `assets/styles/modal-styles.css` | 205 | | 全局样式 |
| `src/views/PackageList.vue` | 340 | `PackageList.vue` | 297 | 2026-07-30 | ✅ 完成 |
| | | `PackageRowActions.vue` | 80 | | 新文件 |
| `src/components/settings/SettingsProxySection.vue` | 329 | `SettingsProxySection.vue` | 288 | 2026-07-30 | ✅ 完成（配置数组化） |
| `src/components/filter/FilterBar.vue` | 324 | `FilterBar.vue` | 186 | 2026-07-30 | ✅ 完成 |
| | | `assets/styles/filter-styles.css` | 148 | | 全局样式 |
| `src/components/package/DetailToolbar.vue` 等 | - | `assets/styles/toolbar-buttons.css` | 105 | 2026-07-30 | ✅ 完成（提取重复按钮样式） |
| `src/assets/styles/base-components.css` | 551 | `base-components.css`(索引) | 14 | 2026-08-10 | ✅ 完成 |
| | | `base/button.css` | 135 | | 新文件 |
| | | `base/card.css` | 68 | | 新文件 |
| | | `base/input.css` | 110 | | 新文件 |
| | | `base/message.css` | 60 | | 新文件 |
| | | `base/select.css` | 85 | | 新文件 |
| | | `base/stat-card.css` | 82 | | 新文件 |
| `src/assets/styles/layout-components.css` | 303 | `layout-components.css`(索引) | 12 | 2026-08-10 | ✅ 完成 |
| | | `layout/tab-bar.css` | 73 | | 新文件 |
| | | `layout/sidebar.css` | 92 | | 新文件 |
| | | `layout/bottom-toolbar.css` | 79 | | 新文件 |
| | | `layout/popup-layout.css` | 51 | | 新文件 |
| `src/views/ProxySettings.vue` | 534 | `ProxySettings.vue` | 256 | 2026-08-10 | ✅ 完成 |
| | | `composables/useProxyActions.ts` | 167 | | 新文件 |
| | | `components/proxy/ProxyEditModal.vue` | 72 | | 新文件 |
| | | `components/proxy/ProxyDetailModal.vue` | 78 | | 新文件 |
| | | `components/proxy/ProxyClearConfirmModal.vue` | 41 | | 新文件 |
| `src/views/CacheManager.vue` | 384 | `CacheManager.vue` | 248 | 2026-08-10 | ✅ 完成 |
| | | `components/cache/CacheSudoersModal.vue` | 167 | | 新文件 |
| `src/views/LogViewer.vue` | 341 | `LogViewer.vue` | 234 | 2026-08-10 | ✅ 完成 |
| | | `components/common/LogToolbar.vue` | 125 | | 新文件 |
| `src/views/PackageList.vue` | 334 | `PackageList.vue` | 240 | 2026-08-10 | ✅ 完成 |
| | | `components/package/PackageTable.vue` | 125 | | 新文件 |
| `src-tauri/src/commands/proxy.rs` | 403 | `commands/proxy/mod.rs` | 16 | 2026-08-10 | ✅ 完成 |
| | | `commands/proxy/basic.rs` | 143 | | 新文件 |
| | | `commands/proxy/test.rs` | 274 | | 新文件 |
| `src-tauri/src/proxy/parse.rs` | 375 | `proxy/parse.rs` | 173 | 2026-08-10 | ✅ 完成 |
| | | `proxy/parse/parse_array.rs` | 130 | | 新文件 |
| | | `proxy/parse/parse_tests.rs` | 86 | | 新文件 |
| `src-tauri/src/commands/fileops/cache_backup.rs` | 309 | `commands/fileops/cache_backup/mod.rs` | 12 | 2026-08-10 | ✅ 完成 |
| | | `commands/fileops/cache_backup/existing.rs` | 210 | | 新文件 |
| | | `commands/fileops/cache_backup/subdirectory.rs` | 110 | | 新文件 |
| `src-tauri/src/lib.rs` | 305 | `lib.rs` | 238 | 2026-08-10 | ✅ 完成 |
| | | `tray.rs` | 83 | | 新文件 |
| `src/views/Dashboard.vue` | 460 | `Dashboard.vue` | 260 | 2026-08-13 | ✅ 完成 |
| | | `components/dashboard/ModuleCard.vue` | 221 | | 新文件 |
| `src/views/CacheManager.vue` | 347 | `CacheManager.vue` | 301 | 2026-08-13 | ✅ 完成 |
| | | `composables/useCacheInfoNav.ts` | 83 | | 新文件 |
| `src/views/BackupManager.vue` | 319 | `BackupManager.vue` | 292 | 2026-08-13 | ✅ 完成 |
| | | `composables/useBackupInfoNav.ts` | 64 | | 新文件 |
| `src-tauri/src/checkers/github/release.rs` | 313 | `release.rs` | 140 | 2026-08-13 | ✅ 完成 |
| | | `release_history.rs` | 194 | | 新文件 |
| `src-tauri/src/checkers/redirect.rs` | 307 | `redirect.rs` | 183 | 2026-08-13 | ✅ 完成 |
| | | `redirect_parse.rs` | 138 | | 新文件（纯解析辅助函数） |
| `src-tauri/src/commands/sysops/backup_install.rs` | 323 | `backup_install.rs` | 224 | 2026-08-13 | ✅ 完成 |
| | | `backup_install_helpers.rs` | 123 | | 新文件（路径校验/sudoers 辅助） |

<!-- ========== 前端重构记录：目录重组与样式提取 ========== -->
### 前端重构记录（2026-07-29）

**重构目标**：组件目录模块化重组 + 样式文件集中管理

**重构内容**：
1. **目录结构重组**：将 `src/components/` 下 50+ 个文件按功能模块拆分为 10 个子目录
   - `base/` - 基础UI组件（7个文件）
   - `common/` - 通用组件（12个文件）
   - `layout/` - 布局组件（8个文件）
   - `package/` - 软件包组件（14个文件）
   - `backup/` - 备份组件（5个文件）
   - `cache/` - 缓存组件（2个文件）
   - `proxy/` - 代理组件（2个文件）
   - `settings/` - 设置组件（6个文件）
   - `filter/` - 筛选组件（1个文件）
   - `enum/` - 枚举组件（2个文件）

2. **冗余文件清理**：
   - 删除 `ConditionFilters.vue`（功能已整合到 FilterBar.vue）
   - 删除 `QuickFilters.vue`（功能已整合到 FilterBar.vue）
   - 删除 `assets/modal.css`（与 modal-styles.css 重复）
   - 创建 `LanguageFormModal.vue`（缺失但被引用的组件）

3. **样式文件提取与集中管理**：
   - 创建 `assets/styles/base-components.css` - 基础组件样式
   - 创建 `assets/styles/layout-components.css` - 布局组件样式
   - 已有 `modal-styles.css`、`table-styles.css`、`settings-styles.css`

4. **导入路径更新**：
   - 更新 `router/index.ts` 中的所有导入路径
   - 更新 `App.vue` 中的导入路径
   - 更新所有 `views/` 目录下的导入路径（10个文件）
   - 批量更新所有子目录组件的导入路径（50+ 文件）

**重构成果**：
- 删除冗余文件：3 个
- 创建新文件：3 个
- 移动文件到子目录：50+ 个
- 更新导入路径：60+ 个文件
- 创建子目录：10 个
- 提取CSS样式文件：2 个（新增）

**状态**：✅ 完成

<!-- ========== 前端重构记录：组件体系统一与代码质量修复 ========== -->
### 前端重构记录（2026-07-30）

**重构目标**：统一双组件体系 + 修复代码质量问题 + 消除超限文件

**重构内容**：
1. **组件体系统一**：
   - 删除 `common/Modal.vue`，两处使用者（SoftwareDetailModal、SoftwareFormModal）迁移到 `StandardizedModal`
   - `StandardizedModal` 支持任意像素宽度（非预设值通过内联样式应用）
   - 删除死代码 `DataTable.vue`、`DataTablePagination.vue`、`DataTableTypes.ts`（无任何使用者，统一使用 StandardizedTable）
   - 删除未使用的 `SoftwareAurCard.vue`、`SoftwareUpstreamCard.vue`

2. **超限文件拆分**：
   - `PackageList.vue`（340→297 行）：提取 `PackageRowActions.vue`
   - `CacheManager.vue`（305→233 行）：提取 `useCacheBackupActions.ts` composable
   - `FilterBar.vue`（324→186 行）：样式提取到 `filter-styles.css`
   - `SettingsProxySection.vue`（329→288 行）：代理 URL 字段配置数组化

3. **重复样式提取**：
   - 创建 `toolbar-buttons.css`：DetailToolbar 和 PackageDetailFooter 中约 90 行重复按钮样式合并
   - `SoftwareSideCards.vue` 改用 `utils/format.ts` 的公共格式化函数

4. **Bug 修复**：
   - Settings.vue 移除未定义的 `applySettings()` 调用
   - StandardizedBadge.vue 修复插槽渲染错误（`{{ $slots.default }}` → `<slot />`）
   - 清理 console.log/console.debug 调试语句

**状态**：✅ 完成（vue-tsc 与 pnpm build 全部通过）

<!-- ========== 开发命令：常用命令速查 ========== -->
## 开发命令
```bash
pnpm install       # 安装前端依赖
pnpm tauri dev     # 开发模式
pnpm tauri build   # 构建生产版本
cargo check        # Rust 类型检查
cargo clippy       # Rust lint 检查
cargo fmt          # Rust 代码格式化
cargo test         # Rust 单元测试
```

<!-- ========== 数据流：前端到后端的完整调用链路 ========== -->
## 数据流

### 完整调用链路
1. 前端 Vue 组件调用 `invoke("command_name", args)` 发起 IPC 请求
2. Tauri 路由到 `commands/` 模块中的对应处理函数
3. 命令函数调用业务逻辑层（checkers/aur/proxy/backup）
4. 业务逻辑层调用 `db/` 模块进行数据库操作
5. 数据库操作通过 rusqlite 执行 SQL 查询
6. 结果通过 serde 序列化为 JSON 返回前端
7. 前端 store 更新状态，组件响应式渲染

### 错误处理流程
- Rust 后端使用 `Result<T, Error>` 返回错误
- 错误信息自动序列化并传递给前端
- 前端统一捕获错误并显示用户友好的提示

<!-- ========== 检查器体系：版本检查器的类型和用途 ========== -->
## 检查器体系

所有检查器实现 `VersionChecker` trait（定义在 `checkers/trait_def.rs`）：

### 检查器类型
- `GitHubTagsChecker` — 通过 GitHub Tags API 获取所有 tags，支持版本提取关键字，适合需要获取大量 tags 的场景
- `GitHubAPIChecker` — 通过 GitHub Release API 获取最新版本，支持二进制文件检查和资产过滤
- `GiteeChecker` — Gitee API
- `GitLabChecker` — GitLab API
- `RedirectChecker` — HTTP 重定向（跟随 URL 获取版本）
- `HttpChecker` — HTML 页面解析（提取版本号）
- `BrowserChecker` — 浏览器（JS 渲染）检查器：适用于上游页面由 JavaScript 动态渲染、静态抓取只能拿到 SPA 空壳的场景（如百度 landingPage），调用本机 Chromium/Chrome 的 `--headless --dump-dom` 渲染后再用正则/HTML 提取版本（实现见 `checkers/browser.rs`）。批量场景下用 `spawn()` + `kill_on_drop(true)` 超时自动回收子进程，避免并发拉起多个 Chrome 导致内存/FD 耗尽及进程泄漏（修复见 AGENTS.md 安全审计「问题 11」）
- `ManualChecker` — 手动更新（用户指定版本）

### GitHub 检查器模块结构
GitHub 检查器采用目录结构（`checkers/github/`），包含以下文件：
- `mod.rs`: 模块声明和导出（不含具体实现）
- `tags_checker.rs`: GitHubTagsChecker 检查器实现，实现 `VersionChecker` trait
- `api_checker.rs`: GitHubAPIChecker 检查器实现，实现 `VersionChecker` trait
- `tags.rs`: Tags 分页获取和版本比较逻辑
- `release.rs`: Release API 调用（latest + 分页遍历）
- `binary_check.rs`: 二进制文件检查工具
- `repo_info.rs`: 仓库元信息获取（License + 编程语言）
- `git_describe.rs`: Git Describe 格式化（-git 包专用），通过 GitHub API 生成类似 `git describe` 的版本字符串
- `graphql_batch.rs`: GitHub GraphQL 批量检查器（`batch_check_github`）：用 alias 在单次请求里批量查多个仓库的 tags/releases + license/languages，按 `owner/repo` 去重；git 包/无 Token/仓库缺失回落逐包 REST；`select_version` 严格镜像 REST 路径保证结果一致
- `graphql_batch_parse.rs`: GitHub GraphQL 响应解析（`RepoSnapshot` / `ReleaseData` / `parse_snapshot`）

### 工具模块
- `checkers/utils.rs` — 通用工具函数（版本号正则提取、URL 解析等）

### CheckResult 结构体
所有检查器现在返回 `CheckResult` 结构体（定义在 `checkers/trait_def.rs`）：

```rust
pub struct CheckResult {
    pub version: Option<String>,  // 版本号
    pub license: Option<String>,  // License SPDX ID（如 "MIT", "Apache-2.0"）
}
```

**License 获取逻辑**：
- GitHub API 检查器（`GitHubAPIChecker`、`GitHubTagsChecker`）会自动获取仓库的 License 信息
- 其他检查器返回 `license: None`
- License 信息存储在 `upstream_info.upstream_license_id` 字段
- 如果 License 不存在于 `enum_licenses` 表，会自动创建新记录

### 版本提取正则表达式
每个检查器支持通过 `version_extract_regex` 参数自定义版本提取规则：
- 正则表达式可以包含捕获组，优先使用第一个捕获组的内容
- 如果正则匹配失败，检查器会回退到默认的版本提取逻辑
- 适用于版本号格式不标准的场景
- 当 `check_binary_files` 启用时，此参数用作资产文件名过滤器

### 调用方式
检查器通过 `checkers/mod.rs` 中的工厂函数创建，根据 `CheckerType` 枚举选择合适的检查器。

## 版本处理模块

`versions/` 模块专门处理各类版本号的解析、标准化和比较操作：

### 功能模块
- `aur.rs` — AUR 版本解析（epoch、version、pkgrel）
- `upstream.rs` — 上游版本清洗（移除前缀/后缀）
- `comparison.rs` — 版本比较算法（ALPM/pacman vercmp）
- `comparison/parser.rs` — 版本字符串解析器（epoch、组件拆分、组件比较）
- `comparison/tests.rs` — 版本比较单元测试
- `rules.rs` — 版本清洗规则配置

### 核心功能
1. **AUR 版本处理**：提取完整版本信息，比较时仅使用 version 部分
2. **上游版本处理**：清洗和标准化版本号，支持自定义规则
3. **版本比较**：基于 vercmp 算法，支持多种版本格式
   - **pkgrel 比较语义（重要）**：pkgrel（末尾 `-<纯数字>` 段）仅在 pkgver 完全相等时才参与比较；pkgver 不等时以 pkgver 为准，**禁止把 pkgrel 当作版本组件错位比较**。例如 `9.0.1-1` 必须判定为大于 `9.0-5`（pkgver `9.0.1` > `9.0`），而非因 pkgrel `1` < `5` 误判更旧。实现见 `comparison.rs` 的 `split_pkgrel` + `compare_vercmp`。
   - **缓存备份 / 上游检查共用**：`compare_vercmp` 是缓存备份「备份新版」与「上游 vs AUR」检查的唯一比较入口，pkgrel 修复同时修正两者。
4. **特殊字符处理**：将 `-` 转换为 `_` 符合 AUR 规范

<!-- ========== 数据库结构：核心数据表概述 ========== -->
## 数据库结构

### 核心数据表
| 表名 | 说明 |
|------|------|
| `software_info` | 软件包基本信息（名称、上游URL、检查器类型、版本提取正则等） |
| `aur_info` | AUR 仓库信息（版本、描述、依赖等） |
| `upstream_info` | 上游版本信息（版本、License ID、检查时间等） |
| `proxies_info` | 代理服务器配置（类型、地址、端口等） |
| `backup_software` | 备份记录（时间、路径、状态等） |
| `cache_software` | 缓存的软件包信息 |
| `logs` | 应用日志（级别、时间、内容） |
| `settings` | 应用设置项 |
| `enum_licenses` | License 枚举表（SPDX ID、全名） |

### AUR 批量查询设置
| 设置键 | 默认值 | 说明 |
|--------|--------|------|
| `aur_batch_size` | 50 | AUR 批量查询每批数量上限（最大100） |
| `aur_batch_interval` | 5 | AUR 批量查询间隔时间（秒） |

**AUR RPC API 限制**：
- URI 最大长度 4443 字节，Info 请求超过约 200 个包时需要分批处理
- 每天每个 IP 最多 4000 次请求
- 搜索结果超过 5000 个时会失败

### 列表每页行数设置
| 设置键 | 默认值 | 说明 |
|--------|--------|------|
| `list_page_size_software` | 50 | 软件管理页面每页行数 |
| `list_page_size_backup` | 50 | 备份管理页面每页行数 |
| `list_page_size_cache` | 50 | 缓存管理页面每页行数 |
| `list_page_size_proxy` | 50 | 代理管理页面每页行数 |
| `list_page_size_license` | 50 | License 管理页面每页行数 |
| `list_page_size_language` | 50 | 编程语言管理页面每页行数 |

**StandardizedTable 组件**：通用数据表格组件（`src/components/common/StandardizedTable.vue`），支持：
- 列配置（字段名、标题、宽度、格式化函数、排序）
- 前端分页（通过 props 传入每页行数）
- 搜索过滤
- 行选择（单选/全选）
- 自定义单元格插槽（`#cell-{key}`）和操作列插槽（`#actions`）

**⚠️ 使用注意事项（重要）**：
- 必须使用 `filteredEntries`（经过筛选的完整数据）而不是 `pageData`（分页后的数据），否则会导致数据不显示
- 必须添加动态 `:key` 确保响应性，如 `:key="\`table-${filteredEntries.length}\`"`
- 建议添加 `:showPagination="false"` 禁用内置分页，改用底部工具栏进行分页控制
- 必须从 composable 中解构 `filteredEntries`、`pageSize`、`currentPage` 等响应式变量

### CSS z-index 层级规范（重要）
- **工具栏（.page-toolbar）**：`z-index: 1001`（`position: relative`）
- **筛选遮罩层（.filter-overlay）**：`z-index: 1000`（`position: fixed`）
- **模态框遮罩层（.modal-overlay）**：`z-index: 1000`（`position: fixed`）
- 工具栏必须高于筛选遮罩层和模态框遮罩层，否则遮罩层打开时工具栏按钮会被拦截
- 修改筛选遮罩层或模态框遮罩层的 z-index 时，必须确保工具栏保持更高层级

### software_info 表字段说明
| 字段 | 类型 | 说明 |
|------|------|------|
| `software_id` | INTEGER | 主键 |
| `pkgname` | TEXT | 软件包名称（唯一） |
| `upstream_url` | TEXT | 上游仓库 URL |
| `checker_type_id` | INTEGER | 检查器类型（枚举） |
| `version_extract_regex` | TEXT | 版本提取正则表达式（可选） |
| `is_outdated` | INTEGER | 是否需要更新（0/1） |
| `check_test_versions` | INTEGER | 是否检查测试版本 |
| `check_binary_files` | INTEGER | 是否检查二进制文件 |
| `auto_check_enabled` | INTEGER | 是否启用自动检查 |

<!-- ========== Tauri 能力配置：权限和 IPC 规则 ========== -->
## Tauri 能力配置

### 权限文件
- `src-tauri/capabilities/default.json` — 默认权限配置

### IPC 通信规则
- 所有命令必须在 `lib.rs` 中注册
- 命令参数使用 `#[command]` 宏声明
- 敏感操作需要额外权限验证

### 前端窗口操作所需权限（重要）
单独打开/激活 Tauri 子窗口（`WebviewWindow`）时，前端调用的每个窗口方法都必须在
`capabilities/default.json` 中授予对应权限，否则命令会被 Tauri 拒绝并抛错：
- `core:window:allow-set-focus` — 调用 `setFocus()`
- `core:window:allow-show` — 调用 `show()` 重新显示已隐藏的窗口
- `core:window:allow-unminimize` — 调用 `unminimize()` 恢复最小化的窗口
- `core:webview:allow-create-webview-window` — 调用 `new WebviewWindow()` 创建新窗口
- 以上权限已包含在默认配置中。缺少其中任意一项都会导致窗口激活逻辑失效。

### Tauri v2 窗口事件（重要）
- **正确事件名称**：`"tauri://close-requested"`（不是 `"close"`）
- Tauri v2 的 `TauriEvent` 枚举定义了以下窗口事件：
  - `WINDOW_CLOSE_REQUESTED = "tauri://close-requested"` — 用户请求关闭窗口
  - `WINDOW_DESTROYED = "tauri://destroyed"` — 窗口已销毁
- **不要使用 `win.once("close", ...)`**，Tauri v2 没有 `"close"` 事件
- **窗口关闭行为**：当前 Rust 后端的 `on_window_event` 对所有窗口执行 `window.hide()` + `api.prevent_close()`（当 `close_action == "minimize_to_tray"` 时），这意味着弹出窗口关闭时只是被隐藏，而不是销毁
- **检测窗口是否存在**：使用 `WebviewWindow.getByLabel(label)` 获取窗口引用，然后调用 `show()` 恢复隐藏的窗口
- **重新激活（修复重复点击失效）**：在窗口已存在时，应先 `unminimize()`（若被最小化）再 `show()` + `setFocus()`；此流程依赖 `allow-show` / `allow-unminimize` / `allow-set-focus` 权限。历史上曾因缺少 `allow-show`，`show()` 抛错导致误入“重新创建”分支，而重复 label 的 `new WebviewWindow` 错误是异步事件（`tauri://error`），外部 `try/catch` 捕获不到，表现为第二次点击静默失效。补上权限并加固 `openWindow` 后修复。
- **参考实现**：`src/components/common/PageToolbar.vue` 的 `openWindow` 函数

<!-- ========== Git 提交规范：代码版本控制规则 ========== -->
## Git 提交规范

### 提交要求
- 每次完成任务后，必须主动将更改提交到 GitHub
- 提交前必须确认所有更改已完成且无错误
- 提交信息必须清晰、规范，使用以下格式

### 提交信息格式
```
<类型>: <简要描述>

<详细说明（可选）>
```

#### 类型说明
| 类型 | 说明 |
|------|------|
| `feat` | 新增功能 |
| `fix` | 修复 Bug |
| `docs` | 文档更新 |
| `style` | 代码格式调整 |
| `refactor` | 代码重构 |
| `test` | 测试相关 |
| `chore` | 构建/工具相关 |

#### 提交示例
```
docs: 完善 AGENTS.md 文档

- 扩展项目概述，添加核心功能模块说明
- 补充技术栈信息
- 新增代码规范章节
```

### 提交流程
1. 使用 `git add -A` 暂存所有更改
2. 使用 `git commit -m "<提交信息>"` 创建提交
3. 使用 `git push origin main` 推送到远程仓库

### 自动提交规则
- AI 编程助手在完成每个任务后，必须自动执行上述提交流程
- 如果提交失败（如网络问题、权限问题），应立即告知用户并提供错误信息
- 禁止在未经用户确认的情况下修改 `.gitignore` 文件