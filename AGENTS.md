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
| `src-tauri/src/main.rs` | Tauri 应用入口点 |
| `src-tauri/src/logger.rs` | 日志系统配置（tracing） |
| `src-tauri/src/db/` | 数据库层 |
| `src-tauri/src/db/mod.rs` | 数据库模块入口和初始化 |
| `src-tauri/src/db/schema.rs` | 数据库 Schema 定义 |
| `src-tauri/src/db/migration_aur.rs` | aur_info 表迁移 |
| `src-tauri/src/db/migration_software.rs` | software_info 表迁移 |
| `src-tauri/src/db/migration_upstream.rs` | upstream_info 表迁移 |
| `src-tauri/src/db/migration_enum.rs` | 枚举表迁移（licenses + languages） |
| `src-tauri/src/db/seed.rs` | 初始数据填充 |
| `src-tauri/src/db/software_info.rs` | 软件包信息表 |
| `src-tauri/src/db/aur_info.rs` | AUR 信息表 |
| `src-tauri/src/db/upstream_info.rs` | 上游版本信息表 |
| `src-tauri/src/db/proxies_info.rs` | 代理配置表 |
| `src-tauri/src/db/backup_software.rs` | 备份软件表 |
| `src-tauri/src/db/cache_software.rs` | 缓存软件表 |
| `src-tauri/src/db/logs.rs` | 日志表 |
| `src-tauri/src/db/settings.rs` | 设置表 |
| `src-tauri/src/commands/` | Tauri IPC 命令（software/sys_command/enums 等） |
| `src-tauri/src/commands/upstream_validate.rs` | 上游 URL 验证命令 |
| `src-tauri/src/commands/software_sync/` | 软件包同步命令模块（目录结构） |
| `src-tauri/src/commands/software_sync/mod.rs` | 模块声明和导出（不含具体实现） |
| `src-tauri/src/commands/software_sync/aur.rs` | AUR 信息同步和更新命令 |
| `src-tauri/src/commands/software_sync/upstream.rs` | 上游版本并行检查命令 |
| `src-tauri/src/commands/software_sync/pkgbuild.rs` | PKGBUILD 文件同步命令 |
| `src-tauri/src/commands/software_sync/utils.rs` | 同步工具函数和类型定义 |
| `src-tauri/src/checkers/` | 版本检查器模块 |
| `src-tauri/src/checkers/mod.rs` | 检查器工厂函数 |
| `src-tauri/src/checkers/trait_def.rs` | VersionChecker trait 定义 |
| `src-tauri/src/checkers/utils.rs` | 检查器工具函数（含版本正则提取） |
| `src-tauri/src/checkers/github/` | GitHub 检查器模块（目录结构） |
| `src-tauri/src/checkers/github/mod.rs` | 模块声明和导出（不含具体实现） |
| `src-tauri/src/checkers/github/tags_checker.rs` | GitHubTagsChecker 检查器实现 |
| `src-tauri/src/checkers/github/api_checker.rs` | GitHubAPIChecker 检查器实现 |
| `src-tauri/src/checkers/github/tags.rs` | GitHub Tags 分页获取和版本比较逻辑 |
| `src-tauri/src/checkers/github/api.rs` | GitHub Release API 调用和资产过滤逻辑 |
| `src-tauri/src/checkers/github/git_describe.rs` | Git Describe 格式化（-git 包专用） |
| `src-tauri/src/versions/` | 版本处理模块（解析、标准化、比较） |
| `src-tauri/src/versions/mod.rs` | versions 模块入口 |
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
| `src-tauri/src/backup/mod.rs` | 备份管理 |
| `src-tauri/src/backup/execute.rs` | 备份执行逻辑 |
| `src-tauri/src/models/` | 数据模型定义 |
| `src-tauri/src/models/upstream_info.rs` | 上游版本信息模型（含 UpstreamUrlStatus 枚举） |
| `src-tauri/src/models/software_list_entry.rs` | 软件包列表展示模型 |

<!-- Vue 前端关键文件列表 -->
### Vue 前端
| 文件 | 说明 |
|------|------|
| `src/main.ts` | 应用入口，初始化 Vue 实例 |
| `src/App.vue` | 根组件，布局容器 |
| `src/router/index.ts` | Vue Router 路由配置 |
| `src/views/` | 页面组件（每个页面一个文件） |
| `src/components/DataTable.vue` | 通用数据表格组件（支持分页、搜索、选择） |
| `src/components/FilterBar.vue` | 筛选器组件（快速筛选 + 条件筛选） |
| `src/components/` | 通用组件（跨页面复用） |
| `src/composables/` | 组合式函数（hooks） |
| `src/composables/footer.ts` | 底部状态栏状态管理 |
| `src/composables/packageActions.ts` | 软件包操作逻辑（同步、检查、删除） |
| `src/composables/usePackageList.ts` | 软件包列表页逻辑（分页、搜索、选择） |
| `src/composables/useSoftwareForm.ts` | 软件包表单逻辑（验证、自动检测） |
| `src/composables/useLicenseSelect.ts` | License 可搜索下拉框逻辑 |
| `src/stores/` | Pinia 状态管理 |
| `src/types/index.ts` | TypeScript 类型定义 |
| `src/assets/styles.css` | 全局样式（TailwindCSS） |

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

**DataTable 组件**：通用数据表格组件（`src/components/DataTable.vue`），支持：
- 列配置（字段名、标题、宽度、格式化函数）
- 前端分页（通过 props 传入每页行数）
- 搜索过滤
- 行选择（单选/全选）
- 自定义单元格插槽（`#cell-{key}`）

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