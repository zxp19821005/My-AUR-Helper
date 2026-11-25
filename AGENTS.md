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
2. **行数硬上限 300 行（强制 / 可机械校验）**
   - 统计口径：`wc -l`，含空行与注释，统计范围为 `src/` 与 `src-tauri/src/` 下的 `.rs` / `.vue` / `.ts` 源文件；不含 `target/`、`node_modules/`、`dist/`、`.cargo/`、`.rustup/`
   - 校验命令（提交前必跑，输出必须为空）：
     ```bash
     find src src-tauri/src -type f \( -name '*.rs' -o -name '*.vue' -o -name '*.ts' \) \
       -print0 | xargs -0 wc -l | sort -rn | awk '$1>300'
     ```
   - **例外清单（仅以下三类）**：
     a. **生成物**：`src/bindings.ts`（tauri-specta 自动生成，禁止手改，**也不得为标注豁免而改文件**；豁免由本规范直接登记；改后端签名后重新生成，见「IPC 铁律」）
     b. **纯数据/枚举表**：仅含常量字典、无任何控制流（`if` / `match` / `for` / `while`）且函数体 ≤ 3 行的文件 → 必须在文件头标注 `// @LINE_LIMIT_EXEMPT: 纯数据表`
     c. **测试文件**：独立 `tests.rs` / `*.test.ts` / `#[cfg(test)]` 专用文件，用例线性堆叠不强制拆分；但单个测试函数 ≤ 80 行 → 必须在文件头标注 `// @LINE_LIMIT_EXEMPT: 测试文件`
   - 不在例外清单内的超限文件**必须拆分**；不得以「历史原因」「暂时」为由豁免。临界预警：>270 行的文件禁止再新增逻辑，必须先拆分或迁移到公共模块
3. **模块拆分**: 超过 300 行的文件必须拆分为多个独立文件
4. **代码重用（强制）**: 优先提取通用组件和工具函数；详见下方「禁止重复实现」
5. **命名一致**: 文件名、函数名、组件名需与功能模块统一；详见下方「统一命名规范表」

<!-- 拆分规则：具体模块的拆分方法 -->
### 拆分规则
- **Rust**：`xxx.rs` 超限时建同名目录 `xxx/` + `mod.rs`（`mod.rs` 仅声明与 `pub use`，不含实现），按职责拆为 `xxx/cache.rs`、`xxx/parse.rs`、`xxx/download.rs`、`xxx/tests.rs`；对外保持原有 `pub use` 路径兼容，调用方不改 import
- **Vue 组件**：先抽 `<style>` 到同名 `Xxx.css`（本项目 `PackageList.vue` → `PackageList.css` 已采用）；再抽 template 中独立区块到 `src/components/<domain>/XxxYyy.vue`；script 中超过 3 个 handler 的逻辑抽到 `src/composables/useXxx.ts`
- **TS 工具/类型**：按领域拆到 `utils/<domain>.ts` / `types/<domain>.ts`，在原 `index.ts` 保留 barrel 再导出（`types/index.ts` 已采用此模式）
- `db/mod.rs` 按表拆分: `db/packages.rs`, `db/proxies.rs` 等
- Vue 通用组件拆分: 通用组件提取到 `src/components/`

### 统一命名规范表（强制 / 可机械校验）
| 对象 | 语言 | 规则 | ✅ 正例 | ❌ 反例 |
|------|------|------|---------|---------|
| 函数 / 变量 / 参数 | Rust | `snake_case` | `get_packages_from_archive` | `getPackagesFromArchive` |
| 结构体 / 枚举 / trait / type | Rust | `PascalCase` | `CheckResult`、`VersionChecker` | `check_result`、`version_checker` |
| 常量 / static | Rust | `UPPER_SNAKE_CASE` | `CACHE_CLEANUP_RULE` | `cacheCleanupRule` |
| 模块 / 文件名 | Rust | `snake_case` | `backup_install_helpers.rs` | `backup-install-helpers.rs` |
| 测试函数 | Rust | `test_<行为>_<预期>`（全仓统一 `test_` 前缀） | `test_validate_rejects_relative_path` | `validate_rejects_relative_path`（现存历史写法，逐步迁移） |
| Tauri 命令 | Rust | `snake_case`，且必须同时标注 `#[tauri::command]` 与 `#[specta::specta]` | `sync_from_aur` | `syncFromAur`；只标一个宏 |
| IPC 前端方法 | TS | `camelCase`（由 bindings 自动转换，不手写） | `commands.getSoftwareDetail` | `commands.get_software_detail` |
| 变量 / 函数 / 参数 | Vue / TS | `camelCase` | `selectedPkgnames`、`fetchView` | `selected_pkgnames`、`FetchView` |
| 类型 / 接口 / 枚举 / 组件名 | Vue / TS | `PascalCase` | `SoftwareInfo`、`PageToolbar` | `softwareInfo`、`pageToolbar` |
| 组件文件名 | Vue | `PascalCase.vue`，与组件名一致 | `PackageTable.vue` | `package-table.vue` |
| 工具 / composable 文件名与导出名 | TS | `camelCase`；composable 必须 `use` 前缀且**文件名 = 导出名** | `usePackageList.ts` → `usePackageList()` | `packageActions.ts` → `usePackageActions()`（现存不一致，见清单 Q-L01） |
| 常量 | Vue / TS | `UPPER_SNAKE_CASE` | `MAX_TOASTS` | `maxToasts` |
| CSS 类名 | CSS | `kebab-case` | `.btn-primary` | `.btnPrimary` |

