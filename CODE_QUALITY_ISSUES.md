# 代码质量与合规性检查问题清单

> 检查日期：2026-11-25（本次为**全量复核**，以当前代码为准重新取证）
> 检查范围：`src/`（Vue/TS）与 `src-tauri/src/`（Rust）全量源码，共 43,289 行；排除 `target/` / `node_modules/` / `dist/` / 生成物
> 上版清单：2026-07-29 创建、2026-08-20 收尾，旧 22 项**全部已修复**，见「五、历史归档（旧清单 22 项）」
> **当前待修复：24 项**（高风险 5 项、中风险 14 项、低风险 5 项）
> 取证方式：`rg --glob '!**/target/**'` + `wc -l` + 逐文件 Read；每条均附真实文件:行号

---

## 目录

- [风险分级说明](#风险分级说明)
- [一、高风险：文件行数超限（5 项）](#一高风险文件行数超限5-项)
- [二、中风险：重复代码 / 重复命令 / 重复调用（14 项）](#二中风险重复代码--重复命令--重复调用14-项)
- [三、低风险：命名与风格不一致（5 项）](#三低风险命名与风格不一致5-项)
- [四、修复进度跟踪](#四修复进度跟踪)
- [五、历史归档（旧清单 22 项）](#五历史归档旧清单-22-项)
- [附录：检查标准参考](#附录检查标准参考)

---

## 风险分级说明

| 级别 | 判定标准 |
|------|----------|
| **H（高）** | 违反 AGENTS.md 强制规范（300 行硬上限），直接影响多 AI 协作的可维护性；必须优先拆分 |
| **M（中）** | 同一逻辑多处重复，改动需同步多处，极易产生行为不一致；涉及安全/权限的重复按 M 高位处理 |
| **L（低）** | 命名、类型、文档表述不一致；不立即引发故障，但持续劣化一致性 |

---

## 一、高风险：文件行数超限（5 项）

### 行数审计总览（`wc -l`，降序）

| 文件 | 行数 | 超出 | 判定 | 编号 |
|------|------|------|------|------|
| `src/bindings.ts` | 1340 | 1040 | ✅ **豁免**：tauri-specta 生成物，由 AGENTS.md 规范直接登记；禁止手改，也不得为标注豁免而改文件 | — |
| `src-tauri/src/aur/metadata.rs` | 348 | 48 | ❌ 违规 | H-001 |
| `src-tauri/src/commands/sysops/software_sync/aur.rs` | 341 | 41 | ❌ 违规 | H-002 |
| `src-tauri/src/checkers/utils.rs` | 339 | 39 | ❌ 违规 | H-003 |
| `src-tauri/src/lib.rs` | 335 | 35 | ❌ 违规 | H-004 |
| `src/views/PackageList.vue` | 303 | 3 | ❌ 违规 | H-005 |

**临界预警（未超限但禁止再增逻辑，>270 行）**：
- W-001 `src-tauri/src/commands/sysops/backup_install_helpers.rs` 286 行（公共方法落点已满，新增能力必须先拆 `sysops/sudoers.rs`）
- W-002 `src-tauri/src/commands/sysops/cache_cleanup.rs` 279 行

---

### H-001：aur/metadata.rs 超限 48 行

| 属性 | 值 |
|------|-----|
| **位置** | `src-tauri/src/aur/metadata.rs`（全文件 348 行） |
| **关键区间** | 37-53 常量；60-144 客户端/缓存；146-293 `get_packages_from_archive`（148 行单函数）；294-320 `get_packages_info_smart`；322-348 测试 |
| **风险** | 高 |

**问题描述**：单文件混合了 HTTP 客户端构建、进程级缓存（OnceLock/Mutex）、归档下载重试、gzip 解析、RPC 降级调度、单元测试六类职责；其中 `get_packages_from_archive` 单函数 148 行，内部有下载重试循环（29-126）、解压解析（127-137）两段独立阶段，AI 修改时极易误伤重试与 304 分支。

**拆分建议**（拆后主文件约 120 行）：
1. `src-tauri/src/aur/metadata/cache.rs` — `ArchiveCache`、`cache_lock()`、`read_fresh_cache()`、`read_validators()`、`touch_cache()`、`write_cache()`、`CACHE_TTL`（对应 37-39、70-144）
2. `src-tauri/src/aur/metadata/download.rs` — `ARCHIVE_URL`、`ARCHIVE_DOWNLOAD_*` 常量、`build_archive_client()`、`download_archive_with_retry()`（从 146-293 抽下载循环），返回 `(bytes, etag, last_modified)` 或 `NotModified`
3. `src-tauri/src/aur/metadata.rs` — 仅保留 `get_packages_from_archive`（编排：缓存 → 下载 → `spawn_blocking(parse::parse_archive)` → 写缓存）与 `get_packages_info_smart`（降级调度）
4. `src-tauri/src/aur/metadata/tests.rs` — 322-348 测试迁入；`mod.rs` 或主文件仅 `#[cfg(test)] mod tests;`

**兼容性**：现有 `mod parse;`（22 行）保持不变；对外 `pub async fn` 路径不变。

---

### H-002：software_sync/aur.rs 超限 41 行

| 属性 | 值 |
|------|-----|
| **位置** | `src-tauri/src/commands/sysops/software_sync/aur.rs`（全文件 341 行） |
| **关键区间** | 42-239 `sync_from_aur`（198 行单命令）；243-340 `update_aur_info`（98 行） |
| **风险** | 高 |

**问题描述**：一个 Tauri 命令函数在单文件里完成「读设置 → 构造 client → 取包名 → 调 AUR 归档 → 解析字段映射 → 逐包构造结果 → 批量写库 → 失败标记 → 统计汇总」9 个阶段；同时塞入第二个命令 `update_aur_info`，职责混杂。

**拆分建议**（主文件降为编排 < 80 行）：
1. `software_sync/aur/config.rs` — `read_sync_settings(&db)`：读用户名 / 超时 / 批大小 / 间隔 / 归档偏好（对应 43-72）
2. `software_sync/aur/collect.rs` — `collect_sync_results(&pkgnames, &aur_results, &client)`：包名→数据映射、缺包分类、逐包字段解析（对应 90-114）
3. `software_sync/aur/persist.rs` — `persist_sync_results()` + `mark_missing_packages()`：批量 upsert `aur_info`、重算 `is_outdated`、失败标记（对应 115-190）
4. `software_sync/aur/update.rs` — 整段迁移 `update_aur_info`（243-340）
5. `software_sync/aur/mod.rs` — 仅声明与 `pub use`，保持 `commands::sysops::software_sync::aur::*` 注册路径不变（`lib.rs` 无需改动，bindings 不变化）

---

### H-003：checkers/utils.rs 超限 39 行

| 属性 | 值 |
|------|-----|
| **位置** | `src-tauri/src/checkers/utils.rs`（全文件 339 行） |
| **关键区间** | 14-44 静态正则集合；46-79 正则提取；81-126 owner/repo；128-162 版本清洗；164-205 归一化；206-235 URL/HTML 提取；237-339 测试（103 行） |
| **风险** | 高 |

**问题描述**：版本提取工具把 5 组静态正则、URL 解析、版本清洗、归一化、测试混在一起；`checkers/mod.rs` 以 `pub use utils::*` 对外暴露，任何内部改名会波及全部检查器。

**拆分建议**（主文件仅 barrel re-export，约 20 行）：
1. `checkers/utils/regex.rs` — `StaticRegexes` + `static_regexes()` + `extract_version_with_regex()`（14-79）
2. `checkers/utils/url.rs` — `extract_owner_repo()` + `extract_version_from_url()` + `extract_version_from_html()`（81-126、206-235）
3. `checkers/utils/version.rs` — `clean_version()` + `normalize_extracted_version()` + `is_pure_version()` + `extract_first_version()`（128-205）
4. `checkers/utils/tests.rs` — 237-339 测试按领域就近拆分或集中迁入
5. `checkers/utils.rs` 改为仅 `mod regex; mod url; mod version; pub use ...;`，外部 import 路径零改动

---

### H-004：lib.rs 超限 35 行

| 属性 | 值 |
|------|-----|
| **位置** | `src-tauri/src/lib.rs`（全文件 335 行） |
| **关键区间** | 12-25 模块声明；32-42 `export_bindings`；44-144 `build_specta_router`（含 88 条命令清单）；148-169 `AppState`/`CloseAction`/配置读取；172-325 `run()`（154 行）；327-335 测试 |
| **风险** | 高 |

**问题描述**：`lib.rs` 同时承担模块根、IPC 命令注册表、应用状态定义、数据库/日志/托盘/缓存初始化、窗口事件处理、退出钩子六类职责。其中 `build_specta_router` 的 `collect_commands!` 清单 88 行是**唯一 IPC 注册点**（必须整体保留、不可拆散到多处），`run()` 的 `setup` 闭包 106 行是最大改造阻力。

**拆分建议**（`lib.rs` 保留模块声明 + `run()` 骨架，约 90 行）：
1. `src-tauri/src/ipc/registry.rs` — 整体迁移 `build_specta_router()`（44-144，含完整 `collect_commands!`）+ `export_bindings()`（32-42），`pub(crate)` 暴露；`lib.rs` 仅 `use ipc::registry::{build_specta_router, export_bindings}`。**注意**：88 条命令必须一条不漏地整体搬迁，`cargo test --lib export_bindings` 生成结果须与原 `bindings.ts` 完全一致（diff 为空）
2. `src-tauri/src/app_state.rs` — `AppState`、`CloseAction`、`get_config_dir()`、`get_setting_string()`（148-169）
3. `src-tauri/src/app_setup.rs` — 从 `setup` 闭包拆 `init_database(app)`、`init_logger(&database, app, &config_dir)`、`init_memory_cache(&database, app)`、`spawn_cache_flush_task(app, &cache_config)`（183-289）
4. `src-tauri/src/app_events.rs` — `on_window_event` 的关闭处理（291-308）与 `RunEvent::Exit` 写盘（312-324）
5. `lib.rs` 保留 `pub mod` 声明与 `run()` 中的 Builder 链式调用

---

### H-005：PackageList.vue 超限 3 行

| 属性 | 值 |
|------|-----|
| **位置** | `src/views/PackageList.vue`（303 行：script 5-125、template 127-301、style 303） |
| **关键区间** | 145-168 八个批量操作按钮；171-226 `#filters` 区块（条件筛选 + 快捷筛选 + 视图切换） |
| **风险** | 高（超限虽小，但该文件是历史反复膨胀点） |

**问题描述**：模板内嵌 57 行工具栏筛选区块（两个 `<select>` + `QuickFilterDropdown` + 视图切换），与 25 行批量操作按钮混排；script 已通过 `usePackageList` / `usePackageActions` 抽过一次，模板未同步拆分。

**拆分建议**（拆后约 245 行）：
1. `src/components/package/PackageListFilterToolbar.vue` — 171-226 的 `#filters` 区块；props：`filterState` / `packageTypeFilterOptions` / `checkerTypeFilterOptions` / `quickFilterOptions` / `activeQuickFilterLabel` / `activeFilterCount` / `viewMode`；emits：`update:condition-filter` / `select-quick-filter` / `clear-quick-filter` / `set-view-mode`
2. `src/components/package/PackageBatchToolbar.vue` — 145-168 的 8 个批量按钮；props：`selectedCount` / `loading` / `checkLoading`；emits 与各操作对应
3. `PackageList.vue` 保留组合与数据流；`parseSelectValue` / `updateConditionFilter`（90-102）随筛选组件迁入

---

## 二、中风险：重复代码 / 重复命令 / 重复调用（14 项）

> 本节为本次治理**重点**。用户诉求「重复执行的命令之类的，需要合并为通用方法」逐条落实如下。

### M-001：获取系统用户名（`whoami`）重复 3 处

| 项 | 值 |
|----|-----|
| **位置** | `backup_install.rs:96-101`、`cache_install.rs:153-157`、`cache_cleanup.rs:206-210` |
| **重复模式** | `tokio::process::Command::new("whoami").output().await.map_err(\|e\| AppError::SystemCommand(format!("获取用户名失败: {}", e)))?` + `String::from_utf8_lossy(&output.stdout).trim().to_string()`，三处逐语义相同 |
| **风险** | 中（用户名进入 sudoers 规则，三处任一改动不同步会导致规则生成不一致） |
| **建议** | 抽取 `pub(crate) async fn current_username() -> AppResult<String>`，落点 `src-tauri/src/commands/sysops/system_cmd.rs`；非空校验在公共方法内完成 |

### M-002：sudoers 追加命令字符串重复 3 处

| 项 | 值 |
|----|-----|
| **位置** | `backup_install.rs:114-116`、`cache_install.rs:188-190`、`cache_cleanup.rs:215-217` |
| **重复模式** | 三处均拼装同一模板：`sudo sh -c 'grep -qF "{rule}" /etc/sudoers.d/aur-helper-backup \|\| printf "%s\n" "{line}" >> ...; chmod 440 ...'`；仅 `rule` / `line` 不同 |
| **风险** | 中（提权命令拼装重复，改动一处漏两处会造成幂等性或权限破损） |
| **建议** | 抽取 `pub(crate) fn build_sudoers_append_command(rule: &str, line: &str) -> String`，落点新建 `src-tauri/src/commands/sysops/sudoers.rs`；并加单测断言「不含覆盖写 tee」「含 grep 幂等」「权限 440」 |

### M-003：免密规则探测模式重复 4 处

| 项 | 值 |
|----|-----|
| **位置** | `backup_install.rs:83-86`、`cache_install.rs:135-138`、`cache_install.rs:176-184`、`cache_cleanup.rs:186-189`、`cache_cleanup.rs:200-204` |
| **重复模式** | `sudo_privileges_text().await` → `has_nopasswd_rule(&text, rule)` 组合，外加「已配置则返回提示语」分支 |
| **风险** | 中 |
| **建议** | 抽取 `pub(crate) async fn is_rule_effective(rule: &str) -> AppResult<bool>`（返回 `Option<String>` 原文更灵活），落点 `sysops/sudoers.rs`；`sudo_privileges_text` / `has_nopasswd_rule` 从已满的 `backup_install_helpers.rs`（286 行）一并迁入 |

### M-004：`sudo pacman -U --noconfirm` 安装流程重复 2 处

| 项 | 值 |
|----|-----|
| **位置** | `backup_install.rs:142-163`、`cache_install.rs:93-111` |
| **重复模式** | `with_pacman_write_lock(|| async { Command::new("sudo").args(["pacman","-U","--noconfirm",path])... })` + 相同成功/失败日志与 `AppError::SystemCommand("安装失败:\n{stderr}")` |
| **风险** | 中（涉及提权，落锁行为必须一致） |
| **建议** | 抽取 `pub(crate) async fn install_package_locked(safe_path: &Path, module_tag: &str) -> AppResult<String>`，落点 `sysops/system_cmd.rs`；`module_tag` 仅用于日志（"[备份管理]" / "[缓存管理]"） |

### M-005：`pacman -Qip` 查询包信息重复 2 处

| 项 | 值 |
|----|-----|
| **位置** | `backup_install.rs:46-60`、`cache_install.rs:56-70` |
| **重复模式** | `Command::new("pacman").args(["-Qip", path]).output().await` + `!status.success()` → `AppError::SystemCommand("pacman -Qip 失败: {stderr}")` + 返回 stdout |
| **风险** | 中 |
| **建议** | 抽取 `pub(crate) async fn pacman_package_info(path: &Path) -> AppResult<String>`，落点 `sysops/system_cmd.rs` |

### M-006：DB 锁获取 + 错误映射重复 32 处（跨 12+ 文件）

| 项 | 值 |
|----|-----|
| **位置（代表）** | `backup_install.rs:38,75,107,132`；`cache_install.rs:51,86,126,163`；`cache_cleanup.rs:80`；`fileops/cache_scan.rs:27,45,71,111`；`fileops/backup_scan.rs:59,135,177`；`sysops/backup_basic.rs:26,37,53,68`；`fileops/cache_backup/existing.rs:55,69,86,188`；`fileops/cache_backup/subdirectory.rs:38,56,98`；`fileops/scan.rs:137`；`fileops/backup_dedup.rs:145,189`；`sysops/upstream_validate.rs:116,190` |
| **重复模式** | `state.db.lock().map_err(\|e\| AppError::DatabaseError(format!("获取数据库锁失败: {}", e)))?` —— 全仓 32 处同文案同结构；另有 101 处 `db.lock()` 散布 |
| **风险** | 中（错误处理不一致；锁中毒时部分路径 panic） |
| **建议** | 在 `src-tauri/src/db/mod.rs` 增加 `pub fn lock_db(db: &Mutex<Database>) -> AppResult<MutexGuard<'_, Database>>`；在 `src-tauri/src/errors.rs` 实现 `impl<T> From<PoisonError<T>> for AppError`；逐步把 32 处替换为 `lock_db(&state.db)?`。**先补 `From`，再分批替换，每批跑 cargo clippy** |

### M-007：CheckResult 构造 + 「检查完成」日志重复 7 处

| 项 | 值 |
|----|-----|
| **位置** | `checkers/http.rs:100-106`、`gitee.rs:86-91`、`gitlab.rs:87-92`、`redirect.rs:229-236`、`browser.rs:217-222`、`github/tags_checker.rs:106-113` 与 `124-133` |
| **重复模式** | `if let Some(v) = &version { info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v) } else { debug!(...) }` + `Ok(CheckResult { version, ..Default::default() })`（tags_checker 为显式 `license: None, language_names`） |
| **风险** | 中（旧清单 L-010 只修了 `api_checker.rs` 的 4 处，其余 7 处未覆盖——旧清单结论已过时） |
| **建议** | 新建 `src-tauri/src/checkers/helpers.rs`：`pub(crate) fn log_check_result(pkgname: &str, version: &Option<String>)` 与 `pub(crate) fn finish_check(version: Option<String>, license: Option<String>, language_names: Vec<String>) -> CheckResult`；7 处统一调用 |

### M-008：UpstreamCheckResult 构造重复 6 处，且已有 builder 未被复用

| 项 | 值 |
|----|-----|
| **位置** | 已存在：`software_sync/batch_helpers.rs:61` 的构造函数；未复用点：`software_sync/cache.rs:87`、`upstream.rs:182`、`software_check/selected.rs:195`、`batch.rs:175`、`batch.rs:219`、`batch_engine.rs:104` |
| **重复模式** | 同样 6 字段结构（`pkgname` / `software_id` / `upstream_version` / `is_outdated` / …）在多处手工字面量构造，字段缺漏风险高 |
| **风险** | 中 |
| **建议** | 将 `batch_helpers.rs::build_upstream_result` 提为 `pub(crate)` 并统一签名，所有 6 处改为调用；落点 `software_sync/batch_helpers.rs` |

### M-009：`formatErrorMessage` 逐字重复 3 处

| 项 | 值 |
|----|-----|
| **位置** | `packageCheckActions.ts:26-31`、`packageSyncActions.ts:35-40`、`packageDeleteActions.ts:27-32` |
| **重复模式** | 三份实现**逐字符相同**（`String(e)` → `isNetworkError` → `${context}：网络异常…` / `${context}失败：${msg}`） |
| **风险** | 中 |
| **建议** | 抽取 `export function formatErrorMessage(e: unknown, context: string): string` 到新建 `src/utils/errors.ts`；三处删除本地定义并 import |

### M-010：`isNetworkError` 逐字重复 3 处

| 项 | 值 |
|----|-----|
| **位置** | `packageCheckActions.ts:13-22`、`packageSyncActions.ts:22-31`、`packageDeleteActions.ts:14-23` |
| **重复模式** | 三份实现**逐字符相同**：`String(e).toLowerCase()` 后 `includes("网络"/"timeout"/"连接"/"connect"/"fetch"/"request failed")` |
| **风险** | 中（网络错误关键词列表需一处维护） |
| **建议** | 与 M-009 同批抽取到 `src/utils/errors.ts`，关键词表提为 `const NETWORK_ERROR_KEYWORDS` |

### M-011：`showError(msg)` 局部封装重复 3 处

| 项 | 值 |
|----|-----|
| **位置** | `packageCheckActions.ts:60-62`、`packageSyncActions.ts:94-96`、`packageDeleteActions.ts:59-61` |
| **重复模式** | `function showError(msg: string) { addMessage(footer, "error", msg); }` 三处相同 |
| **风险** | 中 |
| **建议** | 在 `src/composables/useToast.ts` 增加 `export function showActionError(footer: FooterState, e: unknown, context: string)`，内部组合 M-009/M-010 的公共实现；三处仅保留调用 |

### M-012：HTTP Client 自行构造重复 8 处，绕过公共模块

| 项 | 值 |
|----|-----|
| **位置** | `aur/metadata.rs:61`、`checkers/redirect.rs:42`、`checkers/redirect.rs:60`、`commands/sysops/proxy_utils.rs:175`、`proxy/test.rs:29`、`commands/sysops/upstream_validate.rs:39`（及 `http_client.rs:135,159` 自身两处 builder） |
| **重复模式** | 项目已有 `src-tauri/src/http_client.rs` 提供 `shared_client()` / `build_client()` / `get_client_for_domain()` / `DEFAULT_USER_AGENT` / `DEFAULT_TIMEOUT_SECS`，但 6 个业务文件仍直接 `Client::builder()`，UA / 超时 / 代理策略可能不一致 |
| **风险** | 中 |
| **建议** | 统一改走 `http_client::build_client(&ClientConfig{...})`；确需特殊长超时的 `metadata.rs` 保留独立 client，但改为 `build_client` 传入覆盖参数，落点 `http_client.rs` |

### M-013：GitHub API URL 硬编码散落 14 处

| 项 | 值 |
|----|-----|
| **位置** | `checkers/github/release_history.rs:125`、`release.rs:65`、`graphql_batch_query.rs:66`、`tags.rs:76`、`repo_info.rs:32,74`、`graphql_batch_tags.rs:75`、`git_describe.rs:54,77,98,176,194,232`、`release_tags.rs:147`、`db/github_tag_cache_extend.rs:39` |
| **重复模式** | `"https://api.github.com/repos/{}/{}/..."` 与 `"https://api.github.com/graphql"` 手写 format 串，同一 endpoint（如 tags 列表）在 `tags.rs:76` 与 `graphql_batch_tags.rs:75` 完全相同 |
| **风险** | 中（改 API 版本 / 加 token 头需改 14 处） |
| **建议** | 新建 `src-tauri/src/checkers/github/api_urls.rs`：`pub(crate) const GH_API_BASE`、`pub(crate) fn repo_url(owner, repo)`、`fn releases_latest_url(...)`、`fn tags_url(...)`、`fn graphql_url()`、`fn license_url(...)`、`fn languages_url(...)`；14 处改为调用 |

### M-014：代理 gsettings 读写重复 3 处

| 项 | 值 |
|----|-----|
| **位置** | `commands/sysops/proxy_utils.rs:70`（mode）、`:85`（host）、`:100`（port） |
| **重复模式** | `std::process::Command::new("gsettings").args(["get", schema, key]).output()` + status 判断 + `String::from_utf8_lossy` 解析，三段同构 |
| **风险** | 中 |
| **建议** | 抽取 `fn gsettings_get(schema: &str, key: &str) -> AppResult<String>`，落点 `proxy_utils.rs` 文件内私有函数即可（不涉及跨模块，不新建文件） |

---

## 三、低风险：命名与风格不一致（5 项）

### L-001：composable 文件名与导出名不一致（4 处）

| 项 | 值 |
|----|-----|
| **位置** | `src/composables/packageActions.ts:26` → `usePackageActions`；`packageCheckActions.ts:49` → `usePackageCheckActions`；`packageDeleteActions.ts:52` → `usePackageDeleteActions`；`packageSyncActions.ts:63` → `usePackageSyncActions` |
| **反例 / 正例** | ❌ 文件名 `packageActions.ts`（无 `use` 前缀）却导出 `usePackageActions`；✅ 对照 `usePackageList.ts:24` 导出 `usePackageList`、`useCacheList.ts:43` 导出 `useCacheList`（文件名 = 导出名） |
| **风险** | 低（AI 按名检索时易漏，且破坏自动 import 一致性） |
| **建议** | 按 AGENTS.md「统一命名规范表」重命名为 `usePackageActions.ts` / `usePackageCheckActions.ts` / `usePackageDeleteActions.ts` / `usePackageSyncActions.ts`；同步更新全部 import（`PackageList.vue` 与各 composable 内部引用），重命名须用 `git mv` 保留历史 |

### L-002：Rust 测试函数两种命名风格并存

| 项 | 值 |
|----|-----|
| **位置** | `test_` 前缀 95 处（全仓多数）；无前缀描述式：`backup_install.rs:172,187,203,218`（`validate_accepts_pkg_inside_root` 等 4 个）、`cache_cleanup.rs:259,265,272`（`cleanup_args_match_sudoers_rule` 等 3 个） |
| **反例 / 正例** | ❌ `fn validate_rejects_relative_path()`；✅ `fn test_validate_rejects_relative_path()` |
| **风险** | 低 |
| **建议** | 规范统一为 `test_<行为>_<预期>`；本次仅修正上述 7 个明显反例，其余存量逐步迁移，禁止新增无前缀测试函数 |

### L-003：时间戳格式化函数命名相近、语义难区分

| 项 | 值 |
|----|-----|
| **位置** | `src/utils/format.ts:17` `formatTimestamp()`（本地化日期）与 `:90` `fmtTimestamp()`（定长 YYYY-MM-DD） |
| **风险** | 低（调用方易选错，且 `fmt` 缩写与全仓 `format` 全称风格不一致） |
| **建议** | 将 `fmtTimestamp` 重命名为 `formatDateOnly`，保留旧名一层 `export const fmtTimestamp = formatDateOnly` 过渡（下个版本删除）；禁止新增缩写式导出名 |

### L-004：AGENTS.md 规范描述与代码实际不一致（文档滞后）

| 项 | 值 |
|----|-----|
| **位置** | 原 `AGENTS.md:703` 写「命令参数使用 `#[command]` 宏声明」（实际全仓为 `#[tauri::command]`）；原 `:471` 写「API 模块内部调用 `invoke("command_name", args)`」（实际前端 `invoke(` 0 处、全部走 `commands.*` 74 处） |
| **风险** | 低（但会误导后续 AI 写出违规代码，属高杠杆问题） |
| **建议** | ✅ **本次交付物 A 已修复**：新增「IPC 铁律」小节并同步「IPC 通信规则」速查段；进度表标记已完成 |

### L-005：前端 api 层 `as unknown as` 类型逃逸 10 处

| 项 | 值 |
|----|-----|
| **位置（代表）** | `src/api/software.ts:54,59,64,69,119` 等；全仓共 10 处 |
| **反例 / 正例** | ❌ `return await commands.listSoftware() as unknown as SoftwareInfo[];`；✅ 在 `src/types/software.ts` 定义与 bindings 对齐的类型，或让 bindings 类型成为单一来源后直接返回 |
| **风险** | 低（类型错误在运行时才暴露） |
| **建议** | 禁止新增；在 `T05` 中逐处核对 bindings 生成类型，能在 `src/types/` 对齐的改掉，确需兼容的集中到 `src/api/software.ts` 顶部并加注释说明 |

---

## 四、修复进度跟踪

| 编号 | 类别 | 风险 | 修复状态 | 建议任务 | 备注 |
|------|------|------|----------|----------|------|
| H-001 | 文件超限 | 高 | ✅ **T01 已修复** | T01 | `aur/metadata.rs` 348 → 135 行编排入口 + `metadata/cache.rs` / `download.rs` / `tests.rs` |
| H-002 | 文件超限 | 高 | ✅ **T01 已修复** | T01 | `software_sync/aur.rs` 341 → 92 行编排骨架 + `aur/config.rs` / `collect.rs` / `persist.rs` / `update.rs`；`pub use update::*` 保留宏隐藏符号 |
| H-003 | 文件超限 | 高 | ✅ **T01 已修复** | T01 | `checkers/utils.rs` 339 → 26 行 barrel + `utils/regex.rs` / `url.rs` / `version.rs` / `tests.rs` |
| H-004 | 文件超限 | 高 | ✅ **T01 已修复** | T01 | `lib.rs` 335 → 104 行骨架 + `ipc/registry.rs` / `app_state.rs` / `app_setup.rs` / `app_events.rs`；bindings diff 为空 |
| H-005 | 文件超限 | 高 | ✅ **T02 已修复** | T02 | `PackageList.vue` 303 → **247 行**，抽 `PackageBatchToolbar.vue`(149) / `PackageListFilterToolbar.vue`(179) |
| W-001 | 临界预警 | — | ✅ 已核验 | T01/T03 | `backup_install_helpers.rs` 286 行，T03 未再塞入（sudoers 另建模块），保持 <300 |
| W-002 | 临界预警 | — | ✅ 已核验 | T03 | `cache_cleanup.rs` 279 行，保持 <300 |
| M-001 | 重复命令 | 中 | ✅ **T03 已修复** | T03 | `whoami` ×3 → `system_cmd::current_username()`；全仓仅剩 1 处 |
| M-002 | 重复命令 | 中 | ✅ **T03 已修复** | T03 | sudoers 追加 ×3 → `sudoers::build_sudoers_append_command()`；新增逐 token 单测 |
| M-003 | 重复命令 | 中 | ✅ **T03 已修复** | T03 | 免密探测 ×5 → `is_rule_effective` / `are_all_rules_effective` / `is_any_rule_effective` |
| M-004 | 重复命令 | 中 | ✅ **T03 已修复** | T03 | `sudo pacman -U` ×2 → `install_package_locked()`（保留写锁与错误格式） |
| M-005 | 重复命令 | 中 | ✅ **T03 已修复** | T03 | `pacman -Qip` ×2 → `pacman_package_info()` |
| M-006 | 重复错误处理 | 中 | 🟡 **T03 部分完成** | T03 | `db::lock_db()` 已落地，`From<PoisonError<T>>` 本已存在且真实生效；已迁移 12 处，剩余 23 处 `map_err` + 61 处直连见「六」「八」 |
| M-007 | 重复构造 | 中 | ✅ **T04 已修复** | T04 | 7 处 CheckResult 构造 + 完成日志 → `checkers/helpers.rs::finish_check`；api_checker 保留，见「七」 |
| M-008 | 重复构造 | 中 | 🟡 **T04 核验后保留** | T04 | 9 处 `UpstreamCheckResult` 语义差异大，`:61` 是 `run_one` 非 builder；强行合并会改变行为，见「七」 |
| M-009 | 重复逻辑 | 中 | ✅ **T04 已修复** | T04 | `formatErrorMessage` ×3 → `src/utils/errors.ts`（文案逐字等价） |
| M-010 | 重复逻辑 | 中 | ✅ **T04 已修复** | T04 | `isNetworkError` ×3 → `src/utils/errors.ts`，关键词表提为 `NETWORK_ERROR_KEYWORDS` |
| M-011 | 重复逻辑 | 中 | ✅ **T04 已修复** | T04 | `showError` ×3 → `useToast.ts::showActionError`，等价于原 `addMessage(footer,"error",msg)` |
| M-012 | 重复构造 | 中 | 🟡 **T03 核验后保留** | T03 | 8 处 `Client::builder()` 各有特殊策略，强行合并会改变行为；逐项理由见「六」 |
| M-013 | 重复常量 | 中 | ✅ **T03 已修复** | T03 | `api.github.com` ×15（9 文件）→ `checkers/github/api_urls.rs` 唯一落点 |
| M-014 | 重复命令 | 中 | ✅ **T03 已修复** | T03 | gsettings ×3 → `proxy_utils.rs` 文件内私有 `gsettings_get()` |
| L-001 | 命名 | 低 | ✅ **T05 已修复** | T05 | 4 个 composable `git mv` 为 `useXxx.ts`；改名前后 `rg` 复核，改名后旧名**零命中**，`vue-tsc` 兜底 exit 0 |
| L-002 | 命名 | 低 | ✅ **T05 已处理** | T05 | T01~T04 相关 15 个测试全部 `test_` 前缀（补齐 T01 2 处 / T03 3 处）；存量描述式命名 19+ 处按约定**一律不改**，约束仅对新增生效，见「八」 |
| L-003 | 命名 | 低 | 🟡 **T05 核验后保留** | T05 | `format.ts` 两函数已补对比 JSDoc；`fmtTimestamp → formatDateOnly` 波及 5+ 文件，收益低于风险，不改名，登记后续建议 |
| L-004 | 文档一致性 | 低 | ✅ **本次已修复** | — | AGENTS.md 新增 IPC 铁律并同步速查段 |
| L-005 | 类型安全 | 低 | 🟡 **T05 范围外保留** | T05 | `as unknown as` 10 处本批未动（超出收尾批次范围），登记后续专项治理 |

**统计（T05 收尾回填）**：H 类 5 项**全部修复**；临界监控 2 项已核验仍 <300；M 类 14 项中 **11 项已修复、3 项保留/部分完成**（M-006 部分、M-008 保留、M-012 保留）；L 类 5 项中 3 项已处理、2 项核验后保留（L-003 / L-005）。
**保留项如实留痕，未虚报为全部清零**，明细见「六」「七」「八」；T01~T05 五个任务源码改动全部收口，交付 QA 独立全量回归。

---

## 五、历史归档（旧清单 22 项）

> 旧清单检查日期 2026-07-29，最后更新 2026-08-20；22 项全部已修复，此处仅保留索引，详细修复方案见 git 历史中的本文件旧版本。

| 编号 | 问题摘要 | 状态 | 修复日期 |
|------|----------|------|----------|
| H-001 | FilterBar.vue 371 行超限（通用按钮样式重复） | ✅ 已修复 | 2026-07-29 |
| M-001 | software_info.rs SQL 行宽超限 | ✅ 已修复 | 2026-07-29 |
| M-002 | DataTable.vue CSS 行宽超限 | ✅ 已解决（文件已删除） | 2026-07-30 |
| M-003 | aur_info.rs SQL 行宽超限 | ✅ 已修复 | 2026-07-29 |
| M-004 | checker_type_id = 8 超出类型定义 | ✅ 已修复 | 2026-07-29 |
| M-005 | lib.rs 图标加载 unwrap 风险 | ✅ 已修复 | 2026-07-29 |
| M-006 | software_info.rs LIKE 通配符未转义 | ✅ 已修复 | 2026-07-29 |
| M-007 | useSoftwareForm.ts pkgname 校验缺失 | ✅ 已修复 | 2026-07-29 |
| M-008 | PackageList.vue 复杂表达式行宽 | ✅ 已修复 | 2026-07-29 |
| M-009 | DataTable.vue any 类型泛滥 | ✅ 已解决（文件已删除） | 2026-07-30 |
| M-010 | types/index.ts 298 行接近上限 | ✅ 已修复（拆 7 模块） | 2026-08-19 |
| L-001 | lib.rs 重复 eprintln! | ✅ 已修复 | 2026-07-29 |
| L-002 | lib.rs 静默丢弃窗口操作结果 | ✅ 已修复 | 2026-07-29 |
| L-003 | api_checker.rs 日志级别不当 | ✅ 已修复 | 2026-07-29 |
| L-004 | upstream.rs 静态正则 unwrap | ✅ 已修复 | 2026-07-29 |
| L-005 | git_version.rs 静态正则 unwrap | ✅ 已修复 | 2026-07-29 |
| L-006 | trait_def.rs 文档注释不准确 | ✅ 已修复 | 2026-07-29 |
| L-007 | useSoftwareForm.ts mode 类型不严格 | ✅ 已修复 | 2026-07-29 |
| L-008 | packageActions.ts 命名不一致（rowDelete） | ✅ 已修复 | 2026-08-20 |
| L-009 | software_info.rs 行映射重复 | ✅ 已修复 | 2026-07-29 |
| L-010 | api_checker.rs CheckResult 构造重复 4 处 | ✅ 已修复（**注意**：仅修此文件，另 7 处见本版 M-007） | 2026-08-20 |
| L-011 | aur_info.rs 冗余 COUNT 查询 | ✅ 已修复 | 2026-07-29 |

**重要更正**：旧清单 H-001（FilterBar.vue 超限）已随组件重构消除，当前该文件未超限；当前超限文件以本版「一、高风险」为准。

---

## 附录：检查标准参考

### 代码规模规范（见 AGENTS.md「文件组织原则」）
- 单个源文件 **≤ 300 行**（`wc -l`，含注释空行）
- 例外仅三类：生成物（需 `@LINE_LIMIT_EXEMPT` 标注）、纯数据表、测试文件
- 单行行宽 ≤ 100 字符（推荐）
- 当前全仓核验命令：
  ```bash
  find src src-tauri/src -type f \( -name '*.rs' -o -name '*.vue' -o -name '*.ts' \) \
    -print0 | xargs -0 wc -l | sort -rn | awk '$1>300'
  ```

### 命名规范（见 AGENTS.md「统一命名规范表」）
- Rust：函数/变量 `snake_case`，类型 `PascalCase`，常量 `UPPER_SNAKE_CASE`，测试 `test_*`
- Vue/TS：变量/函数 `camelCase`，类型/组件 `PascalCase`，composable 文件名 = `useXxx` = 导出名
- Tauri 命令：`snake_case` + 双宏；前端 bindings 自动转 `camelCase`

### 重复代码判定
- 同一模式 ≥ 3 处 → 必须抽公共方法
- ≥ 2 处且涉及安全 / 权限 / 命令拼装 / SQL → 必须抽公共方法
- 连续 ≥ 8 行相同 → 禁止复制粘贴

### IPC 规范（见 AGENTS.md「IPC 铁律」）
- 后端双宏 `#[tauri::command]` + `#[specta::specta]`（当前 88/88）
- 唯一注册点 `collect_commands!`（`lib.rs`）
- 前端 `invoke(` 必须为 0 处，全部走 `src/api/*.ts` 的 `commands.*`（当前 74 处）
- 改后端签名 → 重新生成 `src/bindings.ts` → `vue-tsc` 验证

### 安全规范
- 禁止硬编码敏感信息；SQL 必须参数化；输入必须验证；错误必须优雅降级
- 异步上下文禁止直接调用 `reqwest::blocking`（须 `spawn_blocking` 或 `std::thread::spawn` 隔离）

---

## 六、T03 治理落地记录与后续迁移计划

> 本节由 T03（Rust 系统命令 / sudoers / DB 锁 / HTTP client / GitHub URL 去重）回填。
> 原则：公共方法 + 旧副本禁止并存；安全语义（`-n` 位置、路径白名单、sudoers 逐 token、异步禁 blocking）逐字等价。

### 6.1 已落地公共方法

| 新建文件 | 公共方法 | 替换的调用点 |
|----------|----------|--------------|
| `sysops/system_cmd.rs` | `current_username()`（async） | `backup_install.rs:96`、`cache_install.rs:153`、`cache_cleanup.rs:206` 三处 `whoami` |
| `sysops/system_cmd.rs` | `pacman_package_info()`（async） | `backup_install.rs:46`、`cache_install.rs:56` 两处 `pacman -Qip` |
| `sysops/system_cmd.rs` | `install_package_locked()`（async） | `backup_install.rs:144`、`cache_install.rs:95` 两处 `sudo pacman -U`（保留 `with_pacman_write_lock`） |
| `sysops/sudoers.rs` | `build_sudoers_append_command()` | 上述三命令文件的 `grep -qF … printf … chmod 440` 模板 |
| `sysops/sudoers.rs` | `is_rule_effective` / `are_all_rules_effective` / `is_any_rule_effective` | 5 处 `sudo_privileges_text().await` + `has_nopasswd_rule` 组合探测 |
| `db/lock.rs` | `lock_db()`（经 `db/mod.rs` 再导出） | 上述三命令文件共 12 处 DB 锁获取 |
| `checkers/github/api_urls.rs` | 11 个 URL 构造函数 | 9 个文件共 15 处 `api.github.com` 硬编码 |
| `sysops/proxy_utils.rs` | 文件内私有 `gsettings_get()` | 同文件 3 处 `gsettings get` |

- `From<PoisonError<T>> for AppError` 在 `errors/db.rs` **本已存在且真实生效**（`state.db.lock()?` 已在用），本次未重复新增；`lock_db()` 直接以 `Ok(db.lock()?)` 复用该转换。
- `backup_install_helpers.rs` 保持 286 行未再塞入新能力；`sudo_privileges_text` / `has_nopasswd_rule` 仍是唯一的底层实现，`sudoers.rs` 仅复用不重写。
- `cache_cleanup.rs` 中真实存在的单测 `cleanup_args_match_sudoers_rule` 已保留并继续通过；`sudoers.rs` 新增 2 项追加命令单测（验证命令模板逐字一致 + 规则与 `CACHE_CLEANUP_ARGS[1..]` 逐 token 一致）。

### 6.2 M-006 剩余迁移计划（未一次性全改，登记在此）

已迁移 12 处（`backup_install.rs` 4 + `cache_install.rs` 4 + `cache_cleanup.rs` 4）。剩余：

| 剩余形态 | 数量 | 主要分布 |
|----------|------|----------|
| `map_err(\|e\| AppError::DatabaseError("获取数据库锁失败: …"))` | 23 | `fileops/cache_scan.rs` 4、`fileops/cache_backup/existing.rs` 4、`sysops/backup_basic.rs` 4、`fileops/cache_backup/subdirectory.rs` 3、`fileops/backup_scan.rs` 3、`fileops/backup_dedup.rs` 2、`sysops/upstream_validate.rs` 2、`fileops/scan.rs` 1 |
| `state.db.lock()?` 直连 | 61 | `commands/settings.rs`、`commands/proxy/basic.rs`、`commands/proxy/test.rs`、`commands/logs.rs`、`commands/enums.rs`、`sysops/software_check_single.rs` 等 12+ 文件 |

**迁移步骤建议**：
1. 按模块分批（每次 1~2 个文件），把上述写法统一替换为 `use crate::db::lock_db;` + `let db = lock_db(&state.db)?;`。
2. 每批后跑 `cargo check` / `cargo clippy --all-targets -- -D warnings` / `cargo test --lib`，并确认 `src/bindings.ts` diff 仍为空。
3. 注意行为差异：原 `map_err` 产生 `AppError::DatabaseError("获取数据库锁失败: …")`，`lock_db()` 统一为 `AppError::DatabaseLocked("数据库锁中毒: …")`。前端已核验无该字符串依赖，属预期的错误格式统一；若后续要求前端可读文案，应在 `errors/db.rs` 统一调整而非回退到散落 map_err。

### 6.3 M-012 HTTP client 核验结论（8 处全部保留，未合并）

逐个实地核验后，8 处 `Client::builder()` 均有不可无损替换的特殊策略，强行统一到 `http_client.rs` 会改变行为，故登记保留：

| 位置 | 特殊策略 | 保留理由 |
|------|----------|----------|
| `aur/metadata/download.rs:65` | UA 注入 + 归档专用 120s 长超时 + 15s 连接超时 | T01 刚拆出的 AUR 全量归档提速关键设计；并入 `shared_client()` 的 30s 默认超时会破坏大归档下载 |
| `http_client.rs:135` | `shared_client` 单例构建 | 公共模块自身实现，不是重复 |
| `http_client.rs:159` | `build_client(&ClientConfig)` 可配置实现 | 公共模块自身实现，不是重复 |
| `checkers/redirect.rs:42` | `Policy::none()` + 浏览器 UA | 需要读取重定向链而非跟随；`shared_client()` 默认跟随重定向 |
| `checkers/redirect.rs:60` | `Policy::none()` + 浏览器 UA + `danger_accept_invalid_certs(true)` | TLS 校验失败时的兜底探测，安全策略与公共客户端相反，禁止合并 |
| `proxy/test.rs:29` | `Policy::custom` + 自定义 `TEST_TIMEOUT` | 自定义 redirect policy 用于打印代理跳转链日志，公共 API 无等价能力 |
| `sysops/upstream_validate.rs:39` | 10s 超时、不走代理、无 UA 注入 | 面向任意域名的可达性验证，不应强制走代理或套用 GitHub 专属策略 |
| `sysops/proxy_utils.rs:160` | UA 注入 + 可选正向代理 + 可控 `follow_redirects` | 该文件已有 `build_client_with_policy()` 复用 `http_client.rs`，两个入口服务不同调用链，替换会改变代理/重定向语义 |

### 6.4 T03 验收证据

- `rg -n 'Command::new\("whoami"\)' src-tauri/src` → 仅 `system_cmd.rs` 1 处。
- `rg -n 'api\.github\.com' src-tauri/src` → 仅 `checkers/github/api_urls.rs`（常量 + 单测黄金值）。
- `rg -n 'grep -qF' src-tauri/src` → 仅 `sudoers.rs`（模板 + 单测）。
- `cargo check --lib` 通过（零 warning）；`cargo clippy --all-targets -- -D warnings` 当时记录为 2 项告警、放行后完全通过（**定性后经 QA 更正：`fe_log.rs:1` 系本轮新引入、已修复，仅 `trait_def.rs:33` 为真存量**，见第九节）。
- `cargo test --lib` → 145 passed / 0 failed（原 142 + 新增 3 项单测）。
- `cargo test --lib export_bindings` 后 `diff /tmp/bindings_baseline_t03.ts src/bindings.ts` 为空（1340 行不变）。
- 全仓 `.rs` 文件 `wc -l` 无 >300 行。

---

## 七、T04 治理落地记录与保留项说明

> 本节由 T04（Checker 结果构造 + 前端错误工具三件套）回填。原则：日志字面文本逐字等价、
> 特有字段显式传参不得被 `Default` 吞掉、语义差异大的构造不强抽、范围外问题只登记不修改。

### 7.1 后端 `checkers/helpers.rs`（新建，76 行）

```rust
pub(crate) enum MissingVersionLog { Debug, Silence, WarnUrl(String) }

pub(crate) fn finish_check(
    pkgname: &str,
    version: Option<String>,
    license: Option<String>,
    language_names: Vec<String>,
    missing_log: MissingVersionLog,
) -> CheckResult
```

- 有版本统一打 `info!("[版本检查] 检查完成: {} -> 上游版本={}", pkgname, v)`；
- 无版本按枚举变体打：`Debug` → 标准 `debug!(... 未找到上游版本)`、`Silence` → 不打、`WarnUrl(url)` → `warn!(... 未能从重定向 URL 提取版本: {})`；
- 返回**显式构造** `CheckResult { version, license, language_names }`，不使用 `..Default::default()`，
  保证 `tags_checker.rs` 的 `language_names` 不被吞掉（架构师风险提示）。

**收敛的 7 处**（`Ok(CheckResult {...})` + 成对日志）：

| 文件 | 原日志结构 | 传入变体 |
|------|------------|----------|
| `checkers/http.rs` | info + debug 成对 | `Debug` |
| `checkers/browser.rs` | info + debug 成对 | `Debug` |
| `checkers/github/tags_checker.rs:107` | info + debug 成对，`language_names` 显式 | `Debug` |
| `checkers/github/tags_checker.rs:128` | info + debug 成对，`language_names` 显式 | `Debug` |
| `checkers/gitee.rs` | **仅 info，无 else 分支** | `Silence` |
| `checkers/gitlab.rs` | **仅 info，无 else 分支** | `Silence` |
| `checkers/redirect.rs` | info + `warn!(... 未能从重定向 URL 提取版本: {url})` | `WarnUrl(upstream_url.to_string())` |

> 实证偏差：架构师清单称「7 处均为 info+debug 成对」，真实代码里 gitee/gitlab 只有 info、
> redirect 的 else 是带 URL 的 warn。故用 `MissingVersionLog` 显式承载差异，而不是一刀切统一成
> debug（那样会给 gitee/gitlab 新增日志、把 redirect 的 warn 改成 debug，违反逐字等价）。
> 已用 `rg -o '"\[版本检查\][^"]*"'` 对比 `helpers.rs` 与 `git show HEAD:` 原文，三条日志文案完全一致。

### 7.2 前端三件套（`utils/errors.ts` 新建 / `useToast.ts` 补充）

- `src/utils/errors.ts`：`NETWORK_ERROR_KEYWORDS`（6 个关键词，顺序与原实现一致）、
  `isNetworkError(e)`（`.some()` 短路或，等价于原 6 个 `||`）、`formatErrorMessage(e, context)`
  （两条模板字符串逐字等价）。
- `src/composables/useToast.ts` 新增 `showActionError(msg: string): void` → `addToast("error", msg)`。
  等价于原 `addMessage(footer, "error", msg)`：footer 只是 `addMessage` 的历史兼容占位参数，
  其内部即 `addToast(level, text)`，故不引入 footer 参数。
- 三个 composable 删除本地 `isNetworkError` / `formatErrorMessage` / `showError` 定义，
  改 `import { formatErrorMessage } from "@/utils/errors"` + `import { showActionError } from "./useToast"`，
  调用形如 `showActionError(formatErrorMessage(e, context))`。`isNetworkError` 仅由 `formatErrorMessage`
  内部使用，故 composable 不 import 它（tsconfig `noUnusedLocals: true`，多 import 会直接报错）。
- 已逐项 diff 三份原实现：三份**逐字相同**，无「以最全为准」的合并需求；关键词与两条模板逐字等价。
- import 清理：`packageCheckActions.ts` 去掉 `addMessage`（不再用），保留 `FOOTER_KEY`/`footer`（进度条）；
  `packageSyncActions.ts` 保留 `addMessage`（另有 `:136` 直接调用）；`packageDeleteActions.ts` 去掉
  `inject`/`FOOTER_KEY`/`addMessage`/`footer`（删除 `showError` 后全部无引用）。

### 7.3 `api_checker.rs` 保留未改（登记）

`checkers/github/api_checker.rs` 的完成日志共 6 处，其中 `:198` 带 ` (来自 tags)` 后缀，
其余分布在多个分支/多来源（release / tags 回退 / GraphQL 等），且构造点 `:38` 与日志并非
一一对应的成对结构。按 team-lead「特殊分支保留，不一刀切」要求，该文件**整体保留原样**，
未纳入 `finish_check`。若后续要收敛，需先按来源拆分支再评估。

### 7.4 M-008 `UpstreamCheckResult` 保留未合并（登记迁移计划）

实证核验与架构师清单偏差较大：

1. **`batch_helpers.rs:61` 不是 Builder**：它是 `pub(crate) async fn run_one(...) -> UpstreamCheckResult`，
   内部会 `get_checker` + `check_with_retry` 发起真实网络检查，并在 `:83`（成功）/`:97`（失败）
   两处构造结果。把它当构造函数复用会**重复发起网络请求**，明显改变行为。
2. **构造点不是 6 处，是 9 处**：`software_check/selected.rs:195`、`software_sync/upstream.rs:182`、
   `software_sync/cache.rs:87`、`software_sync/batch.rs:175` 与 `:219`、`batch_helpers.rs:83` 与 `:97`、
   `batch_engine.rs:104`（合计 9 处，另有 `utils.rs:61` 为定义）。
3. **字段语义差异大**：结构体 9 个字段，`batch.rs:219` 是 tags 回填占位
   （`pkgname: "__tags_fill__:..."`、`software_id: -1`、`_all_tags: Some(...)`、`_owner/_repo` 带真实值），
   其余点是正常检查结果映射；`license_spdx_id`/`language_names` 有的来自 `r.*`、有的来自
   `cache.snapshot`、有的为 `None`/`vec![]`。统一 builder 需要 9 个参数且仍要为占位分支特判，
   收益低、风险高。

**结论**：按 team-lead「若各构造点字段差异大、复用会改变行为，则不要强行合并，改为登记迁移计划」
保留全部 9 处。后续若推进，建议先新建纯构造函数
`build_upstream_result(pkgname, software_id, upstream_version, is_outdated, license, languages, tags_fill)`，
把 `_all_tags/_owner/_repo` 作为可选 `TagsFill` 参数，再分批替换并每批跑 clippy + 单测。

### 7.5 范围外发现（只登记，未修改）

- **`src/views/PackageList.vue` 当前 303 行 > 300**（HEAD 版 299 行，前序批次 +4 行）。
  它是 T02 / H-005 的目标文件，team-lead 明确要求 T04 不动，本批未触碰；请在 T02 或 T05 收尾时处理。
  除该文件和生成物 `src/bindings.ts`（1340 行，AGENTS.md 已登记豁免）外，
  `find src src-tauri/src -name '*.rs' -o -name '*.vue' -o -name '*.ts'` 无其它 >300 行文件。
- **`cargo fmt --all -- --check` 存在全仓历史格式存量**：约 36 个文件（主要集中在 `models/*` 的
  `use specta::Type` / `use serde::...` 排序，以及 `app_setup.rs`、`aur/rpc.rs` 等换行），
  另有 T03 引入的 `git_describe.rs` / `release_history.rs` 两处长行换行。
  本批已把 **T04 新增/改动的全部文件**修到 rustfmt 合规（`helpers.rs`、`checkers/mod.rs`、
  `tags_checker.rs`、`http.rs`、`gitee.rs`、`gitlab.rs`、`redirect.rs`、`browser.rs`），
  但按「不扩大改动范围」未执行全仓 `cargo fmt --all`（会改动 30+ 个范围外文件）。
  建议 T05 收尾时统一格式化一次并单独提交。
- `redirect.rs` 290 行、`browser.rs` 287 行、`packageSyncActions.ts` 274 行均 >270 但未 >300；
  本批在这三个文件里都是**净删除**逻辑，未新增，符合「>270 行禁止再新增逻辑」。

### 7.6 T04 验收证据

- `rg -n 'function (isNetworkError|formatErrorMessage)|const (isNetworkError|formatErrorMessage)' src/composables` → 无命中；
  定义仅存在于 `src/utils/errors.ts`，三 composable 仅剩 import 与调用。
- `git diff` 三 composable：错误提示上下文文案（`${pkgname} 上游检查`、`"批量删除"`、`"AUR 维护者同步"` 等）
  全部未变，仅 `showError(` → `showActionError(`；网络关键词与两条模板逐字等价。
- 后端完成日志：7 处收敛到 `helpers.rs`，三条日志字面文本与 `git show HEAD:` 原文逐字一致；
  api_checker 6 处保留原样。
- `cargo check`（全目标）通过；`cargo clippy --all-targets -- -D warnings` 当时记录为 2 项告警、放行后完全通过
  （**定性后经 QA 更正：`fe_log.rs:1` 系本轮新引入、已修复，仅 `trait_def.rs:33` 为真存量**，见第九节）；
  `cargo test --lib` → **147 passed / 0 failed**（T03 结束 145 + 本批 2 项新单测）。
- 本批新增 2 项单测守护 `language_names` 不被 `Default` 吞掉：
  `checkers::helpers::tests::test_finish_check_preserves_all_fields`、
  `checkers::helpers::tests::test_finish_check_missing_version_keeps_languages`（三种日志变体全覆盖）。
- `pnpm exec vue-tsc --noEmit` 通过（exit 0，无输出）；`pnpm build` 通过（`✓ built in 2.15s`）。
- `cargo test --lib export_bindings` 后 `diff /tmp/bindings_baseline_t04.ts src/bindings.ts` 为空（1340 行不变）。
- 本批新建/改动文件全部 ≤300 行（最大 `redirect.rs` 290 / `browser.rs` 287 / `packageSyncActions.ts` 274）；
  `cargo fmt --all -- --check` 对 T04 文件无 diff。

---

## 八、T02 + T05 治理落地记录与保留项

> 本节由收尾批次 T02（前端超限拆分）+ T05（命名统一 / 校验脚本 / 全仓格式）回填。
> 原则：以真实代码为准、不扩大改动范围、保留项如实留痕，方便后续 AI 继续。

### 8.1 阶段 A：T02 `PackageList.vue` 拆分

- `src/views/PackageList.vue` **303 → 247 行**（`wc -l`）。
- 新增两个纯展示组件，**均不 import 任何 composable / store**，props/emits 显式 TS 类型、无 `any`：
  - `src/components/package/PackageBatchToolbar.vue`（149 行）：props `loading / checkLoading / selectedCount`；
    emits `sync-aur / sync-by-maintainer / open-aur-import / sync-pkgbuild / add / update-aur /
    check-upstream / delete`。选中集合与具体 IPC 调用仍由父组件持有，组件只发语义事件。
  - `src/components/package/PackageListFilterToolbar.vue`（179 行）：props `conditionFilters /
    packageTypeOptions / checkerTypeOptions / quickFilterOptions / quickFilterKey /
    quickFilterLabel / viewMode`；emits `update-condition-filter / select-quick-filter /
    clear-quick-filter / set-view-mode`。条件下拉 `Event` 在组件内解析为 `number | null`，
    `parseSelectValue` 随区块迁入，父组件删除该副本（无重复实现）。
- 筛选类型（`QuickFilterKey` / `ViewMode` / `ConditionFilters`）以**等价结构类型内联声明**，
  不 import `composables/` 或 `stores/`，与阶段 B 的 4 个 `*Actions.ts` 重命名彻底解耦；
  仅 `import type { SelectOption } from "@/utils/enums"` 与 `QuickFilterDropdown.vue`、`@/icons`。
- **样式处理**：原 `PackageList.css` 为 `<style scoped src>`，插槽内容原本带父组件 scopeId；
  抽成子组件后内部元素改为子组件作用域，故两个新组件各自自带等价 `<style scoped>`
  （`.btn-icon` / `.btn-color-*` / `.toolbar-filter-group` / `.toolbar-divider` / `.btn-active`）。
  `.toolbar-filter-select` 由全局 `filter-styles.css`（`main.ts:24` 引入）提供，无需复制；
  `QuickFilterDropdown.vue` 自带 scoped 样式不受影响。批量按钮组保持**多根 fragment 平铺**，
  仍是 `.toolbar-left` 的直接 flex item，布局与拆分前一致。
- **高危链路未动**：`PackageTable` 的
  `@selection-change="(pkgnames: Set<string>) => setSelected(pkgnames)"`、
  `BlockView` 的 `@select` / `@select-all` / `@delete` 回传 `setSelected` 的各行**逐字保留**
  （`rg` 复核见 8.4）。AGENTS.md 记录的「勾选 13 个却全量更新」历史 bug 链路不受影响。
- 阶段 A 自检：`pnpm exec vue-tsc --noEmit` exit 0；`pnpm build` exit 0（`✓ built in 3.15s`）。

### 8.2 阶段 B：T05 各项落地

- **B1 命名统一**：4 个 composable 用 `git mv` 保留历史（git 状态 `RM`）：
  `packageActions.ts → usePackageActions.ts`、`packageCheckActions.ts → usePackageCheckActions.ts`、
  `packageDeleteActions.ts → usePackageDeleteActions.ts`、`packageSyncActions.ts → usePackageSyncActions.ts`。
  导出名本就是 `useXxx`，仅改文件名。引用同步更新：`src/views/PackageList.vue:8`、
  `usePackageActions.ts` 内 3 处 import + 1 处 re-export，以及 4 个文件头注释。
  改名前后各跑一次 `rg -n 'packageActions|packageCheckActions|packageDeleteActions|packageSyncActions' src`；
  **改名后零命中**（旧路径为小写 `packageActions`，新名大写 `usePackageActions`，不共享子串），
  另加负向后顾 `(?<!use)` 正则复核亦零命中，`vue-tsc` 兜底 exit 0。全仓其余命中均在历史文档
  （`AGENTS.md` / `docs/*` / `deliverables/*` / `OPTIMIZATION_SUMMARY.md`），属治理记录而非运行时引用，
  按「不扩大改动范围」保留。
- **B2 格式工具（实证偏差 → 保留未合并）**：`PackageAurInfoCard.vue:17` 与
  `PackageUpstreamInfoCard.vue:17` 的本地 `fmtTimestamp` **逐字相同**，但与 `src/utils/format.ts:90`
  的 `fmtTimestamp` **不等价**：

  | 实现 | 输出 | 空值判断 |
  |------|------|----------|
  | 两个 Card 本地版 | `toLocaleString("zh-CN", {year,month,day,hour,minute})`，**含时分** | `if (!ts) return "-"`，`0` 也返回 `"-"` |
  | `format.ts:90` `fmtTimestamp` | 定长 `"YYYY-MM-DD"`，**不含时分** | `if (ts == null) return "-"`，`0` 会格式化 |
  | `format.ts:17` `formatTimestamp` | `toLocaleDateString("zh-CN")` 本地化日期 | `if (!ts) return "—"`（em dash） |

  三者互不等价。按 team-lead「不等价则保留并说明差异、登记到清单，不要硬合并」——
  **两个本地实现保留**。`format.ts` 两个函数原有 JSDoc 已说明差异，本批为 `formatTimestamp`
  补一段与 `fmtTimestamp` 及 Card 本地版的对比说明（**仅注释，零逻辑改动**）；
  **不改名**，`fmtTimestamp → formatDateOnly` 波及 BlockView / PackageTable / SoftwareSideCards
  等 5+ 文件，收益低于风险，登记为后续建议。
- **B3 测试函数命名（实证偏差 → 已补齐新增）**：team-lead 假设「T03/T04 新增测试已是 `test_` 前缀」，
  实测发现 **T01 2 处 + T03 3 处共 5 处新增测试缺前缀**：
  `aur/metadata/tests.rs` 的 `archive_download_timeout_is_generous` / `archive_retry_params_are_bounded`、
  `commands/sysops/sudoers.rs` 的 `append_command_matches_rule_token_by_token` /
  `append_command_embeds_full_rule_and_username`、
  `checkers/github/api_urls.rs` 的 `urls_match_pre_extraction_literals`。
  这些文件均为 T01/T03 **新建**，属「本轮新增」，按「约束仅对新增生效」补齐 `test_` 前缀。
  **存量描述式命名一律不改**（宽口径 `rg` 实测 55 行匹配，team-lead 保守计数 19+ 处），
  避免为凑规范制造大范围纯改名 diff。T01~T04 相关 15 个测试现已全部 `test_` 前缀。
- **B4 校验脚本**：新建 `scripts/check-file-lines.sh`（已 `chmod +x`，82 行）。
  统计 `src/` + `src-tauri/src/` 下 `.rs/.vue/.ts`，排除 target/node_modules/dist/.cargo/.rustup；
  识别两类豁免：`src/bindings.ts`（生成物）与文件头 5 行内 `@LINE_LIMIT_EXEMPT` 标注
  （纯 bash 读取，避免管道 SIGPIPE）。实测输出仅
  `src/bindings.ts (1340 行) [生成物豁免]` + `OK: 仅 bindings.ts 豁免`，exit 0。
- **B5 全仓格式**：`cargo fmt --all` 执行 **1 次**（T04 登记的约 36 文件历史格式存量一并解决）。
  fmt 前 `cargo fmt --all -- --check` 报 48 段 diff；fmt 后 exit 0。
  **影响 34 个 `.rs` 文件**，范围集中在 `models/*` 的 `use serde` / `use specta::Type` import 排序，
  以及 `app_setup.rs` / `aur/rpc.rs` / `git_describe.rs` / `release_history.rs` /
  `software_sync/aur/update.rs` / `db/mod.rs` 等换行与长行折行。
  **纯格式证据**：① 对 34 个变化文件逐一比对 fmt 前后所有 `///` 文档注释文本，
  `DOC_CHANGED_COUNT=0`；② `cargo test --lib export_bindings` 后
  `diff /tmp/bindings_baseline_t05.ts src/bindings.ts` **为空**（1340 行不变）；
  ③ `cargo check` / clippy / `cargo test --lib` 全过。
  注意：rustfmt 重排 import 时**行尾 `//` 注释随行位置移动**（例如 `models/aur_info.rs`
  的 `// serde 序列化/反序列化支持` 从 `use serde` 行漂到 `use specta::Type` 行）。
  这是 rustfmt 已知行为，不影响语义、不涉及 `///` doc comment、不影响 bindings，
  本批按「不扩大改动范围」未手工还原，仅登记。

### 8.3 保留项清单（不虚报清零）

| 编号 | 保留内容 | 原因 | 后续建议 |
|------|----------|------|----------|
| M-006 | 剩余 23 处 `map_err` + 61 处 `state.db.lock()?` 直连 | 一次性全改风险高；`db::lock_db()` 与 `From<PoisonError<T>>` 已真实可用 | 分文件渐进迁移 |
| M-008 | 9 处 `UpstreamCheckResult` 构造 | 字段语义差异大、含 tags 回填占位分支，`:61` 是 `run_one` 非 builder | 先建纯构造函数，逐点替换 |
| M-012 | 8 处 `Client::builder()` | 各自有特殊 timeout / redirect / UA / 代理策略（含归档 120s 长超时） | 有统一需求时按策略分组 |
| T04 | `api_checker.rs` 完成日志 | 多分支 / 多来源，`:198` 带 ` (来自 tags)` 后缀，结构不一致 | 单独设计带分支语义的日志 helper |
| L-002 | 存量描述式测试命名 19+ 处 | 纯改名会制造大范围 diff，违反「不扩大改动范围」 | 约束仅对新增生效，存量不动 |
| L-003 | `fmtTimestamp` 命名歧义 | 改名波及 5+ 文件，收益低于风险 | 后续统一改 `formatDateOnly` 并留过渡别名 |
| L-005 | `as unknown as` 10 处 | 超出本收尾批次范围 | 专项治理 bindings 生成类型对齐 |
| — | 两个 Card 本地 `fmtTimestamp` | 与 `format.ts` 语义不等价（含时分） | 若需复用，另抽 `formatTimestampWithTime` |
| — | 1 项真存量 clippy 告警 | `trait_def.rs:33 unfulfilled_lint_expectations`（`#[expect(clippy::double_must_use)]` 因上游宏展开未触发，与 HEAD 逐字一致）；**`fe_log.rs:1 empty_line_after_outer_attr` 经 QA Round 1 证实为本轮新引入、已修复**（详见第九节） | 提交时 `-A unfulfilled_lint_expectations` 单独放行 1 项 |

### 8.4 T02 + T05 验收证据

| 验收项 | 结果 |
|--------|------|
| `bash scripts/check-file-lines.sh` | 仅 `src/bindings.ts` 一行 + `OK: 仅 bindings.ts 豁免`，exit 0；全仓非豁免文件无 >300 行 |
| 改名后 `rg -n 'packageActions\|...' src` | **零命中**（负向后顾复核亦零命中）；`vue-tsc --noEmit` exit 0 |
| `cargo fmt --all -- --check` | exit 0（全仓已格式化，34 文件受影响） |
| `cargo check` | exit 0，零 warning |
| `cargo clippy --all-targets -- -D warnings` | exit 101，当时记录为 2 项告警、放行后 exit 0；**定性后经 QA 更正：其中 `fe_log.rs:1` 系本轮新引入、已修复，现仅剩 1 项真存量**，见第九节 |
| `cargo test --lib` | **147 passed / 0 failed / 0 ignored** |
| `pnpm exec vue-tsc --noEmit` / `pnpm build` | exit 0 / exit 0（`✓ built in 3.32s`） |
| bindings 一致性 | fmt 后重取 `/tmp/bindings_baseline_t05.ts`（1340 行）→ `cargo test --lib export_bindings` → `diff` **为空** |
| 高危链路 | `rg -n 'selection-change\|setSelected(pkgnames)\|setSelected(new Set\|rowDeleteSelected' src/views/PackageList.vue` 命中 6 行，与拆分前逐字一致 |
| 行数 | `PackageList.vue` 247 / `PackageBatchToolbar.vue` 149 / `PackageListFilterToolbar.vue` 179 / `format.ts` 105 / `check-file-lines.sh` 82，全部 ≤300 |

**全局一致性审查（跨文件 import、props/emits 契约、数据流、无重复实现、无循环依赖）：IS_PASS: YES。**
`PackageList.vue` 的 8 个 `@` 事件与两个新组件 `defineEmits` 一一对应；`parseSelectValue` 仅存于
`PackageListFilterToolbar.vue`（父组件副本已删除）；4 个 `useXxx.ts` 互相 import 路径全部指向新名；
`src/bindings.ts` 保持 1340 行未手改。源码改动收口，交付 QA 独立全量回归。

---

## 九、QA 独立回归与最终订正（2026-11-25）

> 本章为**权威订正**：前八章各批次验收快照中的 clippy 告警定性以本章为准。QA 报告全文见
> `deliverables/engineering-assurance/QA-regression-2026-11-25.md`。

### 9.1 两轮回归结论
- **Round 1**（9 大类 32 条校验，30 通过 / 2 失败）：查出 2 项**本轮新引入**缺陷，路由 Engineer。
- **Round 2**（14/14 全通过）：修复复验通过，**最终路由 NoOne，允许交付**。

### 9.2 Round 1 缺陷与修复（已闭环）
| 编号 | 缺陷 | 定性 | 修复 |
|------|------|------|------|
| F-01 | `fe_log.rs:1 empty_line_after_outer_attr` | **非历史存量**：上一轮删 `use tauri::command;` 后文件头 `/** */`（outer doc attribute）后遗留空行所致，HEAD 不触发 | 保留 `/**`（因 specta 会把 doc comment 导出进 bindings，改 `/*` 会使 `bindings.ts` 1340→1324 行），仅删除 `*/` 与 `///` 之间的空行 |
| F-02 | `cargo check --release` 报 `unused import: std::path::PathBuf`（`ipc/registry.rs:16`） | 顶层 import 仅被 `#[cfg(any(debug_assertions, test))] export_bindings()` 使用，release 分支不编译；工程师自检只跑 debug 漏检 | 将 `use` 移入 `export_bindings()` 函数体内，release 零 warning |

### 9.3 关键流程订正：bindings 一致性校验
- `src/bindings.ts` 是 tauri-specta 生成物且**未纳入 git 跟踪**（`git status` 显示 `??`），
  因此 `git diff HEAD -- src/bindings.ts` 恒为空、属**假阴性**。
- 唯一合法闭环已写入 `AGENTS.md`「IPC 铁律」第 7 条：`cp 基线 → (cd src-tauri && cargo test --lib export_bindings) → diff 为空`。
- 命令数量订正：真实注册 **88** 条（此前文档「89」为误计，混入注释里的 `commands::` 字样）；
  registry 88 ↔ bindings 88 个 invoke ↔ 双宏各 88，四方一致。

### 9.4 最终验收（Round 2，QA 独立复验）
`cargo fmt --all -- --check` / `cargo check` / `cargo check --release`（零 warning）/
`cargo clippy --all-targets -- -D warnings`（仅 1 项真存量 `-A` 放行后 exit 0）/
`cargo test --lib`（**147 passed / 0 failed**）/ `pnpm exec vue-tsc --noEmit` / `pnpm build` /
`bash scripts/check-file-lines.sh`（仅 bindings.ts 豁免，exit 0）全部通过；
**bindings 逐字节一致**（1340 行，sha `602437bdfe5a5e58`），`src/bindings.ts` 仍 `??` 未手改、未 git add。

### 9.5 Known Issues（主理人裁定，不阻塞交付，留待后续专项）
| 编号 | 项 | 裁定 |
|------|----|------|
| K-1 | `trait_def.rs:33 unfulfilled_lint_expectations` | HEAD 真存量，提交时 `-A unfulfilled_lint_expectations` 放行，不擅自改 |
| K-2 | 各批次验收快照里的 clippy 旧定性 | 已在 8.3 / 8.4 / 第六 / 第七节就地订正并指向本章，历史事实保留 |
| K-3 | `AGENTS.md:160` 注册点路径滞后 | 已改为 `ipc/registry.rs::build_specta_router()` |
| K-4 | `AGENTS.md` 第 7 条代码块 `cd` 相对路径歧义 | 已用子 shell `(cd src-tauri && ...)` 消除 |
| K-5 | 工作区叠加前序未提交改动（方案 B tauri-specta 迁移、AUR 维护者同步、CSS 配色等，106 files） | **属本会话前序已验收工作，非本轮越界，不回退**；但仓库需一次正式提交，由用户决定，不擅自 commit |
| K-6 | 保留项（M-006/M-008/M-012、api_checker 日志、L-002/L-003/L-005、两个 Card 本地 fmtTimestamp、untracked 杂物） | 如实留痕、未虚报清零；`package.json.tmp` / `.SRCINFO` / `.Trash-0/` 等杂物不擅自删除，交用户处置 |

### 9.6 AUR 孤儿包登记（2026-11-26 实测，只登记不处理）

**背景**：用户 `zxp19821005` 反馈工具栏「从AUR同步」永远只同步本地已有数量。
排查确认为链路缺陷：旧 `sync_from_aur` 只取 `get_all_software()` 的包名去查询，且
`collect.rs` / `persist.rs` 只更新 `aur_info` 表（不新增 `software_info` 记录），
因此「AUR 有、本地缺」的包永远补不进来。该链路已于本次改为「一键全量对齐」。

**实测数据（2026-11-26）**：

| 项 | 数值 |
|----|------|
| AUR search `by=maintainer` resultcount | 1944 |
| AUR search `by=comaintainers` resultcount | 30 |
| 两次 search 重叠 | 0（并集 = 1974） |
| search 返回条数 vs resultcount | 相等（无分页截断） |
| 本地 `software_info` 条数 | **1873**（已复核 `~/.config/com.zxp19821005.aur-helper/my_aur_helper.db`：`SELECT COUNT(*)` = 1873） |
| AUR 有、本地缺 | **133** |
| 本地有、该用户名下已无（孤儿包） | **32**（= 1873 − (1974 − 133)） |

**孤儿包示例**（本地存在，且带用户配置的 `upstream_url`，佐证非脏数据）：

| 包名 | 本地 upstream_url |
|------|-------------------|
| `clash-verge-rev-bin` | `https://github.com/clash-verge-rev/clash-verge-rev` |
| `dingtalk-bin` | `https://dtapp-pub.dingtalk.com/dingtalk-desktop/xc_dingtalk_update/linux_deb/Release` |
| `electron4-bin` ~ `electron7-bin` | `https://github.com/electron/electron` |
| `115-browser-bin` | `https://appversion.115.com/1/web/1.0/api/chrome` |

**成因推测**：这些包疑似已转手 / 弃维护 / 改名，按 `by=maintainer` + `by=comaintainers`
两次 search 无法覆盖，故不属于「本地残留垃圾」，而是 AUR 侧归属变更的本地投影。

**裁定与处置**：
- **本次不删除、不处理**这 32 个包：它们仍可被上游版本检查 / 备份链路正常使用，
  删除会连带丢失用户已配置的 `upstream_url`。
- 升级版 `sync_from_aur` 只增不减（本地 ∪ 该用户名下维护），因此**不会**清理它们；
  二次点击结果稳定，不会越同步越少。
- 若确需清理，另立专项：逐包用 AUR info 确认 → 备份 `upstream_url` → dry-run 预览 → 再执行。

### 9.7 回归期新发现并当场修复的两项（2026-11-26）

| 编号 | 问题 | 处置 |
|----|------|------|
| K-7 | `src/checkers/trait_def.rs:33` 的 `#[expect(clippy::double_must_use)]` 在 clippy 1.96（rustc 1.96）下不再兑现，`-D unfulfilled-lint-expectations` 直接编译失败，阻断提交门禁。该文件不在本次改动范围内，属工具链升级后的存量漂移 | 改为 `#[allow(clippy::double_must_use)]` 并补注释说明「是否触发取决于 clippy / async-trait 版本组合」；`cargo clippy --all-targets -- -D warnings` 复跑通过 |
| K-8 | `AGENTS.md` IPC 铁律第 7 条仍写着「`src/bindings.ts` 未纳入 git 跟踪、禁止 `git add`」，与现状矛盾（现已 tracked，`git status` 显示 ` M`），会误导后续智能体 | 改写为「`git diff` 只作辅助参考，字节级 cp→重新生成→diff 闭环仍是唯一合法判据」，并注明该文件已纳入跟踪、须随代码一起提交 |
| K-9 | `aur_batch_size=0` 时 `slice::chunks(0)` 直接 panic：设置项可填 0，经 `parse_u32("0", 50)==0` → `.min(100)` → `get_packages_info` 分块崩溃；本次改动让 `sync_from_aur` 也走到该链路，故一并修 | 在唯一收口 `aur/rpc_batch/mod.rs::get_packages_info` 加 `batch_size.max(1)`；新增回归用例 `test_get_packages_info_batch_size_zero_does_not_panic`（不可达 DNS client 断言不 panic） |
| K-10 | 旧同步链路死代码：`aur/collect.rs`、`aur/persist.rs` 全仓零调用方，`software_sync/utils.rs::AurSyncResult` 亦仅被自身文档引用 | 用户裁定后**已删除**三处（`git rm` + 删结构体与头注释），并在 `docs/system_design.md` 加状态更新说明；删后 fmt / clippy / 153 测试 / bindings diff 全绿 |
