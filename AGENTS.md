<!-- ================================================================ -->
<!-- AI 助手配置指南 / AGENTS.md                                       -->
<!-- 本文件为 AI 编码助手（如 opencode）提供项目上下文信息，             -->
<!-- 包括技术栈、代码规范、关键文件位置和开发命令等。                  -->
<!-- AI 在生成代码时应参考本文件以确保符合项目约定。                  -->
<!-- ================================================================ -->

# AI 助手配置指南

<!-- ========== 项目概述 ========== -->
## 项目概述
My-AUR-Helper 是一个 Tauri 桌面应用，用于管理 AUR 软件包更新、本地备份和代理。

<!-- ========== 技术栈：列出项目核心技术 ========== -->
## 技术栈
- 后端: Rust + Tauri v2
- 前端: Vue 3 + TypeScript + Vite
- 数据库: SQLite (rusqlite)
- 日志: tracing

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

<!-- ========== 关键文件：项目入口和核心模块位置 ========== -->
## 关键文件

<!-- Rust 后端关键文件列表 -->
### Rust 后端
| 文件 | 说明 |
|------|------|
| `src-tauri/src/lib.rs` | Tauri 应用入口，注册所有命令 |
| `src-tauri/src/db/` | 数据库层，按表拆分为独立文件 |
| `src-tauri/src/commands/` | Tauri IPC 命令（software/files/sys_command/enums 等） |
| `src-tauri/src/checkers/` | 版本检查器 |
| `src-tauri/src/aur/mod.rs` | AUR RPC API 交互 |
| `src-tauri/src/proxy/mod.rs` | 代理管理 |
| `src-tauri/src/backup/mod.rs` | 备份管理 |
| `src-tauri/src/models/` | 数据模型 |

<!-- Vue 前端关键文件列表 -->
### Vue 前端
| 文件 | 说明 |
|------|------|
| `src/views/` | 页面组件（每个页面一个文件） |
| `src/components/` | 通用组件（跨页面复用） |
| `src/stores/` | Pinia 状态管理 |
| `src/types/index.ts` | TypeScript 类型定义 |

<!-- ========== 开发命令：常用命令速查 ========== -->
## 开发命令
```bash
pnpm tauri dev     # 开发模式
pnpm tauri build   # 构建
cargo check        # Rust 类型检查
cargo clippy       # Rust lint
```

<!-- ========== 数据流：前端到后端的完整调用链路 ========== -->
## 数据流
1. 前端调用 `invoke("command_name", args)` 
2. Rust 后端 `commands/` 模块处理请求
3. 数据库操作在 `db/` 模块中执行
4. 结果序列化为 JSON 返回前端

<!-- ========== 检查器体系：版本检查器的类型和用途 ========== -->
## 检查器体系
所有检查器实现 `VersionChecker` trait：
- `GitHubChecker` — GitHub API (release/tag)
- `GiteeChecker` — Gitee API  
- `GitLabChecker` — GitLab API
- `RedirectChecker` — HTTP 重定向
- `HttpChecker` — HTML 页面解析
- `ManualChecker` — 手动更新
