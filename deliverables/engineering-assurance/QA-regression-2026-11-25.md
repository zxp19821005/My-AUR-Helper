# QA 独立全量回归验证报告（阶段 3）

> 验证人：Edward（QA 工程师，独立验证方）
> 日期：2026-11-25
> 仓库：`/run/media/zxp/LocalBak/git/My-AUR-Helper`（Tauri v2 + Vue3/TS + Rust）
> 方式：**fresh eyes 独立执行**，下列命令全部亲自运行、逐条取证，未复用工程师自检结论
> 环境：`export HOME=/home/zxp-archlinux CARGO_HOME=<repo>/.cargo RUSTUP_HOME=<repo>/.rustup`；Bash 关闭 sandbox；检索统一用 `rg --glob '!**/target/**'`
> 轮次：**Round 1**（上限 2 轮）

---

## 一、Summary

| 项 | 结果 |
|---|---|
| 验收大类 | 9 大类（任务清单 1~9），细化为 **32 条具体校验** |
| 通过 | **30 条** |
| 失败（源码缺陷，非阻断） | **2 条** —— 均为本轮新引入的 lint 告警 |
| 需 team-lead 确认（非源码 bug） | **1 组 5 项**：bindings 基线、改动范围、文档计数、设计签名偏差、untracked 杂物 |
| QA 自身判定失误并自修正 | 1 次（clippy 放行用了 deprecated lint 名，已用正确 lint 名重跑通过） |
| **路由判定（Round 1）** | **Engineer**（2 项新引入告警，附精确修复建议，请主理人转回工程师）+ **team-lead 确认**（范围/基线问题） |
| **最终路由（Round 2 后）** | ✅ **NoOne** —— F-01/F-02 已修复且独立复验全部通过、无新增回归，**可交付** |

> **【Round 2 最终判定】** 2026-11-25 第二轮（最后一轮）复验：两项缺陷已由工程师修复并经 fresh eyes 独立验证生效，全部验证项通过，**最终路由 = NoOne**。详见文末「Round 2 最终回归验证」章节。本节 Round 1 的 Failed 明细与范围问题保留作为问题溯源记录，不再作为打回依据。

**一句话结论（Round 1 时点）**：这次大规模治理重构在**编译、类型、IPC、单测、前端构建、行为等价、行数、重复收敛、安全红线**层面均经受住了独立验证，**功能上是安全的**；但**不能判定为"零瑕疵通过"**——存在 2 项本轮新引入的 lint 告警（其中 1 项被工程师错误归为"历史存量"），按 AGENTS.md「提交前 `cargo clippy --all-targets -- -D warnings` 任一失败不得提交」的硬约束，当前代码**不满足提交条件**；同时「bindings.ts 相对 HEAD 零变化」这一核心验收判据因生成物从未入库而**语义失效**，需修复基线管理。

---

## 二、逐条验证结果

> 所有日志保存在 `/tmp/qa_logs/`（A1~A9b / C1~C4），可复核。

### 1. Rust 编译（全目标 + release）

| 校验 | 命令 | 结论 | 关键输出摘录 |
|---|---|---|---|
| 1.1 | `cargo fmt --all -- --check` | ✅ exit 0 | `A1_FMT_EXIT=0`（无输出） |
| 1.2 | `cargo check --all-targets` | ✅ exit 0 | `Finished dev profile ... in 5.48s`，零 warning |
| 1.3 | `cargo check` | ✅ exit 0 | `Finished ... in 0.16s` |
| 1.4 | `cargo check --release` | ⚠️ exit 0（**零 error，但 1 个 warning**） | `warning: unused import: std::path::PathBuf --> src/ipc/registry.rs:16:5`；`my-aur-helper (lib) generated 1 warning`；`Finished release ... in 1m 16s` |

→ 1.4 见「Failed 明细 F-02」。任务要求的"零 error"满足，但 release 构建不干净，工程师自检（仅 debug）遗漏。

### 2. 格式

`cargo fmt --all -- --check` → **exit 0**。本轮做过全仓 `cargo fmt --all`（34 文件纯格式），独立复核确认工作区已完全 rustfmt 合规。✅

### 3. lint（含 2 项历史告警的独立核实）

| 校验 | 命令 | 结论 | 关键输出摘录 |
|---|---|---|---|
| 3.1 | `cargo clippy --all-targets -- -D warnings`（严格） | ⚠️ exit 101，**恰好 2 项** | `error: empty line after outer attribute --> src/commands/fe_log.rs:1:1`；`error: this lint expectation is unfulfilled --> src/checkers/trait_def.rs:33:10`；`could not compile ... due to 2 previous errors` |
| 3.2 | 核实 `trait_def.rs:33` 是否 HEAD 存量 | ✅ **确属存量** | `git diff HEAD -- src-tauri/src/checkers/trait_def.rs` **为空**；`#[expect(clippy::double_must_use)]` 与 HEAD 逐字一致（async-trait 宏展开注释亦一致） |
| 3.3 | 核实 `fe_log.rs:1` 是否 HEAD 存量 | ❌ **不是存量，本轮新引入** | 见 F-01：`git show HEAD:...` 第 15 行为 `use tauri::command;`，工作区该行被删后留下空行 |
| 3.4 | `-D warnings -A clippy::empty_line_after_outer_attr -A unfulfilled_lint_expectations` | ✅ **exit 0** | `Finished dev profile ... in 2.95s`，零告警 → 除上述 2 项外**无新增告警** |

> **QA-self 修正记录**：首次放行用 `-A empty_line_after_outer_attr`（deprecated lint 名），在 `-D warnings` 下触发 `renamed_and_removed_lints` 而 exit 101。这是我的**验证脚本用法错误**，非源码问题；改用 `clippy::` 前缀后 exit 0，已自行修正重跑，不计入源码缺陷。

