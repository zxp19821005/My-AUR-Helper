# 架构 / 代码 / 数据库优化方案（2026-08-20）

> 审查范围：`src-tauri/src`（Rust 后端）、`src`（Vue/TS 前端）、`src-tauri/src/db`（SQLite 层）
> 审查方法：通读关键模块 + 正则扫描 `unwrap()` / `let _ =` / `catch` / `.ok()` / `format!("SELECT` / `format!("INSERT` / 循环内查询
> 交付：本文件为「建议 + 实施记录」。标注 ✅ 为本次已落地；标注「建议」为本次评估后决定延后（架构风险 / 范围过大），附具体改造思路。

---

## 0. 前置修复：未提交代理重构导致编译失败（必须）

工作区存在一批未提交的「正向代理自动探测」重构（新增 `get_forward_proxy` / `detect_local_proxy`、统一 `build_client(timeout, use_proxy: bool)`），但调用方未同步更新，**库编译失败（7 错误）**。本次先行修复使其可编译：

| 文件 | 修复 |
|------|------|
| `checkers/proxy_utils.rs` 调用方 | `build_client(timeout, None)`（旧 `Option<&str>` 签名）→ `build_client(timeout, false)`；`check_selected_upstream` 新增 `github_client = build_client(timeout, true)` |
| `software_sync/batch.rs` | 新增参数 `github_client` 已接入：GraphQL 批量走 `github_client`、GitHub REST 回落分支也改用 `github_client`（消除 `unused variable` 警告） |
| `software_check.rs` | `sw.checker_type_id` 现已是 `CheckerType` 类型，`CheckerType::from_id(sw.checker_type_id)` 改为直接 `let checker_type = sw.checker_type_id;` 并复用 `&checker_type` 传给 `get_checker` |
| `software_sync/aur.rs` / `upstream.rs` | 同步 `build_client(timeout, false)` 并补齐 `github_client` 实参 |

> 设计意图梳理：非 GitHub 检查器（Gitee/GitLab/Redirect/Http）直连；仅 GitHub 走 `github_client`（启用正向代理，契合国内访问 GitHub 场景）。该意图本身合理，仅调用 wiring 未完成。

---

## 1. 架构（Architecture）

### A2 / D1：外键死代码 + 策略矛盾 — Medium — ✅ 已修复

**问题**：`db/connection.rs` 的 `ensure_no_fk_constraints` 查询 `pragma_foreign_key_list('software_info')` 判断是否要移除 FK，但 `software_info` 是**父表**（子表 `aur_info`/`upstream_info` 才有指向它的外键），该计数**恒为 0**，故 `rebuild_software_info_remove_fk` 永不执行；配套的 `fk_checked: Cell<bool>` 字段也无意义。同时 `migration_software.rs:74` 在 `software_info` 上查 `license_id` 外键也属于查错表。整体造成「既声称移除 FK、又 `PRAGMA foreign_keys=ON`」的语义混乱。

**修复**：
- 删除 `ensure_no_fk_constraints`、`rebuild_software_info_remove_fk`、`fk_checked` 字段及其在 `initialize()` 中的调用。
- 保留 `new()` 中的 `PRAGMA foreign_keys=ON`，与 `schema.rs` 中子表 `ON DELETE CASCADE` 形成一致、正确的引用完整性策略（删除软件包时级联清理 `aur_info`/`upstream_info`）。

**收益**：移除约 60 行死代码，消除语义矛盾，行为可预测。

### A1：前端缺少统一 API 抽象层 — Medium — ✅ 已实施

**问题**：`src/composables/*`、`src/stores/*`、`src/components/**` 直接散落 `invoke("command")`，同一命令多处重复（如 `delete_software` 出现在 3 处、`update_aur_info` 出现在 4 处）；store 与 composable 职责重叠（如 `stores/packages.ts` 与 `composables/usePackageList.ts` 都拉包列表）。

**实施**：新增 `src/api/` 目录，按领域拆分为 9 个模块（software/proxy/license/language/settings/backup/cache/dashboard/sudoers），集中封装全部 ~80 处 `invoke` 调用。所有 composable/store/component/view 不再直接 import `invoke`，统一通过 `api/*` 模块访问后端。`ProxyTestResult` 类型从 composable 迁移到 `types/proxy.ts`。`useSudoers` 重构为接受 api 回调函数（消除动态命令名调度）。`vue-tsc --noEmit` + `vite build` 验证 0 error。