### 禁止重复实现（DRY，强制 / 可机械校验）
- **新增逻辑前必须先搜公共模块**：`rg -n '<关键词>' src src-tauri/src --glob '!**/target/**'`；确认无已有实现后再新建
- **同一逻辑模式出现 ≥ 3 处必须抽取公共方法**；≥ 2 处且涉及**安全 / 权限 / 命令拼装 / SQL**（路径校验、sudoers、pacman 调用、参数化查询）时也**必须**抽取，不设 3 处门槛
- **连续 ≥ 8 行完全相同的代码禁止复制粘贴**，必须抽函数或宏
- **公共方法落点（新增前优先放到这里）**：
  - 系统命令执行 → `src-tauri/src/commands/sysops/sys_command.rs`
  - sudoers / 提权 / 路径校验 → `src-tauri/src/commands/sysops/backup_install_helpers.rs`（其已 286 行，新增能力先拆出 `sysops/sudoers.rs`）
  - DB 锁获取与错误映射 → `src-tauri/src/db/mod.rs` 或 `src-tauri/src/errors.rs`
  - Checker 结果构造与日志 → `src-tauri/src/checkers/trait_def.rs` 或新建 `checkers/helpers.rs`
  - HTTP 客户端构造 → `src-tauri/src/http_client.rs`（已有 `shared_client()` / `build_client()`，禁止另起 `Client::builder()`）
  - GitHub API URL → 新建 `src-tauri/src/checkers/github/api_urls.rs`
  - 前端错误格式化 / 判断 → `src/utils/errors.ts`（新建）
  - 前端错误提示 → `src/composables/useToast.ts`（已有 `addToast`）
- 抽取后所有调用点必须改为调用公共方法，禁止「公共方法 + 保留旧副本」并存

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
  - **异步上下文禁止直接调用 `reqwest::blocking`（强制）**：`#[tauri::command] async fn`、`tokio::spawn` 闭包等运行在 tokio worker 线程，本身已处于 runtime 上下文；在其中直接调用 `reqwest::blocking::get` 等阻塞客户端，会因创建/销毁嵌套 runtime 触发 panic `Cannot drop a runtime in a context where blocking is not allowed`。阻塞网络请求必须用 `std::thread::spawn(|| reqwest::blocking::...).join()` 或 `tokio::task::spawn_blocking` 隔离到无 runtime 上下文的线程。本项目 `db/github_tag_cache.rs::check_and_extend_cache` 即采用 `std::thread::spawn` 方案（详见该文件注释与下方「GitHub tags 缓存」模块说明）。

### Vue/TypeScript 编码规范
- 组件命名：PascalCase（文件名和组件名一致）
- 变量命名：camelCase
- 使用组合式 API（Composition API）
- 类型定义放在 `src/types/` 目录
- 状态管理使用 Pinia store
- **Tauri 命令调用（强制）**：禁止在 composable/store/component/view 中直接 `import { invoke } from "@tauri-apps/api/core"`；所有 IPC 调用必须经 `src/api/` 对应领域模块封装，命令字符串与参数形状集中管理
- **StandardizedTable 选中状态同步（强制）**：`StandardizedTable` 内部以 `useTableState.selectedRows` 独立维护选中，仅通过 `selection-change` 事件向父组件回传选中行；任何依赖"已选集合"的工具栏/批量逻辑必须在中间层（如 `PackageTable`）订阅此事件并将 rowKey 转换为业务键后向上 emit，父组件再同步到外部 ref，否则工具栏按钮会因外部集合始终为空而误走"全量操作"分支（PackageList 真实发生过「勾选 13 个 → 全量更新上游」）。详细见 `docs/COMPONENTS_GUIDE.md` StandardizedTable 章节"重要约定"段

<!-- ========== AI 协同约定：多智能体协作的硬约束 ========== -->
### AI 协同约定（强制）
1. **改代码前先读规范**：必须先完整阅读本文件「代码规范（强制）」章节，并查阅 `CODE_QUALITY_ISSUES.md` 的「待修复」清单，避免重复引入已记录问题
2. **提交前必跑校验（逐条执行，任一失败不得提交）**：
   ```bash
   export HOME=/home/zxp-archlinux \
     CARGO_HOME=/run/media/zxp/LocalBak/git/My-AUR-Helper/.cargo \
     RUSTUP_HOME=/run/media/zxp/LocalBak/git/My-AUR-Helper/.rustup
   cargo fmt --all -- --check      # 格式
   cargo clippy --all-targets -- -D warnings   # lint
   cargo test --lib               # 单测（含 bindings 一致性）
   pnpm exec vue-tsc --noEmit     # 前端类型
   pnpm build                     # 前端构建
   # 300 行硬上限检查，输出必须为空（命令见「文件组织原则」）
   ```
   注：Bash 工具跑 cargo 需关闭 sandbox，否则 EACCES
3. **不擅自扩大改动范围**：只改任务指定的文件与函数；发现范围外问题时写入 `CODE_QUALITY_ISSUES.md`，不在本次提交中顺手修改
4. **新文件必须带文件头注释**（功能 / 工作流程 / 适用场景），`pub fn` 必须有 `///` 文档注释，注释语言为中文
5. **行数守护**：任何改动导致文件超过 300 行，必须在同一次提交内完成拆分；禁止把拆分留到「下次」
6. **禁止新增 `as unknown as` 类型逃逸**：类型不一致时在 `src/types/` 定义共享类型并与 bindings 对齐；存量 10 处逐步消除
7. 提交信息格式遵循本文件「Git 提交规范」章节

<!-- ========== IPC 铁律：前后端通信的唯一合法路径 ========== -->
### IPC 铁律（强制）
1. **后端命令双宏**：每个命令必须同时标注 `#[tauri::command]` 与 `#[specta::specta]`（当前 88 个命令 / 88 个 specta 宏，覆盖 100%，必须保持）
2. **注册唯一入口**：只允许 `src-tauri/src/ipc/registry.rs::build_specta_router()` 中的 `collect_commands!` 统一收集（T01 拆分后已从 `lib.rs` 迁至此处）；禁止在其他文件另建 `invoke_handler` 或手工注册
3. **改签名必须重新生成 bindings**：
   - debug 启动 `pnpm tauri dev` 自动导出；
   - 或显式执行 `cargo test --lib export_bindings`（release 前必做）；
   - 产物为 `src/bindings.ts`，**生成物禁止手改**；生成后需 `pnpm exec vue-tsc --noEmit` 验证前端类型
