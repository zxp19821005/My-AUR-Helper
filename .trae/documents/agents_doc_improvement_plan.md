# AGENTS.md 文档完善计划

## 一、现状分析

通过对比当前 AGENTS.md 文档内容与实际项目结构，发现以下信息缺失和描述不清晰的问题：

### 1.1 项目概述过于简略
- 当前仅一句话描述，缺乏核心功能模块的详细说明
- 未提及项目的目标用户和使用场景

### 1.2 技术栈信息不完整
- **缺失内容**：
  - 前端 CSS 框架（从文件结构推测使用了 TailwindCSS）
  - 状态管理库（Pinia）
  - 路由库（Vue Router）
  - UI 图标库
  - 数据库迁移工具
  - HTTP 客户端库（reqwest）
  - 序列化库（serde）

### 1.3 代码规范不够具体
- **缺失内容**：
  - Rust 编码规范（命名、格式化工具）
  - Vue/TypeScript 编码规范
  - Git 提交规范
  - 错误处理规范
  - 异步代码风格

### 1.4 关键文件列表严重不全

#### Rust 后端缺失文件：
| 实际文件 | 文档缺失 | 说明 |
|---------|---------|------|
| `src-tauri/src/logger.rs` | ✅ | 日志配置 |
| `src-tauri/src/main.rs` | ✅ | Tauri 应用入口 |
| `src-tauri/src/aur/pkgbuild.rs` | ✅ | PKGBUILD 解析 |
| `src-tauri/src/aur/rpc.rs` | ✅ | AUR RPC 接口 |
| `src-tauri/src/checkers/trait_def.rs` | ✅ | VersionChecker trait 定义 |
| `src-tauri/src/checkers/utils.rs` | ✅ | 检查器工具函数 |
| `src-tauri/src/proxy/fetch.rs` | ✅ | 代理请求封装 |
| `src-tauri/src/proxy/test.rs` | ✅ | 代理测试 |
| `src-tauri/src/backup/execute.rs` | ✅ | 备份执行逻辑 |
| `src-tauri/src/db/schema.rs` | ✅ | 数据库 Schema 定义 |
| `src-tauri/src/db/migration.rs` | ✅ | 数据库迁移 |
| `src-tauri/src/db/seed.rs` | ✅ | 初始数据 |

#### Vue 前端缺失文件：
| 实际文件 | 文档缺失 | 说明 |
|---------|---------|------|
| `src/composables/` | ✅ | 组合式函数（hooks） |
| `src/router/index.ts` | ✅ | 路由配置 |
| `src/App.vue` | ✅ | 根组件 |
| `src/main.ts` | ✅ | 应用入口 |
| `src/assets/styles.css` | ✅ | 全局样式 |

### 1.5 开发命令不完整
- **缺失命令**：
  - 依赖安装命令（`pnpm install`）
  - 单元测试命令
  - 构建预览命令
  - 代码格式化命令

### 1.6 数据流描述过于笼统
- 未说明具体的数据模型和类型转换
- 未提及错误处理流程
- 未说明异步操作的处理方式

### 1.7 检查器体系描述不完整
- 缺失 `trait_def.rs` 的说明
- 缺失 `utils.rs` 的说明
- 未说明检查器的调用方式和生命周期

### 1.8 缺少重要文档引用
- 项目根目录下已有 `docs/` 文件夹包含架构文档和数据库文档，但未在 AGENTS.md 中引用

---

## 二、完善方案

### 2.1 新增模块

| 模块名称 | 内容说明 |
|---------|---------|
| **项目架构** | 项目分层架构说明，前后端职责划分 |
| **数据库结构** | 主要数据表及其关系概述 |
| **路由配置** | 前端页面路由结构 |
| **Tauri 能力** | 权限配置和 IPC 通信规则 |
| **调试技巧** | 常用调试命令和工具 |

### 2.2 扩展现有模块

#### 2.2.1 项目概述
- 补充核心功能模块说明
- 添加项目架构图描述

#### 2.2.2 技术栈
- 添加前端框架细节（Pinia、Vue Router）
- 添加 Rust 关键依赖（reqwest、serde、rusqlite、tracing）

#### 2.2.3 代码规范
- 添加 Rust 编码规范（cargo fmt、clippy 规则）
- 添加 Vue/TypeScript 编码规范（命名、组件结构）

#### 2.2.4 关键文件
- 补充所有缺失文件的说明
- 按功能模块重新组织文件列表

#### 2.2.5 开发命令
- 添加依赖安装、测试、格式化等命令

#### 2.2.6 数据流
- 补充完整的数据流转图描述
- 添加错误处理流程说明

---

## 三、具体修改内容

### 3.1 项目概述（扩展）
```markdown
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
```

### 3.2 技术栈（补充）
```markdown
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
```

### 3.3 代码规范（新增）
```markdown
### Rust 编码规范
- 使用 `cargo fmt` 进行代码格式化
- 使用 `cargo clippy` 进行 lint 检查
- 函数命名：snake_case
- 结构体/枚举命名：PascalCase
- 使用 `Result<T, Error>` 进行错误处理
- 异步代码使用 `async/await`

### Vue/TypeScript 编码规范
- 组件命名：PascalCase（文件名和组件名一致）
- 变量命名：camelCase
- 使用组合式 API（Composition API）
- 类型定义放在 `src/types/` 目录
- 状态管理使用 Pinia store
```

### 3.4 关键文件（补充）