### 4. 单测全量 + 单测有效性抽查

| 校验 | 命令 | 结论 | 关键输出摘录 |
|---|---|---|---|
| 4.1 | `cargo test --lib` | ✅ | `running 147 tests` → `test result: ok. 147 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s`（与工程师声称一致，独立复现） |
| 4.2 | `cargo test --lib export_bindings` | ✅ | `running 1 test` → `specta_bindings_tests::export_bindings ... ok` |
| 4.3 | 新公共方法单测是否真实运行 | ✅ | A5 日志中确实出现并 ok：`api_urls::tests::test_urls_match_pre_extraction_literals`、`helpers::tests::test_finish_check_preserves_all_fields`、`helpers::tests::test_finish_check_missing_version_keeps_languages`、`sudoers::tests::test_append_command_matches_rule_token_by_token`、`sudoers::tests::test_append_command_embeds_full_rule_and_username`、`cache_cleanup::tests::cleanup_args_match_sudoers_rule` |
| 4.4 | 读测试体确认非空跑 | ✅ | `sudoers.rs`：`assert_eq!(cmd, expected)`（expected 为独立复制的模板字面量）+ `assert_eq!(CACHE_CLEANUP_ARGS[1..].join(" "), CACHE_CLEANUP_RULE)` 逐 token 一致性 + `assert_eq!(CACHE_CLEANUP_ARGS[0], "-n")`；`api_urls.rs`：11 个**硬编码 URL 字面量** `assert_eq!` 黄金值；`helpers.rs`：断言 `version/license/language_names` 三字段原样带出，覆盖 Debug/Silence/WarnUrl 三变体。均为真实断言，非 `assert!(true)` 空跑 |

> 未做"临时破坏断言"变异测试（避免污染源工作区）；通过「测试体逐行审阅 + 运行日志确认 + 硬编码黄金值/字段断言」三重证据判定有效。

### 5. IPC 一致性（最关键项）

| 校验 | 命令 | 结论 | 关键输出摘录 |
|---|---|---|---|
| 5.1 | `git diff HEAD -- src/bindings.ts \| wc -l` | ⚠️ **0，但为假阴性** | `git ls-files --error-unmatch src/bindings.ts` → `错误：路径规格 'src/bindings.ts' 未匹配任何 Git 已知文件`；`git ls-tree -r HEAD` 无该文件；`git log --all -- src/bindings.ts` 空；`git check-ignore` 未忽略（仅从未 `git add`）→ **生成物从未入库，diff 恒为空，该判据无效**。详见 P-01 |
| 5.2 | **真·字节级一致性**：保存基线 → 重生成 → `diff` | ✅ | 基线 `1340 行 sha=602437bdfe5a5e58` → `cargo test --lib export_bindings` → `A7_BINDINGS_DIFF_EXIT=0`、`A7_DIFF_LINES=0`、重生成 `1340 行 sha=602437bdfe5a5e58`，**逐字节完全一致** |
| 5.3 | `rg -o 'commands::' src-tauri/src/ipc/registry.rs \| wc -l` | ✅ 89 命中 | 89 = **88 条真实注册**（`collect_commands![...]` 第 47~134 行）+ 1 处注释（第 20 行 `// collect_commands! 中的 \`commands::...\` ...`）；无重复路径 |
| 5.4 | 三方一致性：registry ↔ bindings ↔ 真宏 | ✅ **无遗漏** | `rg -n '^#\[tauri::command\]'` = **88**、`^#\[specta::specta\]` = **88**（各另 2 处注释提及，合计 90 命中）；`registry_fn=88`、`bindings_fn=88`，`comm -3` **无差异**；`bindings.ts` 中 `__TAURI_INVOKE` 88 处调用（89 命中含第 3 行 import 别名） |
| 5.5 | 前端 `invoke(` 零残留 | ✅ | `rg -o '\binvoke\(' src --glob '!**/bindings.ts' \| wc -l` = **0** |
| 5.6 | `src/api/*.ts` 全走 `commands.*` | ✅ | 10 个文件逐文件：backup 8 / cache 12 / dashboard 1 / language 3 / license 5 / proxy 9 / settings 6 / software 21 / sudoers 6 / update 3，**全部 invoke=0** |
| 5.7 | `collect_commands!` 唯一注册点 | ✅ | 7 处命中中仅 `registry.rs:46` 是宏调用，其余 6 处为注释/文档提及；`use crate::commands;` 显式引入保持路径解析 |

### 6. 前端

| 校验 | 命令 | 结论 | 关键输出摘录 |
|---|---|---|---|
| 6.1 | `pnpm exec vue-tsc --noEmit` | ✅ exit 0 | 无任何输出 |
| 6.2 | `pnpm build` | ✅ exit 0 | `✓ built in 2.42s`，`dist/assets/bindings-*.js` 等产物正常生成 |

### 7. 300 行硬上限

| 校验 | 命令 | 结论 | 关键输出摘录 |
|---|---|---|---|
| 7.1 | `bash scripts/check-file-lines.sh` | ✅ exit 0 | 仅 `src/bindings.ts (1340 行) [生成物豁免]` + `OK: 仅 bindings.ts 豁免` |
| 7.2 | 独立 `find src src-tauri/src ... \| wc -l \| sort -rn \| awk '$1>300'` | ✅ | 输出仅 `1340 src/bindings.ts`（另有 1 行 `44512 总计` 为 `wc` 汇总行，非文件）→ 除生成物外**零超限** |
| 7.3 | 审阅脚本逻辑 | ✅ | `LINE_LIMIT=300`、`GENERATED_EXEMPT="src/bindings.ts"`、文件头 5 行内 `@LINE_LIMIT_EXEMPT` 识别、`find` 排除目录、超限 `exit 1`，逻辑与 AGENTS.md 口径一致 |
| 7.4 | 关键文件行数 | ✅ | `PackageList.vue` **247**（原 303）、`PackageBatchToolbar.vue` 149、`PackageListFilterToolbar.vue` 179、`check-file-lines.sh` 82，全部 ≤300 |