4. **前端唯一通道**：所有 IPC 调用只经 `src/api/<domain>.ts` 中的 `commands.*`（`commands` 从 `src/bindings.ts` 导入）；禁止在 component / composable / store / view 中 `import { invoke } from "@tauri-apps/api/core"`（当前 0 处，必须保持 0 处）
5. **前端 api 模块职责**：`src/api/` 按领域一个文件，集中管理命令调用与参数形状；组件层不得直接拼命令参数对象
6. **类型对齐优先**：bindings 类型与业务类型不一致时，在 `src/types/` 定义共享类型，禁止在 api 层散布 `as unknown as` 强转
7. **bindings 一致性必须走字节级 diff 闭环，禁止用 `git diff` 判断**：`src/bindings.ts` 是 tauri-specta 自动生成物，且**未纳入 git 跟踪**（`git status` 显示 `??`），因此 `git diff HEAD -- src/bindings.ts` 恒为空、属于**假阴性**，严禁据此判断 IPC 是否变化。唯一合法闭环（每次改动命令签名 / 参数 / 返回类型 / 命令 `///` 文档注释后必做，specta 会导出 doc comment）：
   ```bash
   # 全程在仓库根目录执行；用子 shell 跑 cargo，避免 cd 改变当前目录导致第 3 行路径错乱
   cp src/bindings.ts /tmp/bindings_baseline.ts
   (cd src-tauri && cargo test --lib export_bindings)
   diff /tmp/bindings_baseline.ts src/bindings.ts   # 输出必须为空
   ```
   diff 非空即代表 IPC 契约变化，必须修到完全一致或经显式评审接受。可选改进建议（需单独授权，不得擅自 `git add`）：把 `src/bindings.ts` 纳入 git 跟踪，或在 CI 增加「重新生成后 diff 必须为空」的门禁检查。

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
| `src-tauri/src/db/connection.rs` | Database 结构体、连接创建、表初始化和迁移；`new()` 开启 `PRAGMA foreign_keys=ON`，引用完整性由子表 `ON DELETE CASCADE` 保证（原 `ensure_no_fk_constraints` 死代码已移除） |
| `src-tauri/src/db/schema.rs` | 数据库 Schema 定义（子表 `aur_info`/`upstream_info` 通过 `ON DELETE CASCADE` 关联 `software_info`） |
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
| `src-tauri/src/db/stats.rs` | 仪表盘统计聚合查询（各模块计数）；单次 SQL 内 8 个标量子查询合并，仅一次 DB 往返（原 8 次独立 COUNT 已合并） |
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
| `src-tauri/src/commands/sysops/software_sync/import_aur.rs` | 从 AUR RPC 搜索并导入软件包到数据库 |
| `src-tauri/src/commands/sysops/software_sync/upstream.rs` | 上游版本批量检查命令（`check_all_upstream`）：映射任务交给 batch 引擎分类并行检查；Manual 包跳过网络仅回传标记；一次性批量读取 AUR 版本 + 内存映射语言 ID，消除写库阶段 N+1 查询与反复加锁 |
| `src-tauri/src/commands/sysops/software_sync/batch.rs` | 上游批量分类并发执行引擎（核心调度）：按检查器类型分桶（Manual 跳过 / Browser 严格限并发 / 网络类全局限并发）；单一 `JoinSet<UpstreamCheckResult>` 承载所有 `run_one` 任务（浏览器桶 + 必然 REST 桶 + 未命中 GraphQL 的回落桶），分两段 `join_next()` drain（先回收浏览器+必然 REST，待 GraphQL 完成后把未命中回落任务并入同一集合再 drain）；GitHub 批量查询与「必然 REST」桶并发启动，仅未命中者补回落 REST（避免重复请求与重复处理）；类型/常量/第二段 drain 已迁至 `batch_engine.rs` |
| `src-tauri/src/commands/sysops/software_sync/batch_engine.rs` | 批量引擎的类型与辅助层（2026-09-03 从 batch.rs 拆分以符合 300 行约束）：`PackageTask` 任务结构体、`BatchOutcome` 结果结构体、`MAX_BROWSER_CONCURRENCY` / `MAX_NETWORK_CONCURRENCY` 并发常量、`drain_fallback_results` 第二段 drain（区分 tags 回填结果与普通 REST 结果，回填后按各包正则重算版本） |
| `src-tauri/src/commands/sysops/software_sync/batch_helpers.rs` | 批量检查辅助函数（`classify` 任务分类、`run_one` 单包检查、`check_with_retry` 重试） |
| `src-tauri/src/commands/sysops/software_sync/batch_cache.rs` | GitHub tags 缓存写盘封装（`write_github_tag_cache`）：由 batch 引擎通过 `on_cache_ready` 回调在检查完成后调用，避免跨 await 持有 `Database` 引用（`Database` 非 `Send`） |
| `src-tauri/src/commands/sysops/software_sync/cache.rs` | 检查前的 GitHub tags 缓存读取与版本重算（`check_github_cache`）：命中缓存则整仓跳过网络；同仓库不同正则的包各自调 `recompute_version_from_tags` 计算（不可共用同一个正则） |
| `src-tauri/src/commands/sysops/software_sync/cache_fill.rs` | 缓存补写：检查后为「成功产出版本但缓存仍缺失」的仓库补拉 tags 写盘（`collect_missing_repos` 同步判定 → `fetch_repo_tags` async 拉取 → `write_back` 同步写库）；`fill_tags` 正则传 `None`（仓库级共享，按单包正则早停会截断其他 major 线） |
| `src-tauri/src/commands/sysops/software_check/` | 软件检查命令模块（目录结构） |
| `src-tauri/src/commands/sysops/software_check/mod.rs` | 模块声明和导出（不含具体实现） |
| `src-tauri/src/commands/sysops/software_check/selected.rs` | 选中包上游版本检查命令（`check_selected_upstream`，从原 software_check.rs 拆分）：缓存校验 → 并行检查 → 缓存补写 → 批量写库，与 `check_all_upstream` 保持同一套缓存逻辑 |
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
| `src-tauri/src/checkers/github/release_history.rs` | GitHub Releases 历史遍历扫描（分页 + 资产过滤回退；max_pages=30/per_page=30 共 900 条，为响应体积和超时折衷；老版本包如 electron2-bin 的正则匹配范围超出此限会静默返回空） |
| `src-tauri/src/checkers/github/git_describe.rs` | Git Describe 格式化（-git 包专用） |
| `src-tauri/src/checkers/github/graphql_batch.rs` | GitHub GraphQL 批量检查器：`batch_check_github` 用 alias 在单次请求批量查多仓库 tags/releases+license/languages；按 `owner/repo` 构建哈希索引一次性完成去重与按仓库匹配（O(n)，非 O(n²)）；分块用 `JoinSet` 并行发送请求；git 包/无 Token/仓库缺失回落逐包 REST；select_version 镜像 REST 路径；定义 `GithubBatchItem` / `GithubBatchOutcome` / `RepoCache` 三个结构体并 re-export `fill_tags` / `collect_fallback_tasks`（实现在 `graphql_batch_tags.rs`） |
| `src-tauri/src/checkers/github/graphql_batch_tags.rs` | GraphQL 批量检查的 tags 回填辅助（2026-09-03 从 graphql_batch.rs 拆分）：`fill_tags` 从 GraphQL 已扫位置继续 REST 翻页补齐 tags（正则命中早停，`MAX_TAG_PAGES=60` 上限），`collect_fallback_tasks` 识别「version 为空且有正则」需要回填的包 |
| `src-tauri/src/checkers/github/graphql_batch_query.rs` | GraphQL 查询构建与分块执行底层（`build_query` 用 `serde_json::to_string` 生成合法 JSON 字面量；`query_chunk` 执行一次分块请求） |
| `src-tauri/src/checkers/github/graphql_batch_parse.rs` | GitHub GraphQL 快照解析（RepoSnapshot / ReleaseData / parse_snapshot） |
| `src-tauri/src/checkers/github/graphql_batch_helpers.rs` | GraphQL 批量检查的版本挑选辅助（`select_version` / `tags_max_version`，严格镜像 REST 路径保证结果一致）；`select_version` 接受 `all_tags: Option<&[String]>` 供回填后重算 |
| `src-tauri/src/db/github_tag_cache.rs` | GitHub tags 缓存模块（SQLite 表 CRUD、增量校验 CacheCheckResult 枚举、recompute_version_from_cache、check_and_extend_cache 同步函数；内部阻塞 HTTP 已用 std::thread::spawn 隔离到独立 OS 线程，可在 async 命令内直接同步调用） |
| `src-tauri/src/db/migration_github_tag_cache.rs` | github_tag_cache 表迁移（含 cached_version 列 ALTER） |
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
| `src/api/` | 前端统一 API 抽象层（集中封装全部 Tauri invoke 调用，按领域分模块） |
| `src/api/software.ts` | 软件包领域 API（CRUD/AUR 同步/上游检查/URL 校验） |
| `src/api/proxy.ts` | 代理领域 API（增删改查/批量测试/文件下载解析） |
| `src/api/license.ts` | License 领域 API（增删改查/SPDX 同步） |
| `src/api/language.ts` | 编程语言领域 API（查询/upsert/删除） |
| `src/api/settings.ts` | 设置与日志领域 API（get/set/list + 日志读取/清空/应用） |
| `src/api/backup.ts` | 备份领域 API（列表/扫描/去重/安装/删除/子目录） |
| `src/api/cache.ts` | 缓存领域 API（列表/扫描/安装/清理/备份到子目录/备份到已有目录） |
| `src/api/dashboard.ts` | 仪表盘领域 API（get_dashboard_stats 统计聚合） |
| `src/api/sudoers.ts` | sudoers 免密配置领域 API（备份安装/缓存安装/缓存清理三组检测与命令获取） |
| `src/views/` | 页面组件（每个页面一个文件） |
| `src/components/base/` | 基础UI组件（Button、Card、Input、Select、Message、StatCard、Badge） |
| `src/components/common/` | 通用组件（StandardizedTable、StandardizedModal、PaginationControls、ProgressBar、PageToolbar等） |
| `src/components/layout/` | 布局组件（Sidebar、TabBar、BottomToolbar、PopupLayout、LogPanel等） |
| `src/components/package/` | 软件包相关组件（SoftwareDetailModal、SoftwareFormModal、PackageRowActions、PackageTable、各InfoCard等） |
| `src/components/backup/` | 备份管理组件（RowActions、InfoDialog、SudoersDialog等） |
| `src/components/cache/` | 缓存管理组件（CacheRowActions） |
| `src/components/dashboard/` | 仪表盘组件（ModuleCard 模块卡片） |
| `src/components/proxy/` | 代理管理组件（ProxyEditModal、ProxyDetailModal、ProxyClearConfirmModal） |
| `src/components/settings/` | 设置页面组件（SettingsCard、SettingRow、各配置Section） |
| `src/components/filter/` | 筛选组件（FilterBar） |
| `src/components/enum/` | 枚举管理组件（LanguageFormModal、LicenseFormModal） |
| `src/utils/enums.ts` | 枚举常量和共享选项（packageTypes/checkerTypes/filterOptions） |
| `src/utils/format.ts` | 通用格式化工具（时间戳/License/JSON列表/枚举名称/语言名称） |
| `src/utils/felog.ts` | 前端诊断日志工具（feDebug/feInfo/feWarn/feError，经 IPC 转发到终端） |
| `src/composables/` | 组合式函数（hooks） |
| `src/composables/useCacheDirs.ts` | 缓存目录管理 composable（SettingsCacheSection 和 CacheManager 复用；加载函数并行读取全部设置键） |
| `src/composables/footer.ts` | 底部状态栏状态管理 |
| `src/composables/packageActions.ts` | 软件包操作逻辑（同步、检查、删除） |
| `src/composables/usePackageList.ts` | 软件包列表页逻辑（分页、搜索、选择；筛选类型/选项/默认态/纯匹配函数已拆至 filterConfig.ts） |
| `src/composables/filterConfig.ts` | 软件包列表筛选配置与纯匹配函数（FilterState/QuickFilterKey/quickFilterOptions/createDefaultFilterState + matchesSingleQuickFilter/matchesQuickFilters/matchesConditionFilters） |
| `src/composables/useBackupList.ts` | 备份管理列表页逻辑（分页、搜索、选择） |
| `src/composables/useBackupInstall.ts` | 备份包安装逻辑（sudoers 检测、安装、包信息查询） |
| `src/composables/useCacheList.ts` | 缓存管理列表页逻辑（分页、搜索、选择） |
| `src/composables/useCacheBackupActions.ts` | 缓存备份操作逻辑（去重、备份新版本、备份到目录） |
| `src/composables/useCacheCleanup.ts` | 缓存清理操作逻辑（系统缓存、自定义缓存目录、sudoers 检测） |
| `src/composables/useSudoers.ts` | sudoers 免密配置共享 composable（备份安装/缓存安装/缓存清理三场景复用，命令名参数化注入） |
| `src/composables/useCacheManagerInit.ts` | 缓存管理页初始化聚合（加载缓存表/缓存目录/backup_dir/子目录/sudoers） |
| `src/composables/useCacheInfoNav.ts` | 缓存详情弹窗导航与选择逻辑（从 CacheManager 抽取） |
| `src/composables/useBackupInfoNav.ts` | 备份详情弹窗导航与选择逻辑（从 BackupManager 抽取） |
| `src/composables/useSoftwareForm.ts` | 软件包表单逻辑（验证、自动检测） |
| `src/composables/useLicenseSelect.ts` | License 可搜索下拉框逻辑 |
| `src/stores/` | Pinia 状态管理 |
| `src/types/` | TypeScript 类型定义（index.ts 为 barrel 再导出；按领域拆分：package/proxy/enum/backup/cache/settings/dashboard） |
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
| `src-tauri/src/checkers/github/graphql_batch.rs` | 376 | `graphql_batch.rs` | 217 | 2026-08-19 | ✅ 完成 |
| | | `graphql_batch_helpers.rs` | 171 | | 新文件（版本选择/比较辅助） |
| `src-tauri/src/commands/sysops/software_sync/batch.rs` | 371 | `batch.rs` | 232 | 2026-08-19 | ✅ 完成 |
| | | `batch_helpers.rs` | 158 | | 新文件（任务分类/单包检查/重试辅助） |
| `src/types/index.ts` | 313 | `index.ts`（barrel 再导出） | 17 | 2026-08-19 | ✅ 完成 |
| | | `types/package.ts` | 125 | | 新文件（软件包相关类型） |
| | | `types/proxy.ts` | 37 | | 新文件（代理类型） |
| | | `types/enum.ts` | 32 | | 新文件（License/语言枚举） |
| | | `types/backup.ts` | 41 | | 新文件（备份类型） |
| | | `types/cache.ts` | 39 | | 新文件（缓存类型） |
| | | `types/settings.ts` | 41 | | 新文件（设置/日志类型） |
| | | `types/dashboard.ts` | 40 | | 新文件（仪表盘/Footer 状态） |
| `src/views/CacheManager.vue` | 301 | `CacheManager.vue` | 282 | 2026-08-19 | ✅ 完成 |
| | | `composables/useCacheManagerInit.ts` | 71 | | 新文件（onMounted 初始化聚合） |
| `src-tauri/src/commands/sysops/software_check.rs` | 300+ | `software_check/selected.rs` | 273 | 2026-09-03 | ✅ 完成 |
| | | `software_check/mod.rs` | — | | 新文件（模块声明与导出） |
| `src-tauri/src/checkers/github/graphql_batch.rs` | 326 | `graphql_batch.rs` | 181 | 2026-09-03 | ✅ 完成 |
| | | `graphql_batch_tags.rs` | 186 | | 新文件（`fill_tags` / `collect_fallback_tasks`） |
| `src-tauri/src/commands/sysops/software_sync/batch.rs` | 362 | `batch.rs` | 299 | 2026-09-03 | ✅ 完成 |
| | | `batch_engine.rs` | 134 | | 新文件（`PackageTask` / `BatchOutcome` / 并发常量 / `drain_fallback_results`） |
| `src/composables/usePackageList.ts` | 390 | `usePackageList.ts` | 260 | 2026-11-13 | ✅ 完成 |
| | | `filterConfig.ts` | 140 | | 新文件（composables，筛选类型/选项/默认态/纯匹配函数） |
| `src/views/PackageList.vue` | 841 | `PackageList.vue` | 299 | 2026-11-13 | ✅ 完成 |
| | | `AurSearchModal.vue` | 295 | | 新文件（components/package，AUR 搜索导入弹窗） |
| | | `QuickFilterDropdown.vue` | 157 | | 新文件（components/package，工具栏单选快速筛选下拉） |
| | | `PackageList.css` | 149 | | 新文件（views，页面样式外置） |
| `src/components/package/BlockView.vue` | 376 | `BlockView.vue` | 182 | 2026-11-13 | ✅ 完成 |
| | | `BlockView.css` | 198 | | 新文件（components/package，块视图样式外置） |

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
1. 前端 Vue 组件/store/composable 调用 `src/api/` 对应领域模块（如 `softwareApi.listSoftware()`）
2. API 模块内部调用 `invoke("command_name", args)` 发起 IPC 请求（命令字符串与参数形状集中管理）
3. Tauri 路由到 `commands/` 模块中的对应处理函数
4. 命令函数调用业务逻辑层（checkers/aur/proxy/backup）
5. 业务逻辑层调用 `db/` 模块进行数据库操作（批量写库包裹事务，保证原子性）
6. 数据库操作通过 rusqlite 执行 SQL 查询
7. 结果通过 serde 序列化为 JSON 返回前端
8. 前端 API 模块返回 Promise<T>，store 更新状态，组件响应式渲染

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
- `graphql_batch.rs`: GitHub GraphQL 批量检查器（`batch_check_github`）：用 alias 在单次请求里批量查多个仓库的 tags/releases + license/languages，按 `owner/repo` 去重；git 包/无 Token/仓库缺失回落逐包 REST；`select_version` 严格镜像 REST 路径保证结果一致（2026-09-03 起 tags 回填辅助已拆至 `graphql_batch_tags.rs`）
- `graphql_batch_tags.rs`: `fill_tags`（REST 翻页补齐 tags，正则命中早停）+ `collect_fallback_tasks`（识别需回填的包）
- `graphql_batch_query.rs`: GraphQL 查询构建（`build_query`）与分块执行（`query_chunk`）
- `graphql_batch_parse.rs`: GitHub GraphQL 响应解析（`RepoSnapshot` / `ReleaseData` / `parse_snapshot`）
- `graphql_batch_helpers.rs`: 版本挑选辅助（`select_version` / `tags_max_version`）
- `github_tag_cache.rs`（db 层）: GitHub tags 缓存模块——SQLite 表 CRUD、`CacheCheckResult` 枚举（Hit/Extend/Recalculate/Deleted/Miss）、`recompute_version_from_cache`（从缓存 tags JSON 重算版本）、同步函数 `check_and_extend_cache`（缓存过期时调 `releases/latest` 轻量验证，版本一致则顺延 TTL，不一致则用缓存 tags 重算避免全量重拉）；注意：因 `Database` 不可 Send，该方法为同步函数；其内部阻塞 HTTP 已用 `std::thread::spawn` 隔离到独立 OS 线程（避开 tokio worker 上下文，否则 `reqwest::blocking` 创建/销毁嵌套 runtime 会 panic "Cannot drop a runtime in a context where blocking is not allowed"），因此调用侧可直接在 async 命令的 `state.db.lock()` 内串行同步调用，无需再包 spawn_blocking

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
1. **AUR 版本处理**：提取完整版本信息（epoch / version / pkgrel）；归一化只用于对齐上游 tag 表示差异，比较时必须把 epoch 与 pkgrel 补回可比较串，否则 vercmp 语义被绕过（见 `utils.rs::build_aur_comparable`）
2. **上游版本处理**：清洗和标准化版本号，支持自定义规则
3. **版本比较**：基于 vercmp 算法，支持多种版本格式
   - **pkgrel 比较语义（重要）**：pkgrel（末尾 `-<纯数字>` 段）仅在 pkgver 完全相等时才参与比较；pkgver 不等时以 pkgver 为准，**禁止把 pkgrel 当作版本组件错位比较**。例如 `9.0.1-1` 必须判定为大于 `9.0-5`（pkgver `9.0.1` > `9.0`），而非因 pkgrel `1` < `5` 误判更旧。实现见 `comparison.rs` 的 `split_pkgrel` + `compare_vercmp`。
   - **两个比较入口**：「上游 vs AUR」检查用 `versions::compare_versions`（`utils.rs`，归一化后补回 epoch/pkgrel）；缓存备份「备份新版」用 `versions::compare_vercmp`（`comparison.rs`，直接比较原始串）。两者均基于 vercmp，禁止 semver 语义。
   - **git describe 快路径**：两边版本都含 `.rN.gHASH` 时跳过归一化直接按 vercmp 比较，保留 revision，避免 -git 包漏报更新。
   - **系统 vercmp 对照**：`comparison/tests.rs` 以系统 `vercmp` 实测值做一致性回归；自实现含项目自定义预发布/tilde 扩展，与系统 vercmp 并非逐位等价（`1.0a`、`1.0~beta` 两组刻意不同）。
   - **AUR 补充信号**：AUR RPC v5 的 `OutOfDate` 由 `versions::is_outdated_with_aur_signal` 纳入判定，仅在 vercmp 判为「版本相同」时兜底，不得压过确定的版本关系。