### A3：版本比对 + 写入逻辑重复 — Medium — ✅ 已修复

**问题**：`compare_and_update`（单包路径）与 `check_selected_upstream` 内联的写库逻辑实现了同一套「is_outdated 计算 → update_software_outdated → update_software_languages → upsert_upstream_info」，容易漂移。

**修复**：抽取 `apply_upstream_check_result(db, software_id, cleaned_version, is_outdated, upstream_license_id, language_ids, fill_languages)`，两处共用，单一写入口径。

---

## 2. 代码质量（Code Quality）

### C2：批量检查时循环内逐包 `get_aur_info`（N+1 残留）— Medium — ✅ 已修复

**问题**：`check_selected_upstream` 在构建任务前用一个 `for` 循环对每个包调用 `db.get_aur_info(...)`（N 次查询）仅用于判断「有无 AUR 版本」，而紧接着 L308 又一次性 `get_aur_versions_map()` 全量读取 AUR 版本用于比较——前面 N 次查询完全冗余（正是既往审计声称已消除的 N+1 的残留）。

**修复**：把 `get_aur_versions_map()` 的批量读取**提前到任务构建循环之前**，循环内直接用内存中的 `aur_map` 判断 `has_aur`，删除循环内逐包查询。净效果：构建过滤与版本比较共用同一份批量读取，零逐包查询。

### C1：批量写库未包裹事务（数据一致性）— High — ✅ 已实施

**问题**：`check_selected_upstream` 对每个包执行 3 次写（`update_software_outdated` + `update_software_languages` + `upsert_upstream_info`），跨 N 个包循环；`compare_and_update` 同理。中途崩溃会留下部分写入 / 孤儿行。全仓无任何 `transaction()` 使用。

**原阻碍**：项目现有写库方法（`update_software_outdated` 等）签名均为 `&self` 且内部用 `self.conn`；而 rusqlite 的 `Connection::transaction()` 需要 `&mut self`，事务 `Transaction` 与 `&self` 方法无法在同一连接上共存（借阅冲突）。

**改造方案（已落地）**：将写库方法拆分为「接收 `&rusqlite::Connection` 的低层执行函数」+「`&self` 便捷封装」：
- `db/software_info.rs`：新增 `update_software_outdated_conn(conn, ...)` / `update_software_languages_conn(conn, ...)`，`update_software_outdated` / `update_software_languages` 改为委托 `Self::*_conn`。
- `db/upstream_info.rs`：新增 `upsert_upstream_info_conn(conn, ...)`，`upsert_upstream_info` 委托 `Self::upsert_upstream_info_conn`。
- `commands/sysops/software_check.rs`：
  - `apply_upstream_check_result` 改为接收 `&rusqlite::Connection`（事务兼容）。
  - `compare_and_update` 接收 `&mut Database`，单包三段写入包裹于 `db.conn.transaction()`。
  - `check_selected_upstream` 批量写库改为：事务外**预解析**各结果语言 ID（`resolve_language_ids` 需写库，避免与 `&mut db` 借阅冲突）→ 开启 `let mut db = state.db.lock()?; let tx = db.conn.transaction()?;` → 循环内以 `&tx` 调用 `apply_upstream_check_result` / `update_software_outdated_conn` → 末尾 `tx.commit()?;`。

**效果**：批量检查的全部写库操作原子化（全有或全无），任一失败整体回滚；单包检查同样事务化。借贷冲突通过「低层 `&Connection` 函数 + `&self` 委托」模式解决。验证：`cargo check --lib` / `cargo clippy --lib` / `cargo test --lib`（63 passed）均通过。

### C4：HTTP 超时 / 重试解析重复 — Low — ✅ 已实施

**问题**：`check_upstream_version` 与 `check_selected_upstream` 都重复 `parse_u64(get_setting_opt(...).unwrap_or_default(), 30)` / `parse_u32(..., 2)`，共 6 处散落在 `software_check.rs`(2)、`upstream.rs`(1)、`aur.rs`(2)、`enums.rs`(1)。

**实施**：在 `software_sync::utils` 抽取两个函数：
- `read_http_timeout(db) -> u64`：读取 `http_timeout` 设置，解析失败回退 30。
- `read_http_settings(db) -> (timeout: u64, retry: u32)`：一次性读取 `http_timeout` + `http_retry_count`，回退 30 / 2。
6 处调用方统一替换。`enums.rs` 移除本地 `get_setting_opt` / `parse_u64` 副本，改用共享函数。

