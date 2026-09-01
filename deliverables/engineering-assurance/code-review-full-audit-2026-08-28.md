# My-AUR-Helper 全面工程审查报告

**日期**：2026-08-28
**工作流**：工作流 1 - 全面代码审查（含架构影响评估 + 测试覆盖评估）
**参与成员**：Cody（代码审查师）、Archi（系统架构师）、Tessa（测试专家）

---

## 📌 TL;DR（执行摘要，3-5 行）

- 整体结论：架构分层清晰、SQL 参数化 / 敏感信息脱敏 / CSP 等基础安全扎实，但存在 **1 项严重代码安全缺陷（任意路径写入穿越）** 与 **4 项高危安全问题**，且 **测试与 CI 完全缺失**，整体未达合并 / 发布门禁。
- 严重度分布：🔴严重 7 项（1 代码缺陷 + 6 测试真空） / 🟠高 10 项 / 🟡中 16 项 / 🟢低 5 项（已跨来源去重）
- 阻塞 / 非阻塞：发布前阻塞 2 类 —— F-01 任意路径写入（代码级）；测试框架 + CI 缺失（工程级）
- 三维度评级：代码安全 🔴（有可利用漏洞）、架构 🟡（需改进，无🔴）、测试 🔴（严重不足）

---

## 🎯 核心结论卡片

| 项目 | 内容 |
|------|------|
| 整体评级 | 🔴 不通过（合并 / 发布门禁未达） |
| 阻塞项数量 | 2（F-01 任意路径写入；测试 + CI 缺失） |
| 关键行动项 | 10 条（见行动清单） |
| 建议下一步 | 优先修复 F-01 / F-02 / F-05 文件操作沙箱与 F-03 / F-04 浏览器检查器安全项，并立即接入 vitest + cargo test + GitHub Actions CI 门禁 |

---

## 🔍 审查发现（按严重度排序）

> 合并 Cody 代码缺陷、Archi 架构风险、Tessa 测试缺口，跨来源去重；来源列标注 **C**=Cody / **A**=Archi / **T**=Tessa。

### 🔴 严重

| # | 严重度 | 类别 | 文件:行 | 问题描述 | 建议修复 | 来源 |
|---|--------|------|---------|---------|---------|------|
| R1 | 🔴严重 | 安全·代码 | cache_backup/existing.rs:43,158；subdirectory.rs:20,51 | 任意路径写入 + 路径穿越：backup_cache_to_* 直接采用前端传入的 backup_path/subdirectory 作复制目标，create_dir_all+copy 前无路径校验/沙箱，subdirectory 含 ../ 可逃逸写任意位置 | 复用 validate_package_path：canonicalize 后强制前缀匹配 backup 根目录白名单，拒绝任何 .. 段 | C |
| R2 | 🔴严重 | 测试 | 前端 src/（16,249 LOC） | 前端零测试框架、零测试文件，无 vitest/jest；Vue 组件/Pinia store/IPC 封装无回归保护 | 接入 vitest + @testing-library/vue + happy-dom；先给 src/api 与核心 store 写冒烟测试 | T |
| R3 | 🔴严重 | 测试 | checkers/（19/21 文件无测试） | 产品核心价值「版本检查」的解析逻辑（正则/JSON）无测试，版本误判/漏报风险极高 | 提取纯函数 + 样本 fixture 参数化单测 | T |
| R4 | 🔴严重 | 测试 | db/ 下 8 个 migration_*、connection.rs 及全部 entity 模块 | SQL 迁移幂等性/PRAGMA/关键查询无验证；迁移失败或数据损坏无拦截 | :memory: 库做「迁移幂等 + 关键查询参数化」测试 | T |
| R5 | 🔴严重 | 测试·安全 | commands/sysops/backup_install_helpers.rs | sudoers 免密规则判定（has_pacman_install_rule/build_pacman_install_rules）零测试，误判安全风险 | 补充单元断言（见 Tessa 原始产出示例） | T |
| R6 | 🔴严重 | 测试 | src-tauri/tests/（空目录） | IPC 命令入口从未被集成测试，命令→db→返回跨模块数据流未验证 | 新增集成测试，断言命令入参/出参/错误类型 | T |
| R7 | 🔴严重 | 测试·CI | 仓库无 CI 配置 | 任何提交不触发 build/lint/test，回归无法拦截 | 加 GitHub Actions（build+lint+test 门禁） | T |