4. **特殊字符处理**：将 `-` 转换为 `_` 符合 AUR 规范
5. **AUR 批量基线**：`aur/metadata.rs` 从 `packages-meta-ext-v1.json.gz` 一次拉取全量元数据建索引（ext 版才含 Depends/MakeDepends/OptDepends/License），失败自动降级到 RPC v5 批量；设置项 `aur_use_metadata_archive` 可关闭

### 日志规约（高频纯计算链路）
- `logger.rs::init` 全局 `LevelFilter::Debug`，且 `enabled()` 仅放行 `<= Level::Debug` → **TRACE 不落盘**。版本模块所有逐条 `debug!` 已统一降为 `trace!`（`utils.rs` / `upstream.rs` / `comparison.rs` / `aur.rs` 共 19 处）。
- 原因：这些纯计算函数被批量检查（1891 包）与列表浏览（387 包）高频调用，逐条 DEBUG 单日产生约 1.3 万行噪声；一次进入 `/updates` 曾刷出 765 行「解析上游版本」（每包 2 次 `UpstreamVersion::parse` × 387）。
- 高频展示链路优先用静默归一化 `UpstreamVersion::normalized()`（只清洗、不打日志），不要用带日志的 `parse` / `parse_with_rules`；两者共用 `cleanup*`，由单测 `normalized_matches_parse` 锁定一致性。
- 需要逐条排障时临时把 `logger.rs::init` 的 `LevelFilter::Debug` 调成 `LevelFilter::Trace`，不要把这些调用改回 `debug!`。

