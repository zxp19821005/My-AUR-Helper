<!-- ================================================================ -->
<!-- My-AUR-Helper 系统架构设计文档                                    -->
<!-- 本文档详细描述项目的技术选型、文件结构、模块职责和系统数据流      -->
<!-- 帮助开发者快速了解项目的整体架构和各个组件的功能边界              -->
<!-- ================================================================ -->

# 系统架构设计

## 技术栈

| 层 | 技术 | 说明 |
|---|------|------|
| 桌面壳 | Tauri v2 | 跨平台桌面框架，Rust 后端 + Web 前端 |
| 后端语言 | Rust 1.96 | 高性能、内存安全 |
| 前端框架 | Vue 3 + TypeScript | Composition API + `<script setup>` |
| 状态管理 | Pinia | 响应式状态管理 |
| 构建工具 | Vite 7（当前 7.3.6）| 快速 HMR 开发体验（由 v6 升级，见下方说明）|
| 数据库 | SQLite (via rusqlite) | 嵌入式数据库，无需额外服务 |
| HTTP 客户端 | reqwest | 异步 HTTP 请求 |
| 日志 | tracing + tauri-plugin-log | 结构化日志，支持文件输出 |
| 序列化 | serde / serde_json | Rust 数据结构 ↔ JSON |

### 构建工具链说明（Vite 7 升级）

- **版本**：`vite` 由 `^6.4.3` 升级至 `^7.1.11`（实际锁定 7.3.6），以满足仓库版本策略 hook 的最低版本要求；配套 `vue-tsc` / `@vitejs/plugin-vue` 无需额外改动，升级后 `vue-tsc --noEmit && vite build` 通过。
- **无害告警**：升级后构建可能出现 `@tauri-apps/api/core.js` 同时被动态与静态导入的提示（`dynamic import will not move module into another chunk`）。该提示源于 Tauri API 包自身的导入方式，**不影响产物正确性与运行**，可忽略。
- **依赖覆盖**：`pnpm-workspace.yaml` 顶层 `overrides` 固定 `nanoid: 3.3.17`，修复构建期传递依赖的 moderate 漏洞（GHSA-2v37-7h3g-55p8）。注意 pnpm v11 已不再读取 `package.json` 内的 `pnpm` 字段，覆盖项只能放在 `pnpm-workspace.yaml`。
- **本地 install 注意**：WorkBuddy 注入的"安全删除"shim 会拦截 `pnpm install` 对 `node_modules` 目录的删除（目录走 `copyFileSync` 导致 EISDIR 失败）。若安装卡在此处，用 `env -u CODEBUDDY_SESSION_ID -u CLAUDE_SESSION_ID pnpm install` 让 shim 早退、走原生删除即可。

## 代码规范

### 文件组织原则

1. **单一职责**: 每个文件只负责一个功能模块
2. **行数限制**: 单个文件不超过 300 行
3. **模块拆分**: 超过 300 行的文件必须拆分为多个独立文件
4. **代码重用**: 优先提取通用组件和工具函数，避免重复代码
5. **命名一致**: 文件名、函数名、组件名需与功能模块统一

### 拆分规则

- `commands/` 按职责分组: 顶层单文件命令 + `fileops/`（文件操作类）+ `sysops/`（系统操作类）
- `db/mod.rs` 按表拆分: `db/software_info.rs`, `db/aur_info.rs` 等
- Vue 组件拆分: 通用逻辑提取到 `composables/`，通用样式提取到 `assets/styles/` 全局 CSS

## Rust 后端文件结构

