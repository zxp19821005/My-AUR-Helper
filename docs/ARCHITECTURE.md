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
| 构建工具 | Vite 5 | 快速 HMR 开发体验 |
| 数据库 | SQLite (via rusqlite) | 嵌入式数据库，无需额外服务 |
| HTTP 客户端 | reqwest | 异步 HTTP 请求 |
| 日志 | tracing + tauri-plugin-log | 结构化日志，支持文件输出 |
| 序列化 | serde / serde_json | Rust 数据结构 ↔ JSON |

## 代码规范

### 文件组织原则

1. **单一职责**: 每个文件只负责一个功能模块
2. **行数限制**: 单个文件不超过 300 行
3. **模块拆分**: 超过 300 行的文件必须拆分为多个独立文件
4. **代码重用**: 优先提取通用组件和工具函数，避免重复代码
5. **命名一致**: 文件名、函数名、组件名需与功能模块统一

### 拆分规则

- `commands/software.rs` 按功能拆分: `software.rs`, `software_sync.rs`, `software_check.rs`
- `commands/files.rs` 按功能拆分: `files.rs`, `files_scan.rs`
- `db/mod.rs` 按表拆分: `db/software_info.rs`, `db/aur_info.rs` 等
- Vue 组件拆分: 通用逻辑提取到 `composables/`，通用样式提取到全局 CSS

## Rust 后端文件结构

```
src-tauri/src/
├── lib.rs                    # 库入口，Tauri Builder 配置 (262行)
├── main.rs                   # 程序入口
├── logger.rs                 # 日志配置
├── models/
│   ├── mod.rs                # 数据模型导出
│   ├── software_info.rs      # 软件包信息模型
│   ├── software_list_entry.rs # 软件包列表展示模型
│   ├── aur_info.rs           # AUR 信息模型
│   ├── upstream_info.rs      # 上游版本信息模型
│   ├── proxy_info.rs         # 代理信息模型
│   ├── proxy_type.rs         # 代理类型枚举
│   ├── proxy_test.rs         # 代理测试结果模型
│   ├── backup_software.rs    # 备份软件模型
│   ├── cache_software.rs     # 缓存软件模型
│   ├── log_entry.rs          # 日志模型
│   ├── setting.rs            # 设置模型
│   ├── checker_type.rs       # 检查器类型枚举
│   ├── package_type.rs       # 包类型枚举
│   ├── enum_license.rs       # 许可证枚举
│   └── enum_programming_language.rs # 编程语言枚举
├── db/
│   ├── mod.rs                # 数据库初始化和连接管理 (61行)
│   ├── schema.rs             # 数据库 Schema 定义 (155行)
│   ├── migration.rs          # 数据库迁移脚本
│   ├── seed.rs               # 初始数据填充
│   ├── software_info.rs      # SoftwareInfo 表操作 (186行)
│   ├── aur_info.rs           # AurInfo 表操作
│   ├── upstream_info.rs      # UpstreamInfo 表操作
│   ├── proxies_info.rs       # ProxiesInfo 表操作
│   ├── proxies_test.rs       # ProxiesTest 表操作
│   ├── backup_software.rs    # BackupSoftware 表操作
│   ├── cache_software.rs     # CacheSoftware 表操作
│   ├── logs.rs               # Logs 表操作
│   ├── settings.rs           # Settings 表操作
│   ├── enum_licenses.rs      # EnumLicenses 表操作
│   └── enum_programming_languages.rs # EnumProgrammingLanguages 表操作
├── commands/
│   ├── mod.rs                # 命令模块导出 (42行)
│   ├── software.rs           # 软件包 CRUD 命令 (163行)
│   ├── software_sync.rs      # 软件包同步命令 (246行)
│   ├── software_check.rs     # 软件包版本检查命令 (144行)
│   ├── files.rs              # 文件操作命令 (221行)
│   ├── files_scan.rs         # 包文件扫描命令 (85行)
│   ├── backup.rs             # 备份命令
│   ├── proxy.rs              # 代理命令 (83行)
│   ├── sys_command.rs        # 系统命令 (183行)
│   ├── scan.rs               # 目录扫描命令 (193行)
│   ├── logs.rs               # 日志命令
│   ├── settings.rs           # 设置命令
│   └── enums.rs              # 枚举查询命令 (144行)
├── checkers/
│   ├── mod.rs                # 检查器工厂函数 (33行)
│   ├── trait_def.rs          # VersionChecker trait 定义
│   ├── github.rs             # GitHub 检查器 (112行)
│   ├── gitee.rs              # Gitee 检查器
│   ├── gitlab.rs             # GitLab 检查器
│   ├── redirect.rs           # 重定向检查器
│   ├── http.rs               # HTTP 页面解析检查器
│   ├── manual.rs             # 手动检查器
│   └── utils.rs              # 检查器工具函数
├── aur/
│   ├── mod.rs                # AUR 模块导出
│   ├── rpc.rs                # AUR RPC API 请求 (83行)
│   └── pkgbuild.rs           # PKGBUILD 文件解析 (186行)
├── proxy/
│   ├── mod.rs                # 代理模块导出
│   ├── fetch.rs              # 代理获取 (146行)
│   └── test.rs               # 代理测试
└── backup/
    ├── mod.rs                # 备份模块导出
    └── execute.rs            # 备份执行逻辑 (123行)
```