### 🟠 高

| # | 严重度 | 类别 | 文件:行 | 问题描述 | 建议修复 | 来源 |
|---|--------|------|---------|---------|---------|------|
| R8 | 🟠高 | 安全 | fileops/scan.rs:114-117；backup_scan.rs:46-61 | 任意目录读取无沙箱：scan_pkg_files_cmd/scan_backup_directory 接受任意路径递归枚举，违反红线条2 | 仅允许扫描 DB 配置的可信缓存/备份根目录，自定义路径须 canonicalize 前缀校验 | C |
| R9 | 🟠高 | 安全 | checkers/browser.rs:150-158（--no-sandbox） | 渲染进程沙箱关闭，访问互联网 URL，被利用则无沙箱本地代码执行 | 保留沙箱或低权限专用用户运行 + seccomp/namespace 限制，文档化取舍 | C |
| R10 | 🟠高 | 安全 | checkers/browser.rs:157 | Chrome 参数注入：upstream_url 作位置参数传 --dump-dom，以 -- 开头可注入任意 Chrome 参数 | URL 前插入 -- 分隔符，或校验必须以 http(s):// 开头 | C |
| R11 | 🟠高 | 安全·正确 | sysops/cache_cleanup.rs:87-115 | 数据丢失 footgun：clean_custom_cache_dirs 对配置目录 remove_dir_all 删所有非隐藏子项，误设目录即清空用户数据 | 仅删已知缓存产物（.pkg.tar.* 与既定构建子目录），加根目录约束 | C |
| R12 | 🟠高 | 架构·可维护 | software_check.rs:403 | 超 300 行上限；check_with_retry 与 batch_helpers 重复；单包/批量写库逻辑分裂易 diverge | 抽取共享 result_writer 模块，single/selected 统一调用，拆分后 <300 行 | A |
| R13 | 🟠高 | 架构·并发 | lib.rs AppState + db/connection.rs | 单 Mutex<Database> 在 async 命令中跨 .await 持有无静态保证，死锁/吞吐风险 | 将"不得跨 await 持有 db/memory_cache Mutex"写入 AGENTS.md 硬约束 + 评审卡点；长期评估连接池/async SQLite | A |
| R14 | 🟠高 | 测试 | commands/（5,712 LOC 仅 4 测试） | 除 backup_install 外命令逻辑无测试，错误只能运行时暴露 | 按风险挑核心命令做契约测试 | T |
| R15 | 🟠高 | 测试 | src-tauri/Cargo.toml 无 dev-deps | 无 mockall/tempfile/wiremock，难以 mock 外部 HTTP 与文件系统 | 添加必要 dev-deps | T |
| R16 | 🟠高 | 测试 | package.json 无 test 脚本；无统一入口 | 开发者易遗忘运行测试 | 加 test 脚本 + cargo alias | T |
| R17 | 🟠高 | 测试 | network/retry.rs（80 LOC 0 测试） | 外部 HTTP 重试/退避/超时异常路径未验证 | 参数化退避/超时测试 | T |

### 🟡 中