**效果**：HTTP 超时/重试配置解析逻辑单一来源，新增配置项只需改一处。

### C3：日志写入失败被静默吞掉 — Low — ✅ 已实施

**问题**：`logger.rs` 的 `writeln!/flush` 用 `let _ =` 吞写失败，文件写入异常时无任何可见反馈，排障困难。

**实施**：`logger.rs` 三处 `let _ = writeln!(...)` / `let _ = file.flush()` 改为失败时 `eprintln!` 兜底输出到 stderr，确保文件写入失败至少在终端可见。`proxy_utils.rs` 的 `addr.parse().unwrap()` 属静态地址安全场景，沿用既有加固。

---

## 3. 数据库（Database）

### D3：仪表盘 8 次独立 COUNT 查询 — Low — ✅ 已修复

**问题**：`db/stats.rs` 每次仪表盘加载发 8 条独立 `COUNT(*)` 查询（software_info / outdated / backup / cache / proxies / proxies_active / licenses / languages）。

**修复**：合并为单条 SQL，用 8 个标量子查询一次性返回所有计数，仅一次 DB 往返：
```sql
SELECT
  (SELECT COUNT(*) FROM software_info),
  (SELECT COUNT(*) FROM software_info WHERE is_outdated = 1),
  (SELECT COUNT(*) FROM backup_software),
  (SELECT COUNT(*) FROM cache_software),
  (SELECT COUNT(*) FROM proxies_info),
  (SELECT COUNT(*) FROM proxies_info WHERE is_active = 1),
  (SELECT COUNT(*) FROM enum_licenses),
  (SELECT COUNT(*) FROM enum_programming_languages)
```
删除原 `count(conn, sql)` 辅助函数及 `use rusqlite::Connection`。语义不变，往返次数 8→1。

### D2：software_info 重复 SQL / 行映射 — Low/Medium — ✅ 已实施

**问题**：`get_software_detail_by_name` 与 `get_software_list_entry` 是几乎相同的三表 JOIN SELECT；`row_to_software_info` 与 `row_to_list_entry` 对 `software_info` 列的映射重复，列序错位有数据错位先例。

**实施**：在 `db/software_info.rs` 抽取 3 个共享 SQL 列清单常量：
- `SW_INFO_COLS`：单表 SELECT 列清单（11 列，无前缀）
- `SW_INFO_COLS_S`：多表 JOIN 列清单（`s.` 前缀）
- `SW_LIST_COLS`：列表视图 JOIN 列清单（software_info + aur_info + upstream_info）

`get_software_detail_by_name` 复用 `row_to_software_info` 映射前 11 列（0-10），仅手动读取 aur/upstream 扩展列（11+），消除重复行映射。所有 SQL 查询改用 `format!("SELECT {SW_INFO_COLS} ...")` 引用常量，修改列序只需改一处。

**效果**：列名单一来源，消除映射漂移风险。验证：`cargo test --lib` 63 passed（含 schema 一致性单测）。

### D4：高频过滤列索引核查 — Low — 无动作

现有索引已覆盖 `pkgname`、`is_outdated`、`filename`、`name`、`proxy_id`、`category`；`search_software` 的 `upstream_url LIKE` 因前导通配符无法用索引（可接受），`full_path` 无 WHERE 查询无需索引。当前无明确缺索引。

---

## 4. 验证结果

| 检查项 | 结果 |
|--------|------|
| `cargo check --lib` | ✅ 0 error / 0 warning |
| `cargo clippy --lib` | ✅ 0 error / **0 warning**（32→0，全部清零） |
| `cargo test --lib` | ✅ 63 passed / 0 failed |
| `vue-tsc --noEmit` | ✅ 0 error |
| `vite build` | ✅ 通过 |

---

## 5. 本次改动文件清单

### 前置修复（编译通过）