### 8. 重复收敛真实生效（含安全红线）

| 校验 | 结论 | 关键证据 |
|---|---|---|
| 8.1 `whoami` | ✅ 仅 1 处 | `Command::new("whoami")` 仅 `sysops/system_cmd.rs:33`（另 2 处为文件头注释） |
| 8.2 `api.github.com` | ✅ 唯一落点 | 仅 `checkers/github/api_urls.rs`（常量 + 黄金值单测）；11 个构造函数与 HEAD 原始 12 类 URL 字面量**逐一吻合**（`releases/latest`、`releases?per_page=&page=`、`releases/tags/`、`tags?...&sort=pushed_at&direction=desc`、`tags?per_page=1/100`、`commits?sha=&per_page=1`、`compare/...`、`license`、`languages`、`graphql`、`repos/{}/{}`） |
| 8.3 前端错误三件套 | ✅ 1 份且逐字等价 | 定义仅 `src/utils/errors.ts`；6 个关键词（网络/timeout/连接/connect/fetch/request failed）顺序与 HEAD 三份 `\|\|` 实现一致，`.some()` 短路或等价；两条模板 `` `${context}：网络异常…${msg.length>60?"（详情见日志）":""}` `` / `` `${context}失败：${msg}` `` 逐字一致；`showActionError(msg) → addToast("error", msg)` 与原 `showError(msg) → addMessage(footer,"error",msg) → addToast("error", text)` 完全等价（`addMessage` HEAD/工作区逐字一致，`_footer` 为占位参数） |
| 8.4 旧 composable 名 | ✅ 零残留 | `rg -o 'packageActions\|packageCheckActions\|packageDeleteActions\|packageSyncActions' src \| wc -l` = **0**；`git status` 中 4 个 `RM`（git mv 保留历史） |
| 8.5 `selection-change` 选中链路 | ✅ 保留 | `PackageList.vue:204` `@selection-change="(pkgnames: Set<string>) => setSelected(pkgnames)"`；另有 `:45/:152/:203/:215/:216/:222` 共 6 处 `setSelected` 相关回传（含 BlockView `@select`/`@select-all`/`@delete`），AGENTS.md 记录的「勾选 13 个 → 全量更新」历史 bug 链路未破坏 |
| 8.6 props/emits 契约 | ✅ 一一对应 | `PackageBatchToolbar` 8 个 defineEmits ↔ 父组件 8 个 `@` 事件；`PackageListFilterToolbar` 4 个 defineEmits ↔ 父组件 4 个 `@` 事件；props 亦逐项对应；`parseSelectValue` 仅存于子组件（父副本已删） |
| 8.7 Checker 日志行为等价 | ✅ 逐字一致 | `helpers.rs` 三条 `[版本检查]` 文案（完成/未找到/重定向 warn）与 HEAD 六个 checker 原文一致；`finish_check` 显式构造 `CheckResult{version,license,language_names}`，无 `Default` 吞字段；`MissingVersionLog` 三变体承载 gitee/gitlab 静默、redirect warn 的真实差异 |
| 8.8 sudoers 安全红线 | ✅ | `CACHE_CLEANUP_ARGS[0] == "-n"`（单测锁定）；`grep -qF` 幂等、`printf` + `>>` 追加、`chmod 440`、单测显式断言 `!cmd.contains("tee ")`；`CACHE_CLEANUP_ARGS[1..].join(" ") == CACHE_CLEANUP_RULE` 逐 token 一致；`build_sudoers_append_command` 与抽取前模板逐字一致 |
| 8.9 异步禁 `reqwest::blocking` | ✅ | 唯一使用点 `db/github_tag_cache_extend.rs:40` 已用 `std::thread::spawn(move \|\| reqwest::blocking::get(...))` 隔离到无 runtime 上下文线程，符合 AGENTS.md 安全规范 |

### 9. git diff 全貌

`git status --short` + `git diff HEAD --stat`：**106 files changed, 2385 insertions(+), 2494 deletions(-)**。改动**不都在 T01~T05 预期范围**，详见 P-02。`src/bindings.ts` 显示为 `??`（untracked），即"不在改动列表"表面成立但语义失效，详见 P-01。

---

## 三、Failed 明细（2 项，路由：Engineer）

> 两项均为**本轮新引入的 lint 告警**，不影响功能（编译/147 单测/前端构建全过），但按 AGENTS.md「提交前必跑 `cargo clippy --all-targets -- -D warnings`，任一失败不得提交」的硬约束，当前代码不满足提交条件。修复量极小（合计 2 行）。**我不修改源码**，请主理人转回工程师。

### F-01 `fe_log.rs:1 empty_line_after_outer_attr` 并非历史存量，而是本轮新引入（工程师定性错误）