## Vue 前端文件结构

```
src/
├── main.ts                   # 入口文件 (31行)
├── App.vue                   # 根组件 (87行)
├── router/
│   └── index.ts              # 路由配置 (103行)
├── views/                    # 页面组件
│   ├── Dashboard.vue         # 仪表盘 (103行)
│   ├── PackageList.vue       # 软件包列表 (290行)
│   ├── PackageDetail.vue     # 软件包详情/编辑 (320行) ⚠️
│   ├── BackupManager.vue     # 备份管理 (85行)
│   ├── CacheManager.vue      # 缓存管理 (86行)
│   ├── ProxySettings.vue     # 代理设置 (148行)
│   ├── LogViewer.vue         # 日志查看 (122行)
│   ├── Settings.vue          # 应用设置 (266行)
│   ├── LicenseManager.vue    # 许可证管理 (208行)
│   └── LanguageManager.vue   # 编程语言管理 (241行)
├── components/               # 通用组件
│   ├── PageToolbar.vue       # 页面工具栏 (92行)
│   ├── Sidebar.vue           # 侧边栏 (198行)
│   ├── BottomToolbar.vue     # 底部工具栏 (167行)
│   ├── TabBar.vue            # 标签栏 (158行)
│   ├── PopupLayout.vue       # 弹窗布局 (153行)
│   ├── SoftwareFormModal.vue # 软件包添加/编辑弹窗 (353行) ⚠️
│   ├── SoftwareDetailModal.vue # 软件包详情弹窗 (233行)
│   ├── SettingsPopup.vue     # 设置弹窗 (34行)
│   ├── LogsPopup.vue         # 日志弹窗 (24行)
│   └── EnumLayout.vue        # 枚举管理布局 (26行)
├── composables/              # 组合式函数
│   ├── footer.ts             # 底部工具栏状态 (23行)
│   └── packageActions.ts     # 软件包操作逻辑 (189行)
├── stores/                   # Pinia 状态管理
│   ├── packages.ts           # 软件包状态 (57行)
│   └── tabs.ts               # 标签页状态 (77行)
├── types/
│   └── index.ts              # TypeScript 类型定义 (207行)
└── assets/
    └── styles.css            # 全局样式
```

> ⚠️ 标记的文件超过 300 行限制，需要进一步拆分

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
│  │  │  db 模块  │          │  backup 模块  │    ││
│  │  │ (SQLite)  │          │ (备份管理)    │    ││
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

作为前后端通信桥梁，所有 `#[tauri::command]` 在此定义，参数/返回值自动序列化为 JSON。

- `software.rs` — 软件包 CRUD 操作
- `software_sync.rs` — AUR 同步和 PKGBUILD 解析
- `software_check.rs` — 上游版本检查
- `files.rs` — 文件/目录操作
- `files_scan.rs` — 包文件扫描和解析

### models/ — 数据模型

定义核心数据结构：SoftwareInfo、AurInfo、UpstreamInfo、ProxyInfo 等，统一用于 Rust 和 SQLite。

### db/ — 数据库层

封装所有 SQLite 操作，提供 CRUD 方法。使用 rusqlite 的 prepared statement 确保 SQL 注入防护。按功能拆分为独立文件，每个文件负责一张表的操作。

### checkers/ — 版本检查器体系

基于 `VersionChecker` trait 的多态实现：
- GitHubChecker: 通过 GitHub API 获取最新 release/tag
- GiteeChecker: 通过 Gitee API
- GitLabChecker: 通过 GitLab API
- RedirectChecker: 跟踪 HTTP 重定向获取版本
- HttpChecker: 解析 HTML 页面版本信息
- ManualChecker: 占位，等待用户手动更新

### aur/ — AUR 交互

- 通过 AUR RPC v12 接口获取用户维护的包列表
- 解析本地 PKGBUILD 文件，提取版本、URL、检查器信息
- 自动推断合适的检查器类型

### proxy/ — 代理管理

- 从 Greasyfork userscript 中解析代理列表
- 代理健康检测（延迟测试）
- 按类型分类（download/clone/raw）

### backup/ — 备份管理

- 扫描 pacman/paru/yay 缓存目录
- 复制新版本 .pkg.tar.zst 到备份目录
- 清理备份目录中的旧版本

## 检查器选择逻辑

```
解析 PKGBUILD → 检测 _ghurl / _giteeurl / _gitlaburl / _dlurl
              → 检测 url 域名 (github/gitee/gitlab)
              → 检测 pkgver() 函数
              → 自动分配对应检查器
              → 兜底为 ManualChecker
```