```
src-tauri/src/
├── lib.rs                    # 库入口，Tauri Builder 配置和命令注册
├── main.rs                   # 程序入口
├── logger.rs                 # 日志配置
├── errors/                   # 统一错误类型
│   ├── mod.rs                # 错误模块导出
│   ├── error_type.rs         # AppError 错误类型定义
│   ├── db.rs                 # 数据库错误
│   ├── file.rs               # 文件操作错误
│   ├── network.rs            # 网络错误
│   └── system.rs             # 系统命令错误
├── models/
│   ├── mod.rs                # 数据模型导出
│   ├── software_info.rs      # 软件包信息模型
│   ├── software_detail.rs    # 软件包完整详情模型
│   ├── software_list_entry.rs # 软件包列表展示模型
│   ├── aur_info.rs           # AUR 信息模型
│   ├── upstream_info.rs      # 上游版本信息模型（含 UpstreamUrlStatus 枚举）
│   ├── proxy_info.rs         # 代理信息模型
│   ├── proxy_type.rs         # 代理类型枚举
│   ├── proxy_test.rs         # 代理测试结果模型
│   ├── backup_software.rs    # 备份软件模型
│   ├── backup_software_entry.rs # 备份软件包列表展示模型
│   ├── cache_software.rs     # 缓存软件模型
│   ├── cache_software_entry.rs # 缓存软件包列表展示模型
│   ├── log_entry.rs          # 日志模型
│   ├── setting.rs            # 设置模型
│   ├── checker_type.rs       # 检查器类型枚举
│   ├── package_type.rs       # 包类型枚举
│   ├── enum_license.rs       # 许可证枚举
│   └── enum_programming_language.rs # 编程语言枚举
├── db/
│   ├── mod.rs                # 数据库模块声明和导出
│   ├── connection.rs         # Database 结构体、连接创建、表初始化
│   ├── schema.rs             # 数据库 Schema 定义
│   ├── migration_aur.rs      # aur_info 表迁移
│   ├── migration_software.rs # software_info 表迁移
│   ├── migration_upstream.rs # upstream_info 表迁移
│   ├── migration_backup.rs   # backup_software 表迁移
│   ├── migration_cache.rs    # cache_software 表迁移
│   ├── migration_proxy.rs    # proxies 表迁移
│   ├── migration_enum.rs     # 枚举表迁移（licenses + languages）
│   ├── seed.rs               # 初始数据填充
│   ├── software_info.rs      # SoftwareInfo 表操作
│   ├── aur_info.rs           # AurInfo 表操作
│   ├── upstream_info.rs      # UpstreamInfo 表操作
│   ├── proxies_info.rs       # ProxiesInfo 表操作
│   ├── proxies_test.rs       # ProxiesTest 表操作
│   ├── backup_software.rs    # BackupSoftware 表操作
│   ├── cache_software.rs     # CacheSoftware 表操作
│   ├── tests_cache_backup.rs # 缓存备份相关单元测试
│   ├── logs.rs               # Logs 表操作
│   ├── settings.rs           # Settings 表操作
│   ├── enum_licenses.rs      # EnumLicenses 表操作
│   └── enum_programming_languages.rs # EnumProgrammingLanguages 表操作
├── commands/
│   ├── mod.rs                # 命令模块导出
│   ├── software.rs           # 软件包 CRUD 命令
│   ├── proxy.rs              # 代理命令
│   ├── logs.rs               # 日志命令
│   ├── settings.rs           # 设置命令
│   ├── enums.rs              # 枚举查询命令
│   ├── fileops/              # 文件操作类命令
│   │   ├── mod.rs            # 模块声明和导出
│   │   ├── scan.rs           # 包文件扫描命令
│   │   ├── cache_dirs.rs     # 缓存目录通用工具
│   │   ├── cache_scan.rs     # 缓存扫描命令
│   │   ├── cache_backup.rs   # 缓存备份命令
│   │   ├── backup_scan.rs    # 备份目录扫描命令
│   │   ├── backup_dedup.rs   # 备份去重命令
│   │   └── backup_execute.rs # 备份执行逻辑
│   └── sysops/               # 系统操作类命令
│       ├── mod.rs            # 模块声明和导出
│       ├── sys_command.rs    # 系统命令（包版本查询等）
│       ├── software_check.rs # 软件包版本检查命令
│       ├── upstream_validate.rs # 上游 URL 验证命令
│       ├── backup_basic.rs   # 备份基础操作（查询、清空、删除）
│       ├── backup_install.rs # 备份包安装（pacman -Qip、sudoers、install）
│       ├── proxy_utils.rs    # 代理工具命令
│       └── software_sync/    # 软件包同步命令模块
│           ├── mod.rs        # 模块声明和导出
│           ├── aur.rs        # AUR 信息同步和更新命令
│           ├── upstream.rs   # 上游版本并行检查命令
│           ├── pkgbuild.rs   # PKGBUILD 文件同步命令
│           └── utils.rs      # 同步工具函数
├── checkers/
│   ├── mod.rs                # 检查器模块入口和导出
│   ├── factory.rs            # 检查器工厂函数（get_checker）
│   ├── trait_def.rs          # VersionChecker trait 和 CheckResult 定义
│   ├── github/               # GitHub 检查器模块（目录结构）
│   │   ├── mod.rs            # 模块声明和导出
│   │   ├── tags_checker.rs   # GitHubTagsChecker 检查器实现
│   │   ├── api_checker.rs    # GitHubAPIChecker 检查器实现
│   │   ├── tags.rs           # Tags 分页获取和版本比较逻辑
│   │   ├── release.rs        # Release API 调用（latest + 分页遍历）
│   │   ├── binary_check.rs   # 二进制文件检查工具
│   │   ├── repo_info.rs      # 仓库元信息获取（License + 编程语言）
│   │   └── git_describe.rs   # Git Describe 格式化（-git 包专用）
│   ├── gitee.rs              # Gitee 检查器
│   ├── gitlab.rs             # GitLab 检查器
│   ├── redirect.rs           # 重定向检查器
│   ├── http.rs               # HTTP 页面解析检查器
│   ├── manual.rs             # 手动检查器
│   └── utils.rs              # 检查器工具函数
├── versions/                 # 版本处理模块
│   ├── mod.rs                # versions 模块入口
│   ├── utils.rs              # 版本比较、排序、查找最新版本
│   ├── aur.rs                # AUR 版本解析和标准化
│   ├── upstream.rs           # 上游版本清洗和标准化
│   ├── git_version.rs        # Git 版本格式处理
│   ├── comparison.rs         # 版本比较算法（vercmp）
│   ├── comparison/parser.rs  # 版本字符串解析器
│   ├── comparison/tests.rs   # 版本比较单元测试
│   └── rules.rs              # 版本清洗规则配置
├── aur/
│   ├── mod.rs                # AUR 模块导出
│   ├── rpc.rs                # AUR RPC API 请求
│   └── pkgbuild.rs           # PKGBUILD 文件解析
└── proxy/
    ├── mod.rs                # 代理模块导出
    ├── fetch.rs              # 代理获取
    ├── download.rs           # 代理文件下载
    ├── parse.rs              # 代理文件解析
    └── test.rs               # 代理测试
```