<!-- ========== sudoers 免密配置规约 ========== -->
## sudoers 免密配置（备份安装 / 缓存安装 / 缓存清理共用）

### 硬性规约
1. **单一文件**：三类规则统一写在 `/etc/sudoers.d/aur-helper-backup`，权限 `0440`。任何入口**禁止 `tee`（覆盖）**，必须 `tee -a` / `>>` 追加，并用 `grep -qF` 保证幂等；历史上 `get_sudoers_command` 用覆盖写冲掉其它两类规则，导致「已配置 policy 仍要求密码」。
2. **规则与实执行命令逐 token 一致**：`CACHE_CLEANUP_RULE`（`backup_install_helpers.rs`）定义清理规则，`CACHE_CLEANUP_ARGS` 定义传给 `sudo` 的参数表，两者由单测 `cleanup_args_match_sudoers_rule` 约束。sudoers 按完整命令行匹配，命令名须用绝对路径 `/usr/bin/find`，参数不得含 `*` 通配（否则 `/var/cache/pacman/pkg/../../` 可被放行）。
3. **一律 `sudo -n`**：GUI 无可交互终端，不带 `-n` 时免密未生效会让 sudo 挂起等待密码，表现为页面「卡死」。检测与执行都不得退化成密码提示。
4. **检测用 `sudo -n -l`**：`sudo_privileges_text()` + `has_nopasswd_rule()` 解析实际生效的 NOPASSWD 行。禁止用 `sudo -n cat <sudoers文件>`（cat 通常不在免密白名单，必然失败）、禁止直接 `read_to_string`（`/etc/sudoers.d` 为 0750，普通进程读不到）、更禁止用 `sudo -n rm -rf ...` 之类的探测命令（一旦命中规则会真的删文件，且与实际命令无关）。注意 `sudo -l <cmd>` 在存在 `(ALL:ALL) ALL` 通用规则时会返回 0，**不能**据此判断免密。
5. **配置命令单次提权**：生成 `sudo sh -c '...'`，在 root 内完成幂等判断 + 追加 + `chmod 440`，避免多次 sudo 调用或 `2>/dev/null` 吞掉密码提示。