- **校验**：`cargo clippy --all-targets -- -D warnings` → exit 101
- **期望**：按 CODE_QUALITY_ISSUES.md 8.3 / 工程师自检，该项应属 HEAD 原始存量，可用 `-A` 放行
- **实际**：**相对 HEAD 是新引入的**。`git show HEAD:src-tauri/src/commands/fe_log.rs` 第 15 行为 `use tauri::command;`，文档注释 `/** ... */` 之后紧跟该 `use` 语句、**没有空行**，故 HEAD 不触发 `empty_line_after_outer_attr`。本轮改造（删除 `use tauri::command;`、改为 `#[tauri::command]` + `#[specta::specta]` 双宏并把前端日志落盘）后，`git diff HEAD -- src-tauri/src/commands/fe_log.rs` 明确显示：

  ```diff
    */
  -use tauri::command;
   
  -/// 接收前端诊断日志并打印到终端（stdout，不写文件）。
  +/// 接收前端诊断日志：打印到终端 stdout，并按级别写入 applog 文件。
  ...
  -#[command]
  +#[tauri::command]
  +#[specta::specta]
  ```

  `cat -A` 复核：HEAD `*/$` → `use tauri::command;$` → `$` → `///...`；工作区 `*/$` → `$` → `///...`。文档注释本身即 outer attribute（`#[doc]`），其后紧跟空行正是该 lint 的触发条件，报错位置 `fe_log.rs:1:1` 与此一致。
- **涉及文件/行**：`src-tauri/src/commands/fe_log.rs:14-16`（`*/` 后的空行）
- **完整错误输出**：
  ```
  error: empty line after outer attribute
    --> src/commands/fe_log.rs:1:1
  error: could not compile `my-aur-helper` (lib) due to 2 previous errors
  ```
- **修复建议**：删除 `*/` 与 `/// 接收前端诊断日志...` 之间的空行（或在二者间保留一条非空的 `use`/注释），使文档注释后不直接跟空行。修复后严格 clippy 应仅剩 `trait_def.rs:33` 这一项真存量。

### F-02 `cargo check --release` 新增 `unused import: std::path::PathBuf`（工程师自检遗漏）

- **校验**：`cargo check --release`（exit 0，但有 warning）
- **期望**：零 error 且零新增告警；工程师声称 `cargo check`「零 warning」
- **实际**：
  ```
  warning: unused import: `std::path::PathBuf`
    --> src/ipc/registry.rs:16:5
  warning: `my-aur-helper` (lib) generated 1 warning
  ```
- **成因**：T01 把 `build_specta_router` / `export_bindings` 从 `lib.rs` 整体迁入 `src-tauri/src/ipc/registry.rs` 时，顶层保留了 `use std::path::PathBuf;`，但该函数被 `#[cfg(any(debug_assertions, test))]` 保护——`PathBuf` 只在函数体内使用。**release 构建下函数被裁掉、import 变成未使用**。对照 HEAD：`lib.rs:27` 的 `use std::path::PathBuf;` 在 `get_config_dir()`、`log_dir` 等 5 处真实使用，故 HEAD 无此告警。debug 的 `cargo check` / `clippy --all-targets` 都覆盖不到这个 cfg 分支，因此工程师自检完全漏掉。
- **涉及文件/行**：`src-tauri/src/ipc/registry.rs:16`（`use std::path::PathBuf;`）
- **修复建议**：给该 import 加 `#[cfg(any(debug_assertions, test))]`，或把 `use std::path::PathBuf;` 移入 `export_bindings()` 函数体内。修复后 `cargo check --release` 应为零 warning。

> 修复后建议的 Round 2 快速回归（约 2 分钟）：`cargo clippy --all-targets -- -D warnings -A unfulfilled_lint_expectations`（预期仅剩 trait_def 1 项，放行后 exit 0）+ `cargo check --release`（预期零 warning）+ `cargo fmt --all -- --check`。我可随时执行。

---

## 四、需 team-lead 确认的问题（非源码 bug，不计入 Failed）

### P-01 【最重要】`bindings.ts` 从未入库，"相对 HEAD 零变化"判据失效

- 事实：`src/bindings.ts` 在 `git status` 中为 `??`（untracked）；`git ls-files --error-unmatch src/bindings.ts` 报错「未匹配任何 Git 已知文件」；`git ls-tree -r HEAD` 无此文件；`git log --all -- src/bindings.ts` 无记录；`.gitignore` 未忽略它（只是从未 `git add`）。
- 后果：**`git diff HEAD -- src/bindings.ts` 恒为 0 行**，与生成物内容是否正确完全无关。工程师自检用的 `diff /tmp/bindings_baseline_t05.ts src/bindings.ts` 是「自证」（拿自己生成的文件比自己），我这次补上了**真实闭环**：保存工作区基线 → `cargo test --lib export_bindings` 现场重生成 → 字节级 `diff` = 0、sha 一致（`602437bdfe5a5e58`），这才是有效证据。
- 更深一层：**HEAD 根本不是 tauri-specta 架构**——`git show HEAD:src/api/software.ts` 用 `import { invoke } from "@tauri-apps/api/core"`，`git show HEAD:AGENTS.md:471` 写「API 模块内部调用 `invoke("command_name", args)`」，HEAD 无 `#[specta::specta]` 宏（0 处）、`collect_commands!` 仅 84 条、`Cargo.toml` 无 specta 依赖、无 bindings.ts。因此 team-lead 验收清单里「bindings.ts 相对 HEAD 零变化」的**前提不成立**。
- 建议：把 `src/bindings.ts` 纳入 git 跟踪（或明确写入 `.gitignore` 并改用「基线文件 + CI 重生成 diff」的校验方式），否则 IPC 铁律的第 3 条永远无法机械校验。

### P-02 工作区相对 HEAD 存在大量超出 T01~T05 描述范围的改动

`git diff HEAD --stat` = 106 files / +2385 / -2494，远超 5 个治理任务。逐项取证：