| 文件 | 改动 |
|------|------|
| `src-tauri/src/db/connection.rs` | 移除 FK 死代码（`ensure_no_fk_constraints` / `rebuild_software_info_remove_fk` / `fk_checked`） |
| `src-tauri/src/db/stats.rs` | 仪表盘 8 次 COUNT 合并为单条多子查询 |
| `src-tauri/src/commands/sysops/software_check.rs` | CheckerType 类型修正；抽取 `apply_upstream_check_result`；移除 N+1 逐包查询；补齐 `github_client` |
| `src-tauri/src/commands/sysops/software_sync/batch.rs` | 接入 `github_client`（GraphQL + GitHub REST 回落） |
| `src-tauri/src/commands/sysops/software_sync/aur.rs` | `build_client` 新签名适配 |
| `src-tauri/src/commands/sysops/software_sync/upstream.rs` | `build_client` 新签名 + 补齐 `github_client` |
| `src-tauri/src/commands/sysops/proxy_utils.rs`、`enums.rs`、`models/checker_type.rs` 等 | 既有未提交 WIP（正向代理自动探测），本次仅修复其编译接线，未改动其业务逻辑 |

### C1：批量写库事务化

| 文件 | 改动 |
|------|------|
| `src-tauri/src/db/software_info.rs` | 新增 `update_software_outdated_conn` / `update_software_languages_conn`（接收 `&Connection`），原 `&self` 方法委托调用 |
| `src-tauri/src/db/upstream_info.rs` | 新增 `upsert_upstream_info_conn`（接收 `&Connection`），原 `&self` 方法委托调用 |
| `src-tauri/src/commands/sysops/software_check.rs` | `apply_upstream_check_result` 改为接收 `&Connection`；`compare_and_update` 接收 `&mut Database` 包裹事务；`check_selected_upstream` 批量路径改为预解析语言 ID → 开事务 → 循环 `&tx` 写库 → `tx.commit()` |

### A1：前端统一 API 抽象层

| 文件 | 改动 |
|------|------|
| `src/api/software.ts` | 新建：软件包领域 API（CRUD/AUR 同步/上游检查/URL 校验） |
| `src/api/proxy.ts` | 新建：代理领域 API |
| `src/api/license.ts` | 新建：License 领域 API |
| `src/api/language.ts` | 新建：编程语言领域 API |
| `src/api/settings.ts` | 新建：设置与日志领域 API |
| `src/api/backup.ts` | 新建：备份领域 API |
| `src/api/cache.ts` | 新建：缓存领域 API |
| `src/api/dashboard.ts` | 新建：仪表盘领域 API |
| `src/api/sudoers.ts` | 新建：sudoers 免密配置领域 API |
| `src/types/proxy.ts` | 新增 `ProxyTestResult` 接口（从 `useProxyList.ts` 迁移） |
| `src/composables/useSudoers.ts` | 重构为接受 api 回调函数（消除动态命令名调度） |
| `src/composables/*.ts`、`src/stores/*.ts`、`src/components/**/*.vue`、`src/views/*.vue`（30 个文件） | 移除 `import { invoke }`，改用 `import * as xxxApi from "@/api/xxx"` |

### C4：HTTP 超时 / 重试解析去重

| 文件 | 改动 |
|------|------|
| `src-tauri/src/commands/sysops/software_sync/utils.rs` | 新增 `read_http_timeout` / `read_http_settings` 共享函数 |
| `src-tauri/src/commands/sysops/software_check.rs` | 2 处重复调用替换为 `read_http_settings` |
| `src-tauri/src/commands/sysops/software_sync/upstream.rs` | 1 处替换为 `read_http_settings` |
| `src-tauri/src/commands/sysops/software_sync/aur.rs` | 2 处替换为 `read_http_timeout` |
| `src-tauri/src/commands/enums.rs` | 移除本地 `get_setting_opt`/`parse_u64` 副本，改用共享函数 |

### C3：logger 写入失败兜底

| 文件 | 改动 |
|------|------|
| `src-tauri/src/logger.rs` | 3 处 `let _ = writeln!/flush` 改为失败时 `eprintln!` 兜底 |

### D2：software_info SQL 列清单去重

| 文件 | 改动 |
|------|------|
| `src-tauri/src/db/software_info.rs` | 抽取 `SW_INFO_COLS` / `SW_INFO_COLS_S` / `SW_LIST_COLS` 常量；`get_software_detail_by_name` 复用 `row_to_software_info` 映射前 11 列 |

### L-008 / L-010 + clippy warning 清零（32→0）

