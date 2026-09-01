# 软件管理页面工具栏问题修复记录

> 日期：2026-09-01
> 范围：软件管理（PackageList）工具栏 4 项问题
> 验证：`vue-tsc --noEmit` 0 错误、`vite build` 通过、`cargo check` 通过

## 问题 1：下拉默认文案

- **现象**：「全部类型」语义不准确。
- **修复**：`src/views/PackageList.vue` 中 `packageType` 下拉的默认项文案由 `全部类型` 改为 `全部软件包`。

## 问题 2：工具栏单选快速筛选下拉

### 需求

- 在「全部检查器」下拉之后新增一个**筛选按钮**；
- 点击弹出**单选**下拉菜单，选项包含：AUR更新失败、上游更新失败、Github地址失效、无AUR版本、无上游版本……；
- 选中后按钮旁显示**清除筛选**按钮；
- **不支持多选**。

### 根因与方案

原先 `aur_info` / `upstream_info` 只记录版本字段，**「同步失败」与「无版本」在数据库层无法区分**（都表现为 `aur_version` 为空）。要精确区分必须在错误发生时落库错误原因。

**后端（Rust）**：
- `models/aur_info.rs` 新增 `last_sync_error: Option<String>`；`models/upstream_info.rs` 新增 `last_check_error: Option<String>`。
- `db/schema.rs` 建表 DDL 补列；`db/migration_aur.rs` / `db/migration_upstream.rs` 分别新增幂等 `ALTER TABLE ... ADD COLUMN`（`migrate_aur_error_column` / `migrate_upstream_error_column`），兼容老库。
- `db/aur_info.rs` / `db/upstream_info.rs` 的 upsert 写入新列；新增 `mark_aur_sync_error` / `mark_upstream_check_error`。
- 写入时机：
  - AUR 同步成功 → 置 `last_sync_error = NULL`；AUR RPC 未返回该包 → 标记 `AUR 中未找到该包`；写库失败 → 标记 `写入数据库失败`。
  - 上游检查成功 → 置 `last_check_error = NULL`；检查未返回版本 → 标记 `检查未返回上游版本`。
- `db/software_info.rs` 列表视图 `SW_LIST_COLS` 增加 `a.last_sync_error, u.last_check_error`，并映射到 `SoftwareListEntry`；`models/software_list_entry.rs` 增加对应字段。

**前端（TS/Vue）**：
- `types/package.ts` 的 `SoftwareListEntry` 增加 `aur_sync_error` / `upstream_check_error`。
- `composables/usePackageList.ts`：新增单选类型 `QuickFilterKey` 与 `quickFilterOptions`（7 项）；`FilterState` 新增 `quickFilter: QuickFilterKey | null`（与 FilterBar 的多重 `quickFilters` 并存，二者 OR 叠加）；`matchesQuickFilters` 新增单选项判定逻辑，用新错误字段精确区分「更新失败」与「无版本」；导出 `selectQuickFilter`（再次点击同项即取消，保证单选）、`clearQuickFilter`、`activeQuickFilterLabel`。
- `views/PackageList.vue`：在「全部检查器」后插入筛选按钮 + 下拉菜单 + 清除按钮 + 背景遮罩（`qf-backdrop` 点击关闭），并对 FilterBar 的多重筛选零侵入。

**筛选语义对照表**

| 选项 | 判定条件 |
|------|----------|
| AUR更新失败 | `aur_sync_error != null` |
| 上游更新失败 | `upstream_check_error != null` |
| Github地址失效 | `upstream_url_status` 非空且 != `ok` |
| 无AUR版本 | `aur_version == null` |
| 无上游版本 | `upstream_version == null` |
| 无上游地址 | `upstream_url == null` |
| License缺失 | `upstream_license_id == null` |

## 问题 3：清除搜索后页码回弹

- **现象**：在第 5 页搜索后再清空搜索条件，会直接跳回第 1 页，而非保留搜索前所在页。
- **根因**：`usePackageList.ts` 中 `watch(searchQuery, () => { currentPage.value = 1; })` 在搜索词**任何变化**时都强制回到第 1 页。
- **修复**：记录「进入搜索前」的页码 `pageBeforeSearch`。搜索由空变非空时保存当前页并跳第 1 页；由非空变空时恢复到 `pageBeforeSearch`。两非空值之间切换仍保持第 1 页（新一轮搜索）。`syncToolbar` 的边界夹紧逻辑照常兜底超界。

## 问题 4：查看详情弹窗延迟

- **现象**：点击「查看详情」后，内容区域有 1–2 秒空白感。
- **诊断**：`SoftwareDetailModal.loadSoftware()` 原实现**串行**调用两个 IPC：`getSoftwareDetail` 后再 `getPrevNextSoftware`，两个往返叠加。后端 `get_software_detail_by_name` 为单条 JOIN 查询、`get_prev_next_software` 为两次索引查询，**无 N+1**；瓶颈在前端串行 IPC。
- **修复**：
  1. 两个 IPC 改为 `Promise.all` 并行，往返由 2 次降为 1 次；
  2. 标题即时显示 `props.pkgname`（加载中也不再显示通用占位文案）；
  3. 用 `feDebug("Detail", ...)` 埋点打印并行耗时（输出到浏览器控制台 + 终端），便于后续量化真实延迟。

## 验证结果

| 检查项 | 命令 | 结果 |
|--------|------|------|
| 前端类型 | `npx vue-tsc --noEmit` | 0 错误 |
| 前端构建 | `npx vite build` | 通过（4.46s） |
| 后端编译 | `cargo check` | 通过 |

## 涉及文件

后端：`models/{aur_info,upstream_info,software_list_entry}.rs`、`db/{schema,migration_aur,migration_upstream,aur_info,upstream_info,software_info}.rs`、`commands/sysops/software_sync/{aur,upstream}.rs`、`commands/sysops/software_check.rs`

前端：`types/package.ts`、`composables/usePackageList.ts`、`views/PackageList.vue`、`components/package/SoftwareDetailModal.vue`