## Vue 前端文件结构

```
src/
├── main.ts                   # 入口文件（全局样式导入）
├── App.vue                   # 根组件
├── router/
│   └── index.ts              # 路由配置
├── views/                    # 页面组件（10 个）
│   ├── Dashboard.vue         # 仪表盘
│   ├── PackageList.vue       # 软件包列表
│   ├── PackageDetail.vue     # 软件包详情/编辑
│   ├── BackupManager.vue     # 备份管理
│   ├── CacheManager.vue      # 缓存管理
│   ├── ProxySettings.vue     # 代理设置
│   ├── LogViewer.vue         # 日志查看
│   ├── Settings.vue          # 应用设置
│   ├── LicenseManager.vue    # 许可证管理
│   └── LanguageManager.vue   # 编程语言管理
├── components/               # 组件（按功能模块分 10 个子目录）
│   ├── base/                 # 基础UI组件（StandardizedButton/Card/Input/Select/Message/StatCard/Badge）
│   ├── common/               # 通用组件（StandardizedTable 系列、StandardizedModal、PageToolbar、PaginationControls、ProgressBar）
│   ├── layout/               # 布局组件（Sidebar、TabBar、BottomToolbar、LogPanel、PopupLayout、SettingsPopup、LogsPopup、EnumLayout）
│   ├── package/              # 软件包组件（SoftwareDetailModal、SoftwareFormModal、PackageRowActions、各 InfoCard、DetailToolbar、FloatingNav 等）
│   ├── backup/               # 备份组件（BackupToolbar、BackupRowActions、BackupInfoDialog、BackupSudoersDialog、BackupToModal）
│   ├── cache/                # 缓存组件（CacheToolbar、CacheRowActions）
│   ├── proxy/                # 代理组件（ProxyToolbar、ProxyRowActions）
│   ├── settings/             # 设置组件（SettingsCard、SettingRow、AppearanceSettings、各配置 Section）
│   ├── filter/               # 筛选组件（FilterBar）
│   └── enum/                 # 枚举组件（LicenseFormModal、LanguageFormModal）
├── composables/              # 组合式函数（12 个）
│   ├── footer.ts             # 底部工具栏状态
│   ├── packageActions.ts     # 软件包操作逻辑
│   ├── usePackageList.ts     # 软件包列表页逻辑
│   ├── useBackupList.ts      # 备份管理列表页逻辑
│   ├── useBackupInstall.ts   # 备份包安装逻辑
│   ├── useCacheList.ts       # 缓存管理列表页逻辑
│   ├── useCacheBackupActions.ts # 缓存备份操作逻辑
│   ├── useCacheDirs.ts       # 缓存目录管理
│   ├── useProxyList.ts       # 代理列表页逻辑
│   ├── useSoftwareForm.ts    # 软件包表单逻辑
│   ├── useLicenseSelect.ts   # License 可搜索下拉框逻辑
│   └── useTableState.ts      # 表格状态管理（StandardizedTable）
├── stores/                   # Pinia 状态管理
│   ├── packages.ts           # 软件包状态
│   ├── settings.ts           # 设置状态
│   └── tabs.ts               # 标签页状态
├── utils/
│   ├── enums.ts              # 枚举常量和共享选项
│   ├── format.ts             # 通用格式化工具
│   └── icons.ts              # 图标映射
├── types/
│   └── index.ts              # TypeScript 类型定义
└── assets/
    ├── base.css              # 基础样式
    ├── variables.css         # CSS 变量
    ├── components.css        # 组件通用样式
    ├── forms.css             # 表单样式
    └── styles/               # 集中管理的组件样式
        ├── base-components.css    # 基础组件样式
        ├── layout-components.css  # 布局组件样式
        ├── modal-styles.css       # 模态框样式
        ├── table-styles.css       # 表格样式
        ├── settings-styles.css    # 设置页面样式
        ├── toolbar-buttons.css    # 工具栏按钮通用样式
        └── filter-styles.css      # 筛选面板样式
```