#### Rust 后端完整列表：
| 文件 | 说明 |
|------|------|
| `src-tauri/src/lib.rs` | Tauri 命令注册和应用初始化 |
| `src-tauri/src/main.rs` | Tauri 应用入口点 |
| `src-tauri/src/logger.rs` | 日志系统配置（tracing） |
| `src-tauri/src/db/` | 数据库层 |
| `src-tauri/src/db/schema.rs` | 数据库 Schema 定义 |
| `src-tauri/src/db/migration.rs` | 数据库迁移脚本 |
| `src-tauri/src/db/seed.rs` | 初始数据填充 |
| `src-tauri/src/db/software_info.rs` | 软件包信息表 |
| `src-tauri/src/db/aur_info.rs` | AUR 信息表 |
| `src-tauri/src/db/upstream_info.rs` | 上游版本信息表 |
| `src-tauri/src/db/proxies_info.rs` | 代理配置表 |
| `src-tauri/src/db/backup_software.rs` | 备份软件表 |
| `src-tauri/src/db/cache_software.rs` | 缓存软件表 |
| `src-tauri/src/db/logs.rs` | 日志表 |
| `src-tauri/src/db/settings.rs` | 设置表 |
| `src-tauri/src/commands/` | Tauri IPC 命令 |
| `src-tauri/src/checkers/` | 版本检查器 |
| `src-tauri/src/checkers/trait_def.rs` | VersionChecker trait 定义 |
| `src-tauri/src/checkers/utils.rs` | 检查器工具函数 |
| `src-tauri/src/aur/mod.rs` | AUR RPC API 交互 |
| `src-tauri/src/aur/rpc.rs` | AUR RPC 请求封装 |
| `src-tauri/src/aur/pkgbuild.rs` | PKGBUILD 文件解析 |
| `src-tauri/src/proxy/mod.rs` | 代理管理 |
| `src-tauri/src/proxy/fetch.rs` | 代理请求封装 |
| `src-tauri/src/proxy/test.rs` | 代理连通性测试 |
| `src-tauri/src/backup/mod.rs` | 备份管理 |
| `src-tauri/src/backup/execute.rs` | 备份执行逻辑 |
| `src-tauri/src/models/` | 数据模型定义 |

#### Vue 前端完整列表：
| 文件 | 说明 |
|------|------|
| `src/main.ts` | 应用入口，初始化 Vue 实例 |
| `src/App.vue` | 根组件，布局容器 |
| `src/router/index.ts` | Vue Router 路由配置 |
| `src/views/` | 页面组件 |
| `src/components/` | 通用组件 |
| `src/composables/` | 组合式函数（hooks） |
| `src/stores/` | Pinia 状态管理 |
| `src/types/index.ts` | TypeScript 类型定义 |
| `src/assets/styles.css` | 全局样式（TailwindCSS） |

### 3.5 开发命令（补充）
```markdown
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
```

### 3.6 数据流（扩展）
```markdown
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
```

### 3.7 检查器体系（扩展）
```markdown
## 检查器体系

所有检查器实现 `VersionChecker` trait（定义在 `checkers/trait_def.rs`）：

### 检查器类型
- `GitHubChecker` — GitHub API (release/tag)
- `GiteeChecker` — Gitee API
- `GitLabChecker` — GitLab API
- `RedirectChecker` — HTTP 重定向（跟随 URL 获取版本）
- `HttpChecker` — HTML 页面解析（提取版本号）
- `ManualChecker` — 手动更新（用户指定版本）

### 工具模块
- `checkers/utils.rs` — 通用工具函数（版本号比较、HTTP 请求封装等）

### 调用方式
检查器通过 `checkers/mod.rs` 中的工厂函数创建，根据 `CheckerType` 枚举选择合适的检查器。
```

### 3.8 新增模块示例

#### 数据库结构
```markdown
## 数据库结构

### 核心数据表
| 表名 | 说明 |
|------|------|
| `software_info` | 软件包基本信息（名称、版本、描述等） |
| `aur_info` | AUR 仓库信息（URL、维护者等） |
| `upstream_info` | 上游版本信息（来源类型、URL、最新版本等） |
| `proxies_info` | 代理服务器配置（类型、地址、端口等） |
| `backup_software` | 备份记录（时间、路径、状态等） |
| `cache_software` | 缓存的软件包信息 |
| `logs` | 应用日志（级别、时间、内容） |
| `settings` | 应用设置项 |
```

#### Tauri 能力配置
```markdown
## Tauri 能力配置

### 权限文件
- `src-tauri/capabilities/default.json` — 默认权限配置

### IPC 通信规则
- 所有命令必须在 `lib.rs` 中注册
- 命令参数使用 `#[command]` 宏声明
- 敏感操作需要额外权限验证
```

---

## 四、实施步骤

| 步骤 | 内容 | 预期产出 |
|------|------|---------|
| 1 | 扩展项目概述和技术栈 | 更新项目背景和技术栈信息 |
| 2 | 新增代码规范章节 | 添加 Rust 和 Vue/TS 编码规范 |
| 3 | 补充关键文件列表 | 完善前后端文件清单 |
| 4 | 扩展开发命令 | 添加缺失的开发命令 |
| 5 | 扩展数据流描述 | 添加完整调用链路和错误处理 |
| 6 | 扩展检查器体系 | 添加 trait 定义和工具模块说明 |
| 7 | 新增数据库结构章节 | 添加数据表概述 |
| 8 | 新增 Tauri 能力章节 | 添加权限配置说明 |
| 9 | 验证文档完整性 | 对比实际项目结构确认无遗漏 |

---

## 五、风险评估

| 风险 | 等级 | 应对措施 |
|------|------|---------|
| 文档与代码不同步 | 中 | 建议在 PR 流程中增加文档更新检查 |
| 新增内容过多导致文档冗长 | 低 | 保持每个章节简洁，避免重复 |
| 技术栈版本变更 | 低 | 在文档中标注版本号，定期更新 |