<!-- ========== 软件更新模块 ========== -->
## 软件更新模块（导航中位于「软件管理」之后）

### 页面与文件
- 路由 `/updates`，页面 `src/views/SoftwareUpdate.vue`；弹窗 `src/components/update/UpdateDiffDialog.vue`
- 复用 `PageToolbar`（三段式工具栏）+ `StandardizedTable` + `StandardizedBadge` + `useListBase`；图标令牌 `navUpdates / updatePreview / updateApply` 统一注册在 `src/icons/index.ts`
- 弹窗内容区自然撑开、禁用滚动条；底部统一「确认 / 取消」两个图标+文本按钮
- 前端类型 `src/types/update.ts`、IPC 封装 `src/api/update.ts`；后端 `models/software_update.rs`、`db/software_update.rs`、`commands/sysops/software_update/`

### 列表范围与状态推导（重要：不新增表、不改 schema）
- **列表只显示 `software_info.is_outdated = 1` 的包**：`db/software_update.rs::list_update_candidates` 的 JOIN 必须带 `WHERE s.is_outdated = 1`（已有 `idx_software_outdated` 索引）。该字段由版本检查链路写入，是「是否需要更新」的唯一事实来源；页面浏览时**禁止重新跑 vercmp 判定筛选**，也不扫描未过期的包
- 「是否已写回」**不落库**：写回本地 PKGBUILD 后 AUR 现网版本并未改变，故以 `<aur_packages_dir>/<pkgname>/PKGBUILD` 的实际 pkgver 实时推导，rescan 即刷新
- 状态 `SoftwareUpdateStatus` 仅描述本地写回进度：`pending`（待写回）/ `local_updated`（已写回·待推送 AUR）/ `missing`（PKGBUILD 缺失）/ `no_upstream`；**不含 `up_to_date`**，未过期的包根本不进列表
- **浏览时零版本比较、零逐包日志**：列表仅对上游版本做一次静默归一化 `UpstreamVersion::normalized()` 得到 `target_version`（即写回写入 pkgver 的值），再与本地 `local.pkgver` 做字面相等比较区分 `pending` / `local_updated`；不调用 `versions::compare_versions` / `is_outdated_with_aur_signal`，也不用带日志的 `parse`
- 每包只做 1 次纯字符串清洗 + 1 次本地 PKGBUILD 读取；`apply.rs::plan_rewrite` 同样用 `normalized()`，保证写回值与列表状态判定同源

