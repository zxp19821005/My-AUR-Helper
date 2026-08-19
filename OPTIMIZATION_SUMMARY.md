# 整体优化总结（2026-08-19）

> 本轮对 My-AUR-Helper 前后端全部模块做整体体检与优化，共完成 **11 项任务**，
> 覆盖后端（Rust）错误处理、性能、日志脱敏、代码拆分，与前端（Vue/TS）死代码清理、
> 类型拆分、反模式修复、状态集中化、重复逻辑抽取。
> 全部改动通过 `vue-tsc --noEmit`、`vite build`、`cargo check --lib`、`cargo clippy --all-targets` 验证。

---

## 一、后端优化（T1-T6）

### T1：修复 proxy/basic.rs 吞掉插入错误并修正下载计数

- **问题**：代理导入时 `let _ = db.insert_proxy(...)` 静默吞掉写入失败，用户会看到"导入成功 N 条"但实际部分未入库；`download_proxy_file` 无条件返回 `Ok(0)`。
- **修复**：逐条 `if let Err(e) = ... { warn!(...) } else { count += 1 }`，只统计真实成功数；下载命令改为返回真实解析条数（抽取 `parse_and_insert_proxies` 供下载/解析两命令复用，消除重复）。

### T2：共享 reqwest::Client 替代每请求新建（OnceLock）

- **问题**：多处每请求 `Client::new()` 重复建立连接池，批量场景下握手/资源开销大。
- **修复**：
  - 新增 `src-tauri/src/http_client.rs`：`shared_client()` 用 `OnceLock` 缓存单例客户端（30s 默认超时），代理导入/下载复用。
  - 重定向检查器 `redirect.rs` 新增 `redirect_client()`（`Policy::none` + 浏览器 UA，`OnceLock`）。
  - 上游 URL 验证 `upstream_validate.rs` 新增 `validate_client()`（10s 超时，`OnceLock`）。
  - 三类无特殊需求/有特殊需求的场景各自复用，避免每请求重建。

### T3：日志脱敏

- `proxy/basic.rs` `update_proxy` 日志不再打印完整代理 URL（新增 `mask_url`，与 `proxy/test.rs` 同思路；数据库仍写入真实 URL）。
- `db/software_info.rs` 详情日志不再 dump 完整 license JSON，改为 `has_aur_license` / `has_upstream_license` 两个布尔值。

### T4：热路径静态正则改为 OnceLock 懒加载

- `aur/pkgbuild.rs`：8 处每调用 `Regex::new(...).unwrap()` → `pkgbuild_regexes()`（`OnceLock` 结构体，进程内仅编译一次）。
- `checkers/utils.rs`：5 个静态正则（clean/url_v/url_num/html_kw/html_td）收敛到 `static_regexes()`；用户动态传入的正则（`extract_version_with_regex`）保持不变。

### T5：拆分超 300 行后端文件

| 原文件 | 原行数 | 拆分后 | 说明 |
|--------|--------|--------|------|
| `checkers/github/graphql_batch.rs` | 376 | 217 + `graphql_batch_helpers.rs`（171） | 6 个纯辅助函数（版本选择/比较）迁移 |
| `commands/sysops/software_sync/batch.rs` | 371 | 232 + `batch_helpers.rs`（158） | 任务分类/单包检查/重试辅助迁移 |

- 子模块声明按项目惯例放在父级 `mod.rs`（文件模块的子模块须在其同名目录下，或由父 mod 声明为同级文件）。

### T6：backup_install.rs 测试显式处理 FS 错误

- 测试 setup 中 `let _ = create_dir_all(&tmp)` → `.expect("创建测试目录失败")`，失败立即暴露而非静默继续。

---

## 二、前端优化（T7-T11）

### T7：删除死代码组件 `ProxyRowActions.vue`

- 全仓库无任何 import 引用（仅自身文档注释提到），是 2026-07-29 拆分记录遗留的孤儿组件，直接删除。

### T8：拆分超 300 行前端文件