| 超范围项 | 证据 | 与设计文档的冲突 |
|---|---|---|
| **新增 3 个第三方依赖** | `src-tauri/Cargo.toml` diff 新增 `specta = "=2.0.0-rc.28"`、`specta-typescript = "=0.0.15"`、`tauri-specta = "=2.0.0-rc.28"`，并把 `tokio = "1"` 锁为 `"1.44.2"`；`Cargo.lock` +153 行 | 设计文档 1.3(6) / 6：「**不新增第三方依赖**」 |
| **整套 tauri-specta（方案 B）迁移** | 全部 10 个 `src/api/*.ts` 从 `invoke("cmd", args)` 改为 `commands.*`；后端命令加 `#[specta::specta]` 双宏；新建 `ipc/registry.rs`；生成 `bindings.ts` | 设计文档称"89 条命令整体搬迁"，隐含基线已是该架构 |
| **新增 5 个 Tauri 命令** | HEAD `collect_commands!` 84 条 → 工作区 88 条；新增 `enums::get_license / update_license / delete_license`、`maintainer_sync::sync_packages_by_maintainer` 等；`git grep maintainer_sync HEAD` **零命中** | 设计文档未登记任何新增命令 |
| **3 个设计清单外新文件** | `software_sync/maintainer_sync.rs`(242)、`db/software_info_sync.rs`(121)、`db/tests_aur_sync.rs`(171)；均带 `#[tauri::command]` / `#[test]` 且被注册 | 设计文档「新增文件 19 个」未含 |
| 其它 | `variables.css` 新增 CSS 语义别名（配色修复）、`SettingsLogSection.vue`、`felog.ts`、`types/package.ts`、`docs/*.mermaid`、`Cargo.lock` 等 | T01~T05 未提及 |

抽样确认这些是**「方案 B：IPC 全链路类型生成」+ 若干新功能（按维护者同步 / License CRUD / 软件更新工作台 / CSS 配色）**的独立工作，与治理重构叠加在同一未提交工作区里（如 `src/api/software.ts` diff 明写「命令名/参数/返回值经 tauri-specta 生成的 bindings 做编译期校验」，`variables.css` 明写「历史上多个组件引用了 --primary 等变量但从未定义」）。

**我不判定这是工程师的越界**：这些很可能是前序/并行任务遗留的未提交工作，且代码本身能编译、能构建、行为自洽。但按任务要求「确认改动都在预期范围、没有误改或半成品」，**必须请 team-lead 显式确认归属**；若属本轮夹带，则违反「不扩大改动范围」与「不新增第三方依赖」。另 `as unknown as` 工作区 10 处 / HEAD 0 处，亦随方案 B 新引入（CODE_QUALITY_ISSUES L-005 已如实登记为保留项）。

### P-03 AGENTS.md / 设计文档「89 个命令」计数偏差

实测：`collect_commands!` **88 条真实注册**（`registry.rs:47-134`），88 个 `#[tauri::command]` 真宏、88 个 `#[specta::specta]` 真宏、`bindings.ts` 88 个 `__TAURI_INVOKE` 调用——**四方一致、无遗漏**。AGENTS.md「当前 89 个命令 / 89 个 specta 宏」、设计文档「89 条整体搬迁」以及任务清单「89 条命令无遗漏」中的"89"应是把 `registry.rs:20` 注释里的 `commands::...` 字面量算进去了（`rg -o 'commands::' | wc -l` 恰为 89）。属文档计数错误（低风险），建议统一改为 88 或删除注释中的 `commands::` 字样以免误导后续 AI。

### P-04 设计文档签名与落地实现不一致（已确认行为等价，仅记录）

| 契约 | 设计文档 | 实际实现 | 判定 |
|---|---|---|---|
| `finish_check` | 3 参数 `(version, license, language_names)` | 5 参数 `(pkgname, version, license, language_names, missing_log)` | 更严谨：用 `MissingVersionLog` 承载 gitee/gitlab 静默、redirect warn 的真实差异，避免一刀切改日志；日志文案与 HEAD 逐字一致、字段显式构造 → **行为等价** |
| `showActionError` | `(footer, e, context) => void` | `(msg: string) => void` | 由 `formatErrorMessage(e, context)` 在调用侧组合；`addMessage` 的 footer 本就是占位参数 → **行为等价** |
| `is_rule_effective` | `AppResult<bool>` | `bool`（失败返回 false） | 与抽取前 `Some(text)=>has_nopasswd_rule, None=>false` 等价 → **行为等价** |

均为合理的落地调整，非缺陷；但设计文档未回填，建议同步以免误导。

### P-05 untracked 杂物（疑似半成品/临时文件）

`package.json.tmp`（**0 字节空文件**）、`.SRCINFO`（**0 字节空文件**）、`.Trash-0/`（回收站目录）、`deliverables/diagnostics-and-typing-assessment.md`。`package.json.tmp` 与 `package.json` 的 diff 显示前者为空壳残留。建议清理或确认归属，避免误提交。

---

## 五、Known Issues（登记，不影响本轮通过判定）

1. **F-01 / F-02** 两项本轮新引入 lint 告警 —— 待 Engineer 修复后 Round 2 回归。
2. **P-01** bindings.ts 基线管理缺失 —— 建议纳入 git 跟踪或建立 CI 重生成 diff 机制。
3. **P-02** 工作区叠加多批未提交改动、含 3 个新增第三方依赖 —— 待 team-lead 确认归属。
4. **P-03 / P-04 / P-05** 文档计数、设计签名、untracked 杂物。
5. 工程师已如实留痕的保留项（我复核其存在、未虚报清零）：M-006 剩余 23 处 `map_err` + 61 处 `state.db.lock()?`；M-008 9 处 `UpstreamCheckResult`；M-012 8 处 `Client::builder()`；`api_checker.rs` 多分支完成日志；L-002 存量描述式测试命名；L-003 `fmtTimestamp` 命名歧义；L-005 `as unknown as` 10 处；两个 Card 本地 `fmtTimestamp`。
6. `cargo clippy --all-targets -- -D warnings` 真存量 1 项：`trait_def.rs:33 unfulfilled_lint_expectations`（`#[expect(clippy::double_must_use)]` 与 async-trait/clippy 版本相关，HEAD 逐字一致）—— 建议单独专项处理或长期 `-A unfulfilled_lint_expectations` 放行。

