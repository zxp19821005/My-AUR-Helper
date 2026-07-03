<!-- ================================================================ -->
<!-- My-AUR-Helper 系统架构设计文档                                    -->
<!-- 本文档详细描述项目的技术选型、文件结构、模块职责和系统数据流      -->
<!-- 帮助开发者快速了解项目的整体架构和各个组件的功能边界              -->
<!-- ================================================================ -->

# 系统架构设计

<!-- ========== 技术栈：列出项目使用的所有关键技术及其用途 ========== -->
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

<!-- ========== 代码规范：文件组织原则 ========== -->
## 代码规范

<!-- 文件组织原则：确保项目结构清晰、模块边界明确 -->
### 文件组织原则

1. **单一职责**: 每个文件只负责一个功能模块
2. **行数限制**: 单个文件不超过 300 行
3. **模块拆分**: 超过 300 行的文件必须拆分为多个独立文件
4. **代码重用**: 优先提取通用组件和工具函数，避免重复代码
5. **命名一致**: 文件名、函数名、组件名需与功能模块统一

<!-- Rust 后端文件结构：按功能模块划分的目录树 -->
### Rust 后端文件结构

```
src-tauri/src/
├── lib.rs                    # 库入口，Tauri Builder 配置
├── main.rs                   # 程序入口
├── models/
│   └── mod.rs                # 数据模型（Package、Proxy、Backup、Log 等）
├── db/
│   ├── mod.rs                # 数据库初始化和连接管理
│   ├── packages.rs           # Package CRUD 操作
│   ├── proxies.rs            # Proxy CRUD 操作
│   ├── backup_configs.rs     # BackupConfig CRUD 操作
│   ├── logs.rs               # Log CRUD 操作
│   ├── settings.rs           # Settings 操作
│   ├── checker_configs.rs    # CheckerConfig CRUD 操作
│   ├── licenses.rs           # License CRUD 操作
│   └── languages.rs          # Language CRUD 操作
├── commands/
│   ├── mod.rs                # 命令模块导出
│   ├── packages.rs           # 包管理命令
│   ├── backup.rs             # 备份命令
│   ├── proxy.rs              # 代理命令
│   ├── files.rs              # 文件操作命令
│   ├── sys_command.rs        # 系统命令（pacman/makepkg）
│   ├── logs.rs               # 日志命令
│   ├── settings.rs           # 设置命令
│   └── enums.rs              # 枚举查询命令
├── checkers/
│   └── mod.rs                # 版本检查器（所有实现）
├── aur/
│   └── mod.rs                # AUR RPC 交互和 PKGBUILD 解析
├── proxy/
│   └── mod.rs                # 代理获取和管理
├── backup/
│   └── mod.rs                # 备份扫描和执行
└── log/
    └── mod.rs                # tracing 日志初始化
```

<!-- Vue 前端文件结构：前端页面和组件的组织方式 -->
### Vue 前端文件结构

```
src/
├── main.ts                   # 入口文件
├── App.vue                   # 根组件
├── router/
│   └── index.ts              # 路由配置
├── views/                    # 页面组件（每个页面一个文件）
│   ├── Dashboard.vue
│   ├── PackageList.vue
│   ├── PackageDetail.vue
│   ├── BackupManager.vue
│   ├── ProxySettings.vue
│   ├── LogViewer.vue
│   ├── Settings.vue
│   ├── LicenseManager.vue
│   └── LanguageManager.vue
├── components/               # 通用组件（跨页面复用）
│   ├── ui/                   # 基础 UI 组件
│   ├── forms/                # 表单组件
│   └── layout/               # 布局组件
├── stores/
│   └── packages.ts           # Pinia 状态管理
├── types/
│   └── index.ts              # TypeScript 类型定义
└── assets/                   # 静态资源
```

<!-- ========== 系统架构图：展示前后端交互和各模块调用关系 ========== -->
## 系统架构图

<!--
  架构分层说明：
  - 顶层：Tauri 桌面窗口，提供原生桌面体验
  - 中间层（前端）：Vue 3 页面通过 IPC 与后端通信
  - 底层（后端）：Rust 模块分层处理业务逻辑
-->
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

<!-- ========== 模块职责：详细说明每个后端模块的功能边界 ========== -->
## 模块职责

<!-- commands/ — 作为前后端通信的桥梁，接收前端 IPC 调用并分发到各业务模块 -->
### commands/ — Tauri IPC 命令入口
作为前后端通信桥梁，所有 `#[tauri::command]` 在此定义，参数/返回值自动序列化为 JSON。

<!-- models/ — 定义所有数据模型，用于 Rust 内部和数据库之间的数据交换 -->
### models/ — 数据模型
定义核心数据结构：Package、ProxySource、BackupConfig、LogEntry 等，统一用于 Rust 和 SQLite。

<!-- db/ — 数据库访问层，封装所有 SQLite 操作，每个文件负责一张表 -->
### db/ — 数据库层
封装所有 SQLite 操作，提供 CRUD 方法。使用 rusqlite 的 prepared statement 确保 SQL 注入防护。
按功能拆分为独立文件，每个文件负责一张表的操作。

<!-- checkers/ — 版本检查器体系，基于 trait 的多态实现，支持多种版本获取方式 -->
### checkers/ — 版本检查器体系
基于 `VersionChecker` trait 的多态实现：
- GitHubChecker: 通过 GitHub API 获取最新 release/tag
- GiteeChecker: 通过 Gitee API
- GitLabChecker: 通过 GitLab API
- RedirectChecker: 跟踪 HTTP 重定向获取版本
- HttpChecker: 解析 HTML 页面版本信息
- ManualChecker: 占位，等待用户手动更新

<!-- aur/ — AUR 交互模块，负责从 AUR 获取包信息并解析 PKGBUILD -->
### aur/ — AUR 交互
- 通过 AUR RPC v12 接口获取用户维护的包列表
- 解析本地 PKGBUILD 文件，提取版本、URL、检查器信息
- 自动推断合适的检查器类型

<!-- proxy/ — 代理管理模块，从 Greasyfork 获取代理列表并检测可用性 -->
### proxy/ — 代理管理
- 从 Greasyfork userscript 中解析代理列表
- 代理健康检测（延迟测试）
- 按类型分类（download/clone/raw）

<!-- backup/ — 备份管理模块，扫描 pacman 缓存并备份已安装的包文件 -->
### backup/ — 备份管理
- 扫描 pacman/paru/yay 缓存目录
- 复制新版本 .pkg.tar.zst 到备份目录
- 清理备份目录中的旧版本

<!-- log/ — 日志模块，基于 tracing 框架实现结构化日志输出 -->
### log/ — 日志模块
基于 tracing 框架，支持控制台和文件日志输出。

<!-- ========== 检查器选择逻辑：描述如何根据 PKGBUILD 内容自动选择合适的版本检查器 ========== -->
## 检查器选择逻辑

```
解析 PKGBUILD → 检测 _ghurl / _giteeurl / _gitlaburl / _dlurl
              → 检测 url 域名 (github/gitee/gitlab)
              → 检测 pkgver() 函数
              → 自动分配对应检查器
              → 兜底为 ManualChecker
```