### dry-run + diff 预览（强制）
- 调用链：`list_software_updates` → `preview_software_updates`（纯读，生成 diff）→ 弹窗确认 → `apply_software_updates`
- `apply` 从数据库重读上游版本，不信任前端传参，避免预览与执行之间版本过期
- 只改 `pkgver=` 与 `pkgrel=1`，不触碰 sha256sums / source；写前备份为同目录 `PKGBUILD.bak`，写失败尝试从备份恢复
- `validate_pkgname` 白名单 + 路径越界校验，防止目录穿越；单包失败不影响整批
- 写回不更新数据库；推送 AUR 后由既有「同步 AUR」链路刷新 `aur_info.aur_version`

<!-- ========== 标签栏与窗口图标 ========== -->
## 标签栏 / 窗口图标规约
1. **新增主窗口路由必须同步登记 `App.vue::routeTabs`**：映射 `path → { label, icon }`，图标须与 `Sidebar.vue` 对应导航项一致。历史上 watch 只维护 `routeLabels` 且对所有路由复用 `Icon.navPackages`，导致标签图标全部显示成「软件包」，且新增 `/updates` 后刷新页面不会自动打开标签。
2. `stores/tabs.ts::openTab` 对已存在的标签同步 `label` / `icon`，保证侧边栏与路由 watch 两条入口结果一致，重启或重新导航即可纠正旧图标。
3. **`bundle.icon` 首项必须是小尺寸 PNG（当前 `icons/32x32.png`）**：`tauri-codegen::find_icon` 取列表中**第一个** `.png` 作为 `default_window_icon`，编译期解码为 RGBA 内嵌；512×512 大图作为窗口/托盘默认图标在 GTK/X11 下可能只落地为旧式 `WM_HINTS` 单色位图，且徒增二进制体积。列表其余项（128/256/512 PNG + `icon.ico`）供打包器使用。
4. **Linux 任务栏图标走 `.desktop` + hicolor，而不是仅靠窗口属性**：XFCE/KDE 面板用 `WM_CLASS`（实测 `my-aur-helper`）匹配 `.desktop` 的 `StartupWMClass`，再由 `Icon=` 从 hicolor 主题取图。`scripts/install-dev-icon.sh` 负责生成 `~/.local/share/applications/my-aur-helper.desktop` 并安装 `~/.local/share/icons/hicolor/<size>/apps/my-aur-helper.png`，随后刷新 `update-desktop-database` / `gtk-update-icon-cache` / `xfce4-panel -r`。dev 模式没有安装打包产物，**必须执行该脚本**；只改 `bundle.icon` 不足以修正任务栏图标。
5. **`window_icon.rs::apply_window_icon` 在 setup 阶段对 `main` 窗口显式 `set_icon()`**：补齐 X11 `_NET_WM_ICON` ARGB 路径，作为生产打包环境的兜底；`tray.rs` 仍用 `default_window_icon()`，两者共用同一图标。
6. `index.html` 的 favicon 指向 `public/icon.png`（Vite 拷入 `dist/`），不要再引用仓库中不存在的 `/vite.svg`。

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
| `github_tag_cache` | GitHub tags/releases 快照缓存（按 owner+repo 存储，TTL 默认 24h，含 cached_version 用于增量校验） |
| `enum_licenses` | License 枚举表（SPDX ID、全名） |