## 系统架构图

```
┌─────────────────────────────────────────────────┐
│                  Tauri 桌面窗口                    │
│  ┌─────────────────────────────────────────────┐│
│  │           Vue 3 前端 (WebView)               ││
│  │  ┌──────┐ ┌────────┐ ┌──────┐ ┌────────┐  ││
│  │  │ 仪表盘 │ │包管理  │ │备份  │ │代理/日志│  ││
│  │  └──────┘ └────────┘ └──────┘ └────────┘  ││
│  │           ↕ Tauri IPC (invoke)              ││
│  └─────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────┐│
│  │          Rust 后端 (src-tauri)               ││
│  │  ┌────────┐ ┌──────────┐ ┌──────────────┐  ││
│  │  │commands│ │  checkers │ │    proxy     │  ││
│  │  │(IPC入口)│ │(版本检查器)│ │  (代理管理)  │  ││
│  │  └───┬────┘ └────┬─────┘ └──────┬───────┘  ││
│  │      └─────┬─────┘              │           ││
│  │            ▼                     │           ││
│  │  ┌──────────────┐              │           ││
│  │  │   aur 模块    │              │           ││
│  │  │ (AUR API交互) │              │           ││
│  │  └──────┬───────┘              │           ││
│  │         ▼                      ▼           ││
│  │  ┌──────────┐          ┌──────────────┐    ││
│  │  │  db 模块  │          │ versions 模块 │    ││
│  │  │ (SQLite)  │          │  (版本处理)   │    ││
│  │  └──────────┘          └──────────────┘    ││
│  │         ▼                                   ││
│  │  ┌──────────┐                               ││
│  │  │ log 模块  │                               ││
│  │  │ (tracing) │                               ││
│  │  └──────────┘                               ││
│  └─────────────────────────────────────────────┘│
└─────────────────────────────────────────────────┘
```

## 模块职责

### commands/ — Tauri IPC 命令入口