---

## 六、独立结论：这次大规模重构是否安全？

**功能与类型层面：安全，可交付。** 依据：

- Rust 全目标 + release 编译零 error；147 个单测全过（含 5 项针对新公共方法的真实断言）；前端类型检查与生产构建通过；
- IPC 闭环成立：88 条注册 ↔ 88 个真宏 ↔ 88 个 bindings 命令三方一致，现场重生成 bindings **逐字节未变**，前端 `invoke(` 零残留、10 个 api 模块全走 `commands.*`；
- 行为等价性经逐字比对确认：Checker 三条日志文案、前端错误三件套（6 关键词 + 2 模板 + toast 出口）、GitHub 12 类 URL 黄金值、sudoers 提权命令模板与 `-n` 位置、`selection-change` 选中链路、组件 props/emits 契约，全部与 HEAD 一致；
- 300 行硬上限、重复收敛、安全红线（`-n` / 逐 token / 追加写 / 异步禁 blocking）均实测生效。

**工程治理层面：不达标，需回炉 2 处小修 + 1 项基建修复。** 依据：

- 严格 `cargo clippy --all-targets -- -D warnings` 仍 exit 101，其中 `fe_log.rs:1` 被工程师错误归为「历史存量」，实为本轮删除 `use tauri::command;` 留下的空行所引入；`cargo check --release` 另有 1 个 `unused import` 新告警被自检漏掉。按 AGENTS.md 提交前硬约束，**当前不满足提交条件**；
- 「bindings.ts 相对 HEAD 零变化」这条最关键验收判据因生成物从未入库而**恒为真**，等于没有校验；HEAD 与工作区之间还隔着整套 tauri-specta 架构迁移，验收口径与实际仓库状态错位。

**综合判定：Round 1 路由 = Engineer（F-01/F-02，2 行修复）+ team-lead 确认（P-01~P-05）。** 修复后我做一次 Round 2 快速回归即可收口；若 team-lead 确认 P-02 属前序已评审工作且接受 2 项 lint 告警以 `-A` 长期放行，则可直接判 NoOne 并注明例外。**不建议在现状下直接判定"全部通过、可提交"。**

---

## 附录：关键命令与原始输出索引

| 编号 | 日志 | 内容 |
|---|---|---|
| A1~A9 | `/tmp/qa_logs/A1_fmt.log` … `A9_clippy_allow.log` | Rust 全套：fmt / check×3 / test / export_bindings / clippy 严格与放行 |
| A9b | `/tmp/qa_logs/A9b.log` | 修正 lint 名后的 clippy 放行（exit 0） |
| C1~C4 | `/tmp/qa_logs/C1_vuetsc.log` … `C4_lines_indep.log` | 前端类型/构建/行数脚本/独立 awk |
| `/tmp/qa_bindings_baseline.ts` | 保存的 bindings 基线（1340 行, sha `602437bdfe5a5e58`） | 与重生成产物 diff = 0 |
| `/tmp/qa_head_cmds.txt` / `qa_ws_cmds.txt` / `qa_registry_fn.txt` / `qa_bindings_fn.txt` | 命令清单 comm 比对输入 | HEAD 84 / 工作区 88 / registry 88 / bindings 88 |

---

# Round 2 最终回归验证（最后一轮，2 轮硬上限）

> 时点：2026-11-25（Round 1 之后）｜验证人：Edward｜方式：**fresh eyes 独立执行**，未复用工程师/主理人自检结论
> 目标：验证 F-01 / F-02 修复**真实生效**且**无新增回归**；复核 P-01 规约落地、P-03 命令数回填、AGENTS.md 第 7 条措辞
> 日志：`/tmp/qa_logs/r2/`；bindings 基线：`/tmp/qa_round2_baseline.ts`

## R2.0 修复点定向核查（先看代码，再跑命令）

| 缺陷 | 工程师修复方式 | 独立核查结果 |
|---|---|---|
| F-01 `fe_log.rs:1` | 未采用改 `/*` 的方案（实证 `/** */` 是 outer doc comment、specta 会导出，改 `/*` 会让 bindings 1340→1324 行）；改为**保留 `/** ... */` + 删除 `*/` 与后续 `///` 之间的空行**，两 doc 块连续堆叠 | ✅ `cat -A`：`*/$` 后紧接 `/// 接收前端诊断日志...$`，**无空行**；文件头 `/**` 保留；`sed -n 160p AGENTS.md` 之外另见 R2-7 证明导出未变 |
| F-02 `registry.rs:16` | 删除顶层 `use std::path::PathBuf;`，在 `#[cfg(any(debug_assertions, test))] export_bindings()` 函数体内就近引入，并附成因注释 | ✅ `registry.rs:26-32` 见函数体内 `use std::path::PathBuf;`；顶层已无该 import；`fe_log.rs` 38 行 / `registry.rs` 138 行，均 ≤300 |

## R2.1 逐条验证结果（真实命令与输出）