| # | 严重度 | 类别 | 文件:行 | 问题描述 | 建议修复 | 来源 |
|---|--------|------|---------|---------|---------|------|
| R18 | 🟡中 | 安全 | lib.rs:69 + capabilities/default.json:8 | tauri_plugin_shell 仅授予 shell:allow-open，open 可借前端打开任意 file://，插件可能多余 | 确认 open 是否使用，否移除插件以缩小暴露面 | C |
| R19 | 🟡中 | 安全·可维护 | sys_command.rs:23,50 | get_package_version/list_installed_packages 前端无调用，list_installed 暴露全部已装包 | 全局确认后移除未使用危险命令 | C |
| R20 | 🟡中 | 安全·可维护 | backup_install_helpers.rs:122；cache_cleanup.rs:182,209 | 硬编码用户专属路径魔法值 + sudoers 路径未转义（含空格/逗号失效） | 默认值可配置/常量集中；拼 sudoers 前引号包裹+合法性校验 | C |
| R21 | 🟡中 | 正确性 | software_sync/upstream.rs:91 vs 111-115 | 版本规范化时序错误：is_outdated 用原始上游版本（可能带 v），入库却 strip v，导致比较偏离 | 比较前统一 strip_prefix('v')/清洗，比较与入库用同一规范化值 | C |
| R22 | 🟡中 | 正确性·性能 | software_sync/upstream.rs:108-176 | 批量写库未在事务内、持锁贯穿整轮循环，非原子、长时间阻塞、中途失败留部分更新 | BEGIN TRANSACTION 包裹整轮写入或批量 upsert 一次性提交 | C |
| R23 | 🟡中 | 可维护 | SettingsMemoryCacheSection.vue:409、packageActions.ts:366、lib.rs:303（software_check.rs 见 R12） | 3 处前端/入口文件超 300 行（违反强制限制），且有回归 | 按单一职责拆分；lib.rs 可显式豁免或聚合 handlers() 子函数 | C+A |
| R24 | 🟡中 | 架构 | lib.rs:303 | 命令注册清单回升至 303 行再超 300 | 聚合 handlers() 子函数组合或显式豁免 | A |
| R25 | 🟡中 | 架构 | SettingsMemoryCacheSection.vue:409 | 超 300 行未拆分 | 按子功能拆组件 + composable | A |
| R26 | 🟡中 | 架构 | packageActions.ts:366 | composable 超 300 行，承载多类操作 | 按操作类型拆分或使用页面 composable | A |
| R27 | 🟡中 | 架构 | PackageList.vue:301 | 已拆至 240 现回升 301 | 复查增量迁移回 PackageTable | A |
| R28 | 🟡中 | 架构·数据 | db/connection.rs + migration_*.rs | 迁移无版本表/框架，无回滚/历史，无法表达破坏性迁移 | 引入 schema_migrations 版本表或明文记录策略补幂等测试 | A |
| R29 | 🟡中 | 架构·前后端 | stores/composables/api | 写命令变更数据后 store 不自动失效，跨窗口状态不一致风险 | 后端关键写操作 emit 变更事件，前端 mitt 总线/订阅刷新 | A |
| R30 | 🟡中 | 测试 | src/api/*.ts（10 文件） | IPC 封装命令名/参数若与 Rust 端漂移无测试发现 | vitest + mock invoke 断言 | T |
| R31 | 🟡中 | 测试 | src/stores/*.ts | 状态派生/过滤逻辑未验证 | store 单元测试 | T |
| R32 | 🟡中 | 测试 | 无覆盖率度量/门禁 | 无法量化覆盖防退化 | 引入 tarpaulin/vitest coverage + 阈值 | T |
| R33 | 🟡中 | 测试 | models/（635 LOC 0 测试） | 与前端 TS 类型对齐的序列化边界未验证 | 往返序列化测试 | T |
| R34 | 🟡中 | 测试 | errors/（329 LOC 0 测试） | AppError→前端错误码映射未验证 | 映射测试 | T |

### 🟢 低

| # | 严重度 | 类别 | 文件:行 | 问题描述 | 建议修复 | 来源 |
|---|--------|------|---------|---------|---------|------|
| R35 | 🟢低 | 正确性 | backup_scan.rs:35,66 | path.file_name().unwrap() 防御性 unwrap | 改 if let Some | C |
| R36 | 🟢低 | 正确性 | upstream_validate.rs:41 | 处理函数内 .expect 极端 panic 异步运行时 | 改 ? 返回 AppError | C |
| R37 | 🟢低 | 可维护 | db/software_info.rs 多处 | format! 拼接列名常量（入参已参数化，红线条3满足） | 提为 const 字符串常量 | C |
| R38 | 🟢低 | 架构·IPC | proxy/{basic,test}.rs + api/proxy.ts | test_proxy/test_proxies_batch/test_proxy_single 语义重叠 | 合并为 test_proxies(ids?) 单一命令 | A |
| R39 | 🟢低 | 测试 | versions/(43)/proxy(9)/cache(6)/validate_package_path(4) | 已覆盖良好，作为测试范式参考 | 维持并推广 | T |

---

## 🏗️ 架构影响评估（Archi）

**架构概览**：前端 Vue3 + Pinia 经 `src/api/` 统一封装 Tauri IPC；后端 Rust 命令层(`commands/`)→业务逻辑(`checkers/aur/proxy/backup`)→SQLite 数据层(`db/`)；版本检查采用 `VersionChecker` trait + `get_checker` 工厂分发，上游批量检查由 `software_sync/batch.rs` 分类并发引擎执行；全局单一 `Database` 经 `Mutex` 串行化访问。

**架构优势（实测验证）**：
1. 分层与 IPC 边界清晰——前端无一处直接 `import { invoke}`（全量 grep 验证），`mod.rs` 16 个文件全部仅做声明，无实现泄漏。
2. 检查器体系可扩展性佳——trait + 工厂符合开闭原则，新增上游源成本低。
3. 数据层引用完整性到位——级联删除 + `foreign_keys=ON` + WAL + 动态列查询表白名单防注入。
4. 安全面收敛良好——已移除 `run_command`/`install_package` 等任意执行命令；`sys_command.rs` 用严格正则校验 pkgname。
5. 前端状态管理取舍合理——仅 3 个 Pinia store，列表用 `shallowRef` 避免深度响应式阻塞。

**ADR 建议**：
- **ADR-1（收敛版本检查结果写入路径）**：抽取 `software_sync/result_writer.rs` 共享 `check_with_retry` + `compare_and_write`，消除 software_check.rs 与 batch 引擎的重复与 300 行违规，低成本高收益，优先。
- **ADR-2（数据层并发与迁移治理）**：① 将"不得跨 await 持有 db/memory_cache Mutex"写入 AGENTS.md 硬约束 + PR 卡点；② 引入 `schema_migrations` 版本表；③ 评估 rusqlite 连接池 / async SQLite。

**整体架构健康度：🟡 需改进**（无🔴级安全/数据事故）。

---

## 🧪 测试覆盖评估（Tessa）

**测试现状总览**：
- 前端：❌ 未接入测试框架，0 测试文件（16,249 LOC 无回归保护）。
- 后端：⚠️ 仅 Rust 内置 `cargo test`，无 `[dev-dependencies]`；76 个 `#[test]` 但 56% 集中在 `versions/`。
- 集成测试：❌ `src-tauri/tests/` 为空目录。
- CI：❌ 无 `.github/workflows/` 等任何配置。

**测试缺口（详见 R2-R7/R14-R17/R30-R34/R39）**：前端零测试、checkers/db/commands 业务关键路径大量零测试、sudoers 规则检测零测试、零集成测试、零覆盖率度量、零 CI 门禁。

**推荐测试策略（分层 + 优先级）**：
- P0：checkers 版本解析 fixture 测试、db 迁移幂等 + 关键查询、sudoers 规则检测、前端测试框架接入 + src/api 与核心 store 冒烟、CI 基础门禁。
- P1：commands IPC 契约集成测试、`network/retry` 退避/超时测试、前端组件测试、覆盖率工具 + 阈值门禁。
- P2：E2E（tauri-driver / Playwright）、混沌/负载测试。

**可直接执行示例（见 Tessa 原始产出）**：sudoers 规则检测单测、db 迁移幂等测试（:memory:）、前端 IPC 封装冒烟测试（vitest）。

**整体测试健康度：🔴 缺失 / 严重不足**（估算有效语句覆盖率 <15%，业务关键路径 <5%）。

---

## ✅ 行动清单（按优先级排序，至少 3 条具体可执行项）

| # | 行动 | 负责角色 | 紧急度 | 预期完成 |
|---|------|---------|--------|---------|
| 1 | 修复 R1 任意路径写入：cache_backup 复用到 validate_package_path 白名单校验，canonicalize + 前缀匹配 backup 根目录，拒绝 .. 段 | Cody（Rex 复核） | P0 | 1 周内 |
| 2 | 修复 R8/R11 文件操作沙箱：scan 限定可信目录；clean_custom_cache_dirs 仅删缓存产物 + 根约束 + 危险确认 | Cody | P0 | 1 周内 |
| 3 | 修复 R9/R10 浏览器检查器：移除 --no-sandbox 或低权限化 + URL 前插 -- 防参数注入 | Cody | P0 | 2 周内 |
| 4 | 接入测试框架 + CI 门禁：vitest + cargo test + GitHub Actions（build/lint/test） | Tessa（Docu 协助） | P0 | 2 周内 |
| 5 | 补齐 P0 单测：checkers 版本解析 / db 迁移幂等 / sudoers 规则（用 Tessa 提供示例） | Tessa | P0 | 3 周内 |
| 6 | ADR-1 收敛写库路径 + 拆分 software_check.rs（R12） | Archi | P1 | 1 月内 |
| 7 | DB 事务包裹批量写（R22）+ 将"不得跨 await 持锁"写入 AGENTS.md 硬约束（R13） | Archi + Cody | P1 | 1 月内 |
| 8 | 清理未使用命令/壳插件（R18/R19）、修复 R21 版本规范化时序 | Cody | P1 | 1 月内 |
| 9 | 前端 4 处大文件（>300 行）拆分（R23-R27） | Archi | P2 | 2 月内 |
| 10 | 迁移版本化（schema_migrations）+ 跨窗口状态失效机制（R28/R29） | Archi | P2 | 季度内 |

---

## ⚠️ 待完善 / 已知局限

- 本次为静态审查（Grep/Read + 部分行数统计），未运行 `cargo clippy`/`cargo test`/`vitest` 动态验证；R1-R11 等安全问题建议结合动态验证复核。
- 事故响应流程（工作流 3）因缺少具体生产事故/告警上下文未触发；如需运行，请提供：影响范围、报错信息、时间线、当前告警。
- 前端构建产物与 Rust target 已排除，未纳入审查范围。
- 严重度分布含跨来源去重（超 300 行大文件合并为 R23），测试真空类🔴与代码缺陷类🔴分属不同维度，已通过来源/类别列区分。

---

## 📚 数据来源 & 成员产出索引

- Cody（代码审查师）原始产出：14 项代码发现（F-01~F-14）+ 7 条强制安全红线核查结论；重点 R1/R8-R11/R18-R22/R35-R37。红线核查结果：红线条1/3/5/6/7 达标，红线条2 违反（F-01/F-02），红线条4 部分达标（F-06/F-07）。
- Archi（架构师）原始产出：架构概览 + 5 项优势 + 9 项风险（A-1~A-9）+ 2 条 ADR；重点 R12/R13/R23-R29/R38。
- Tessa（测试专家）原始产出：测试现状总览 + 16 项缺口（T-1~T-16）+ 推荐测试策略 + 可直接执行测试示例 + CI 建议；重点 R2-R7/R14-R17/R30-R34/R39。

---

> 本报告由工程保障团队 AI 协作生成，关键决策请由人类工程负责人复核。