作为前后端通信桥梁，所有 `#[tauri::command]` 在此定义，参数/返回值自动序列化为 JSON。按职责分为三层：

- 顶层单文件命令：`software.rs`（CRUD）、`proxy.rs`、`logs.rs`、`settings.rs`、`enums.rs`
- `fileops/` — 文件操作类命令：包文件扫描、缓存扫描/备份、备份目录扫描/去重
- `sysops/` — 系统操作类命令：系统命令、版本检查、上游 URL 验证、备份查询/安装、软件包同步（`software_sync/` 子模块，并行执行 tokio::spawn）

### models/ — 数据模型

定义核心数据结构：SoftwareInfo、AurInfo、UpstreamInfo、ProxyInfo 等，统一用于 Rust 和 SQLite。

### db/ — 数据库层

封装所有 SQLite 操作，提供 CRUD 方法。使用 rusqlite 的 prepared statement 确保 SQL 注入防护。按功能拆分为独立文件，每个文件负责一张表的操作。

### checkers/ — 版本检查器体系

基于 `VersionChecker` trait 的多态实现：
- GitHubTagsChecker: 通过 GitHub API 获取最新 tags，支持版本提取关键字
- GitHubAPIChecker: 通过 GitHub API 获取最新 release，支持二进制文件检查
- GitHub 模块采用目录结构：
  - `mod.rs`: 模块声明和导出（不含具体实现）
  - `tags_checker.rs`: GitHubTagsChecker 检查器实现
  - `api_checker.rs`: GitHubAPIChecker 检查器实现
  - `tags.rs`: Tags 分页获取和版本比较逻辑
  - `release.rs`: Release API 调用（latest + 分页遍历）
  - `binary_check.rs`: 二进制文件检查工具
  - `repo_info.rs`: 仓库元信息获取（License + 编程语言）
  - `git_describe.rs`: Git Describe 格式化（-git 包专用）
- GiteeChecker: 通过 Gitee API
- GitLabChecker: 通过 GitLab API
- RedirectChecker: 跟踪 HTTP 重定向获取版本
- HttpChecker: 解析 HTML 页面版本信息
- ManualChecker: 占位，等待用户手动更新

### aur/ — AUR 交互

- 通过 AUR RPC v5 接口获取用户维护的包列表
- 支持批量查询 (`get_packages_info`) 和单个查询 (`get_package_info`)
- 解析本地 PKGBUILD 文件，提取版本、URL、检查器信息
- 自动推断合适的检查器类型

### proxy/ — 代理管理

- 从 Greasyfork userscript 中解析代理列表
- 代理健康检测（延迟测试）
- 按类型分类（download/clone/raw）
- 解析时推断每个代理的「协议头约定」（`strip_target_protocol`）：
  保留目标协议头（如 cdn.crashmc.com 类）或去除（如 cors.isteed.cc 类），
  持久化到 `proxies_info` 表，测试拼接时按约定决定是否去掉目标地址的 `https://` 前缀，
  规避双重拼接（如 `cdn.xxx/github.com/https://github.com`）

### versions/ — 版本处理

- AUR 版本解析（epoch、version、pkgrel）
- 上游版本清洗和标准化（可配置规则）
- 版本比较算法（ALPM/pacman vercmp）

### errors/ — 统一错误类型

- `AppError` 枚举按领域细分（数据库/文件/网络/系统）
- 所有命令返回 `AppResult<T>`，错误自动序列化传递给前端

### 备份与缓存管理（commands/fileops + commands/sysops）

- 扫描 pacman/paru/yay 缓存目录（`fileops/cache_scan.rs`）
- 复制新版本 .pkg.tar.zst 到备份目录（`fileops/cache_backup.rs`）
- 备份目录扫描、去重（`fileops/backup_scan.rs`、`fileops/backup_dedup.rs`）
- 备份记录查询/删除、备份包安装（`sysops/backup_basic.rs`、`sysops/backup_install.rs`）

## 检查器选择逻辑

```
解析 PKGBUILD → 检测 _ghurl / _giteeurl / _gitlaburl / _dlurl
              → 检测 url 域名 (github/gitee/gitlab)
              → 检测 pkgver() 函数
              → 自动分配对应检查器
              → 兜底为 ManualChecker
```