### AUR 批量查询设置
| 设置键 | 默认值 | 说明 |
|--------|--------|------|
| `aur_batch_size` | 50 | AUR 批量查询每批数量上限（代码 `min(100)` 硬上限） |
| `aur_batch_interval` | 5 | AUR 批量查询间隔时间（秒） |
| `aur_use_metadata_archive` | 1 | 优先使用全量元数据归档；设 0/false/no/off 回退到 RPC v5 分批 |

**AUR RPC API 限制**：
- URI 最大长度 4443 字节，Info 请求超过约 200 个包时需要分批处理——**这是阈值，不是批次大小**；当前实现用 POST form body（`arg[]`），实际批次取 `aur_batch_size`，硬上限 100
- 每天每个 IP 最多 4000 次请求
- 搜索结果超过 5000 个时会失败

**AUR 全量元数据归档（`aur/metadata*`）规约**：
- 归档 `packages-meta-ext-v1.json.gz` 实测压缩后约 14 MiB、解压后约 74 MiB，顶层是 **JSON 数组**（不是 RPC v5 的 `{"results":[...]}`）；`parse.rs::parse_archive` 必须同时兼容两种形状，单测须用真实数组格式，否则真实归档会被误判「缺少 results 数组」后静默降级。
- `decompress_gzip` **必须在独立线程写 stdin，同时由 `wait_with_output` 并发读 stdout**：解压输出远超管道缓冲区（64 KiB），先 `write_all` 再读会双向死锁，表现为「starting new connection 之后永久无输出」，且不受 `http_timeout` 约束（HTTP 阶段已结束）。由 `parse_archive_large_payload_no_pipe_deadlock` 单测锁定。
- 下载 / 解压 / 解析 / 建立索引各阶段必须有 INFO 日志与耗时；归档失败降级到 RPC 时须打印降级参数（包数 / 每批 / 间隔），便于区分「卡死」与「19 批 × 5 秒的正常等待」。

### 列表每页行数设置
| 设置键 | 默认值 | 说明 |
|--------|--------|------|
| `list_page_size_software` | 50 | 软件管理页面每页行数 |
| `list_page_size_update` | 50 | 软件更新页面每页行数 |
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
- 完整规则见本文件「代码规范（强制）→ IPC 铁律」，以下为速查：
- 所有命令必须经 `lib.rs` 的 `collect_commands!` 注册，并同时标注 `#[tauri::command]` + `#[specta::specta]`
- 命令签名变更后必须重新生成 `src/bindings.ts`；前端只允许经 `src/api/*.ts` 的 `commands.*` 调用
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

### SSH 推送故障排查
- 若 push 时报 `Bad owner or permissions on /etc/ssh/ssh_config.d/*.conf`：该符号链接指向的 systemd-ssh-proxy 配置由 WorkBuddy 注入，权限属主为 `nobody`，SSH 拒绝读取。可临时用 `ssh -F /dev/null` 绕过，或联系管理员修复 `/etc/ssh/ssh_config.d/` 下对应文件的所有权。
- 若提示 `sudo: /etc/sudo.conf is owned by uid 65534`：说明 sandbox 环境 sudoers 配置异常，push 前请确认当前 shell 有正常 SSH 密钥访问权限。

### 自动提交规则
- AI 编程助手在完成每个任务后，必须自动执行上述提交流程
- 如果提交失败（如网络问题、权限问题），应立即告知用户并提供错误信息
- 禁止在未经用户确认的情况下修改 `.gitignore` 文件