| # | 校验项 / 命令 | 期望 | 实际 | 结论 |
|---|---|---|---|---|
| R2-1 | `cargo clippy --all-targets -- -D warnings` | F-01 消失，仅剩 1 项 `trait_def.rs:33` | **exit 101**，`error: this lint expectation is unfulfilled --> src/checkers/trait_def.rs:33:10`（`due to 1 previous error`）；`fe_log` / `unused_doc_comments` / `unused import` 关键词**零命中** | ✅ **F-01 彻底修复，无新告警** |
| R2-2 | `cargo clippy --all-targets -- -D warnings -A unfulfilled_lint_expectations` | exit 0 | **exit 0**，`Finished dev profile ... in 2.46s` | ✅ 放行唯一真存量后零告警 |
| R2-3 | `cargo check --release` | **零 warning**（F-02 消失） | **exit 0**，`warning:` 计数 **0**；`Finished release ... in 0.26s` | ✅ **F-02 彻底修复**（Round 1 我抓出的告警已消失） |
| R2-4 | `cargo check --all-targets`（debug） | exit 0 | **exit 0**，`Finished dev ... in 2.21s` | ✅ |
| R2-5 | `cargo fmt --all -- --check` | exit 0 | **exit 0**（无输出） | ✅ |
| R2-6 | `cargo test --lib` | 147 passed | **147 passed / 0 failed / 0 ignored / 0 measured**（0.07s） | ✅ 无回归 |
| R2-7 | **bindings 真闭环**：`cp src/bindings.ts /tmp/qa_round2_baseline.ts` → `cd src-tauri && cargo test --lib export_bindings` → `diff` | diff 空 / **1340 行** / 仍 `??` | 基线 `1340 行 sha=602437bdfe5a5e58`；`export_bindings` = 1 passed；`R2_7_BINDINGS_DIFF_EXIT=0`、`DIFF_LINES=0`；重生成 `1340 行 sha=602437bdfe5a5e58`；`git status --short src/bindings.ts` = **`?? src/bindings.ts`** | ✅ **最关键项通过**：F-01 保留 doc comment 方案**未影响 specta 导出**，1340 行逐字节不变 |
| R2-8 | `pnpm exec vue-tsc --noEmit` | exit 0 | **exit 0**（无输出） | ✅ |
| R2-9 | `pnpm build` | exit 0 | **exit 0**，`✓ built in 3.36s` | ✅ |
| R2-10 | `bash scripts/check-file-lines.sh` | exit 0，仅 bindings.ts | **exit 0**；`src/bindings.ts (1340 行) [生成物豁免]` + `OK: 仅 bindings.ts 豁免` | ✅ |
| R2-11 | 独立 `find … \| wc -l \| awk '$1>300'` | 除生成物外零超限 | 输出 2 行：`44513 总计`（wc 汇总行，非文件）+ `1340 src/bindings.ts` | ✅ |
| R2-12 | **P-03** `rg -n '89' AGENTS.md CODE_QUALITY_ISSUES.md docs/system_design.md src-tauri/src/ipc/registry.rs` | 剩余命中全为非命令数量语境 | **7 处，全部非命令数量**：`43,289 行`×2、`183-289`/`186-189`/`145,189` 行号范围×3、`189` 行数、`1891 包` | ✅ |
| R2-13 | 命令数语境全为 88 | 88 | AGENTS.md:159「88 个命令 / 88 个 specta 宏」；registry.rs:5/14/19 三处「88」；CODE_QUALITY_ISSUES.md:115/118/121/417 四处；system_design.md:35/191/348 三处 —— **共 11 处全为 88** | ✅ |
| R2-14 | 四方计数一致性 | 88 | `commands::` 命中 89（含 1 处注释）、unique 注册 **88**、`^#[tauri::command]` **88**、`^#[specta::specta]` **88**、bindings invoke **88** | ✅ 无遗漏 |

## R2.2 AGENTS.md 第 7 条规约措辞评估（team-lead 指定「顺带扫一眼」）

第 168-174 行新增第 7 条，逐项评估：

- ✅ **可机械执行、无实质歧义**：明确写出「`git diff HEAD -- src/bindings.ts` 恒为空、属于**假阴性**，严禁据此判断 IPC 是否变化」，给出可直接复制的三步闭环命令，明确触发条件（改命令签名 / 参数 / 返回类型 / **命令 `///` 文档注释**，注明 specta 会导出 doc comment），明确判定标准（`diff` 输出必须为空），明确异常处置（修到完全一致或经显式评审接受），并约束「可选改进需单独授权、不得擅自 `git add`」。
- ✅ 与本次 F-01 修复方式自洽：正因为规约点明「命令 `///` 文档注释会导出到 bindings」，工程师才放弃改 `/*`，我 R2-7 实测 1340 行不变，证明该规约描述正确且被正确执行。
- ⚠️ **1 处可执行性建议（非阻断）**：代码块中第 2 行 `cd src-tauri && cargo test --lib export_bindings` 会改变当前目录，第 3 行按字面「同一 shell 顺序执行」时 `diff /tmp/bindings_baseline.ts src/bindings.ts` 会被解析为 `src-tauri/src/bindings.ts`（不存在）。实测：
  ```
  literal_EXIT=2  → diff: src/bindings.ts: 没有那个文件或目录
  correct_EXIT=0  → 改为 ../src/bindings.ts
  ```
  失败是显式报错、不会误判为「一致」，但会卡住执行者。建议把第 3 行改为 `../src/bindings.ts`，或把第 2 行写成子 shell `(cd src-tauri && cargo test --lib export_bindings)` 后回到仓库根。
- ⚠️ **1 处文档路径滞后（非阻断）**：`AGENTS.md:160` 仍写「只允许 `src-tauri/src/lib.rs::build_specta_router()` 中的 `collect_commands!`」，但 T01 已把该函数迁到 `src-tauri/src/ipc/registry.rs:43`（实测 `lib.rs` 已无 `fn build_specta_router`）。属本轮重构后的文档滞后，建议同步为 `ipc/registry.rs::build_specta_router()`，否则后续 AI 会去 lib.rs 找注册点。