| 原文件 | 原行数 | 拆分后 | 说明 |
|--------|--------|--------|------|
| `types/index.ts` | 313 | `index.ts`（17，barrel）+ 7 个领域模块 | package/proxy/enum/backup/cache/settings/dashboard |
| `views/CacheManager.vue` | 301 | 282 + `useCacheManagerInit.ts`（71） | onMounted 5 项初始化聚合到 composable |

- `types/index.ts` 保留 `export *` 再导出，所有现有 `import type { X } from "../types"` 零改动。

### T9：修复 useProxyList.ts 空 try/catch 反模式

- `extractProxyName` 纯字符串操作套无意义 try/catch → 移除。
- `toggleProxyActive` / `updateProxy` / `deleteProxy` 的 `catch (e) { throw e; }` 空转重抛 → 移除，异常自然向上传播。
- 顺带修复 `BackupManager.vue` 两处空 catch（补 `console.error` 诊断）+ `loadSettings` 里冗余的动态 `import()`（改为使用顶部静态 `invoke`）。
- 全局残留的 3 处 `checkSudoers` 空 catch 由 T11 的 `useSudoers` 统一收口；`usePopupWindow.ts` 的 `unminimize` 空 catch 属尽力而为操作，保留。

### T10：集中化 set_setting 并并行化 useCacheDirs 串行加载

- **集中化写入**：全部 `invoke("set_setting", ...)` 收敛到 settings store 的 `setSetting`（写库 + 同步缓存）。涉及 `SettingsLogSection`（4 项并行保存）、`SettingsCacheSection`（默认目录 path/enabled 并行写）、`SettingsDynamicSection`（Promise.all 全量保存）、`useCacheDirs.saveCustomCacheDirs`（内部走 store）。
- **并行化读取**：`loadCacheDirs` / `loadEnabledCacheDirs` 各 7 次串行 `get_setting` → `Promise.all` 并发，缓存目录加载延迟显著下降。

### T11：抽取 useSudoers composable 消除 3 处重复

- `useBackupInstall` / `useCacheInstall` / `useCacheCleanup` 三处重复的 `sudoersAvailable` / `sudoersCommand` / `showSudoersPrompt` 状态与 `checkSudoers` / `loadSudoersCommand` / `closeSudoersPrompt` 实现，收敛为 `useSudoers.ts`，后端命令名参数化注入（`check_sudoers_config` / `check_cache_install_sudoers` / `check_cache_cleanup_sudoers` 等）。
- 行为保持不变（含 `sudoersAvailable` 失败置 false 的历史语义），空 catch 补 `console.error` 诊断。

---

## 三、验证结果

| 检查项 | 结果 |
|--------|------|
| `vue-tsc --noEmit` | ✅ 通过（0 错误） |
| `vite build` | ✅ 通过（2.5s） |
| `cargo check --lib` | ✅ 通过 |
| `cargo clippy --all-targets` | ✅ 无新增 warning（既有警告与本次改动无关） |
| 行数规范（≤300 行） | ✅ 本次涉及文件全部达标 |

---

## 四、已知限制与后续建议

1. **ESLint 缺失**：`package.json` 的 `lint` 脚本引用的 `eslint` 未安装（devDependencies 无此依赖），`pnpm lint` 无法执行。当前以 `vue-tsc` + `vite build` 作为前端质量门禁；建议补充 ESLint 配置（`eslint-plugin-vue` + `typescript-eslint`）并修复存量告警。
2. **CODE_QUALITY_ISSUES.md 遗留 2 项**：L-008（packageActions.ts 命名不一致）、L-010（api_checker.rs CheckResult 构造重复 4 次），均为低优先级，需更大范围重构，留待后续。
3. **vite 6.x 升级**：3 个前端 dev-server 相关中危依赖漏洞需 vite 主版本升级，不影响生产构建，建议下个周期评估。
4. **既有 clippy 警告**：`versions/comparison.rs`（needless_borrow）、`comparison/tests.rs`（module_inception）等为历史遗留，未在本轮处理。