| 文件 | 改动 |
|------|------|
| `src-tauri/src/checkers/github/api_checker.rs` | L-010：提取 `build_result` 辅助函数消除 4 处重复 CheckResult 构造；`/** */`→`//!` 模块文档 |
| `src/composables/packageActions.ts` | L-008：`rowDelete` → `rowDeleteSelected`，与 `deleteSelected` 命名一致 |
| `src/views/PackageList.vue` | 适配 `rowDeleteSelected` 重命名 |
| `src-tauri/src/checkers/github/release_history.rs` | 提取 `ReleaseScanParams<'a>` struct，`check_github_releases` 从 8 参数降为 2（`too_many_arguments` 消除） |
| `src-tauri/src/checkers/github/release.rs` | 2 处调用适配 `ReleaseScanParams` |
| `src-tauri/src/checkers/github/api_checker.rs` | 1 处调用适配 `ReleaseScanParams` |
| `src-tauri/src/commands/fileops/backup_dedup.rs` | 提取 `BackupPkgEntry` struct + `PkgBackupMap` type alias 替代 tuple，消除 `type_complexity` 告警 |
| `src-tauri/src/commands/software.rs` | `add_software`(10 参数) / `update_software`(12 参数) 加 `#[allow(clippy::too_many_arguments)]`（Tauri IPC 契约约束，改 struct 会破坏前端扁平传参一致性） |
| `src-tauri/src/checkers/github/mod.rs` 等 7 个 mod.rs | `/** */` 块注释 → `//!` 模块级文档（消除 `empty_line_after_doc_comment` + `doc_lazy_continuation`） |
| `src-tauri/src/checkers/github/repo_info.rs` | `sort_by` → `sort_by_key(std::cmp::Reverse)` |
| `src-tauri/src/models/upstream_info.rs` | `from_str` → `parse_from_str`（避免与 `FromStr` trait 混淆） |
| `src-tauri/src/proxy/test.rs` | 独立 `///` 改为 `//` 普通注释 |
| `cargo clippy --fix` 自动修复 | 12 个 `needless_borrow` / `redundant_closure` / `to_string` 等 |

### 编译性能优化：移除 rusqlite bundled + profile.dev

**问题**：`tauri dev` 编译耗时 2m23s，且系统内存压力大。根因：rusqlite `bundled` feature 每次编译 SQLite C 源码（sccache 不覆盖 C 编译），且无 `[profile.dev]` 配置导致链接阶段生成完整 DWARF 调试信息。

| 文件 | 改动 |
|------|------|
| `src-tauri/Cargo.toml` | 移除 rusqlite `bundled` feature，改用系统 SQLite（Arch Linux 自带 3.53.4）；追加 `[profile.dev] debug = 1` 仅保留行号调试信息 |

**验证结果**：

| 检查项 | 结果 |
|--------|------|
| `cargo check --lib` | ✅ 通过（不再编译 libsqlite3-sys C 源码） |
| `cargo clippy --lib` | ✅ 0 warning（11.81s） |
| `cargo test --lib` | ✅ 63 passed; 0 failed |
| `pnpm run build`（vite） | ✅ 构建成功（2.10s） |

> **注意**：移除 `bundled` 后首次触发全量重编（~10min，因 feature + profile 变更），之后增量编译省去 SQLite C 编译开销。`debug = 1` 对 515 crate 的 debug 链接有显著加速。如需完整调试信息可临时设 `debug = 2`。

### 路由注释更新

| 文件 | 改动 |
|------|------|
| `src/router/index.ts` | 移除对已删除 `preloadRoutes()` 的过时引用；更新注释说明 dev 模式不做预取的原因 |

### WebKitGTK 2.52.6 DMABUF 渲染回归（外部依赖问题，2026-08-20 下午）

**问题**：webkit2gtk-4.1 升级 2.52.5-2 → 2.52.6-1 后，应用所有页面卡顿（切换 15-37s）、WebKitWebProcess CPU 占用高、温度高。Chromium 系浏览器不受影响（不同引擎）。

**根因**：2.52.6 的 DMABUF 硬件加速渲染路径在 Intel i915 + X11 环境回归。

**应急修复**：`scripts/dev.sh` 中 `export WEBKIT_DISABLE_DMABUF_RENDERER=1`（禁用 DMABUF，回退软件渲染）。验证：页面切换降至 0.1-2.4s。上游修复后删除该行。

**彻底方案**：`sudo pacman -U https://archive.archlinux.org/packages/w/webkit2gtk-4.1/webkit2gtk-4.1-2.52.5-2-x86_64.pkg.tar.zst` 降级恢复硬件加速，并在 `/etc/pacman.conf` 加 `IgnorePkg = webkit2gtk-4.1` 防自动升级；建议向上游（bugs.webkit.org / Arch）报 bug。