## R2.3 并行文档回填状态（如实记录，不作为失败）

主理人说明工程师**正在并行**把「2 项 clippy 告警」历史记录修正为「1 项真存量 `trait_def.rs:33` + `fe_log.rs:1` 已修复」，且明确「不影响编译/测试类验证」。我在 Round 2 扫描时点（14:1x）取证：

- `docs/system_design.md:332` 仍为「T01 遗留 2 项 clippy 历史告警（`fe_log.rs:1` / `trait_def.rs:33`）」
- `CODE_QUALITY_ISSUES.md:484`、`:601-602`、`:712`、`:722` 仍为「T01 遗留 2 项历史告警」「除 T01 遗留 2 项历史告警外…」，其中 `:712` 还写着「与本轮无关」（Round 1 已证该定性错误）

→ 即**扫描时点回填尚未落盘**。这与 Round 2 事实（严格 clippy 仅剩 1 项真存量、`fe_log.rs:1` 已彻底消失）不符，属纯文字、非阻断；**请主理人确认最终落盘**。我不把它计入验证失败（遵守「并行文档回填不影响编译/测试类验证」的约定），也不改文档。

## R2.4 最终 Summary 与路由判定

| 项 | 结果 |
|---|---|
| Round 2 校验项 | **14 条**（R2-1 ~ R2-14），含 2 项定向修复核查 |
| 通过 | **14 / 14** |
| 失败 | **0** |
| 新增回归 | **0**（clippy 无 `unused_doc_comments`/`unused_imports` 等新告警、release 零 warning、147 单测无回归、bindings 逐字节不变、前端构建正常） |
| **最终路由判定** | ✅ **NoOne** |

**Round 1 两项缺陷的闭环**：
- F-01（`fe_log.rs:1 empty_line_after_outer_attr`，本轮新引入）→ 删空行后严格 clippy 中**完全消失**，且未引入 `unused_doc_comments`；保留 `/** */` 使 bindings 维持 1340 行 ✓
- F-02（`registry.rs:16` release 下 unused import）→ `cargo check --release` **warning 计数 0** ✓
- P-01（bindings 基线）→ AGENTS.md 第 7 条规约落地，我按该规约亲自跑通闭环并验证判据有效 ✓
- P-03（命令数 89→88）→ 11 处命令语境全部 88、`rg '89'` 剩余 7 处全为非命令语境、四方计数一致 ✓

## R2.5 独立结论：是否可交付？

**结论：可以交付（Round 2 修复后）。**

1. **功能与类型安全**：编译（debug/release 全目标）、147 单测、前端类型检查与生产构建、300 行硬上限全部通过；Round 1 建立的 IPC 字节级一致性、行为等价性、重复收敛、安全红线结论在 Round 2 未被破坏（bindings sha 仍 `602437bdfe5a5e58`，147 单测全过）。
2. **两项缺陷真实修复且无回归**：F-01/F-02 均经我独立复验消失，且严格 clippy 只剩 **1 项已用 `git show HEAD:` 证实的 HEAD 原始存量**（`trait_def.rs:33`），放行后零告警；`cargo check --release` 从「1 warning」变为「0 warning」。按 AGENTS.md 提交前硬约束，当前**已满足提交条件**（提交时用 `-A unfulfilled_lint_expectations` 放行该存量，或专项修）。
3. **验证基建修复到位**：bindings 未跟踪导致的「git diff 假阴性」已写入 AGENTS.md 强制规约，且我按规约跑通了唯一合法闭环，IPC 铁律第 3 条从此可机械校验。

**遗留项（非阻断，写入 Known Issues，不进入第 3 轮）**：
- K-1 严格 clippy 仍有 1 项 HEAD 存量 `trait_def.rs:33 unfulfilled_lint_expectations`（`#[expect(clippy::double_must_use)]` 与 async-trait/clippy 版本相关），需长期 `-A` 放行或专项修；
- K-2 并行文档回填扫描时点未落盘（R2.3 所列 5 处旧文案），请主理人确认；
- K-3 `AGENTS.md:160` 注册点路径滞后（`lib.rs` → `ipc/registry.rs:43`）；
- K-4 AGENTS.md 第 7 条代码块 `cd src-tauri` 后的相对路径歧义（建议 `../src/bindings.ts`）；
- K-5 Round 1 P-02 的范围问题**仍需主理人最终确认归属**：`git diff HEAD --stat` = 106 files / +2392 / -2495，含新增 3 个第三方依赖（specta / specta-typescript / tauri-specta）、相对 HEAD 新增 5 个命令、整套 tauri-specta 迁移、3 个设计清单外新文件、CSS/日志等改动；Round 2 未再变化。此项**不属源码缺陷**，但若确认为本轮夹带则违反「不扩大改动范围 / 不新增第三方依赖」；
- K-6 Round 1 已如实留痕、工程师保留未清零的项：M-006 / M-008 / M-012、`api_checker.rs` 多分支日志、L-002 / L-003 / L-005、两个 Card 本地 `fmtTimestamp`、untracked 杂物（`package.json.tmp` / `.SRCINFO` 空文件、`.Trash-0/`）。

> 交付建议：**判定 NoOne，允许交付**；把 K-1~K-6 排入后续文档/专项小修，不阻塞本次。2 轮硬上限已用满，不再进入第 3 轮。

---

> 报告生成：Edward / QA 工程师 / 2026-11-25（Round 1 + Round 2）。全程未修改任何源码文件；仅写本报告。
