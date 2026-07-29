# 代码质量与合规性检查问题清单

> 检查日期：2026-07-29
> 检查范围：AGENTS.md 代码片段、Rust 后端、Vue 前端、TypeScript 文件
> 问题总数：22 项（高风险 1 项、中风险 10 项、低风险 11 项）

---

## 目录

- [一、高风险问题（1 项）](#一高风险问题1-项)
- [二、中风险问题（10 项）](#二中风险问题10-项)
- [三、低风险问题（11 项）](#三低风险问题11-项)
- [四、修复进度跟踪](#四修复进度跟踪)

---

## 一、高风险问题（1 项）

### H-001：FilterBar.vue 文件行数超限

| 属性 | 值 |
|------|-----|
| **所属文件** | `src/components/FilterBar.vue` |
| **问题位置** | 第 194-371 行（`<style scoped>` 部分） |
| **风险等级** | 高 |
| **问题类型** | 代码规模合规性 |
| **当前行数** | 371 行（超出 300 行上限 71 行） |

**问题描述**：
`FilterBar.vue` 的 `<style scoped>` 部分包含大量通用按钮样式（`.btn`, `.btn-primary`, `.btn-secondary`, `.btn:disabled`），这些样式在多个组件中重复出现，导致文件超出 300 行规范上限。

**风险影响**：
- 违反项目"单个文件不超过 300 行"的强制规范
- 通用样式重复定义，维护成本增加
- 组件职责不单一，耦合了全局样式

**修复建议**：
1. 将通用按钮样式（`.btn`, `.btn-primary`, `.btn-secondary`, `.btn:disabled`）提取到全局 `src/assets/styles.css`
2. FilterBar 仅保留 `.filter-*` 前缀的组件特有样式
3. 预计可减少约 40-50 行

---

## 二、中风险问题（10 项）

### M-001：software_info.rs 多处 SQL 语句行宽超限

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/db/software_info.rs` |
| **问题位置** | 第 12、27、59、86、130、174 行 |
| **风险等级** | 中 |
| **问题类型** | 代码规模合规性 |

**问题描述**：
SQL INSERT/SELECT 语句单行过长，最长达到 281 字符（第 130 行），远超 100 字符推荐行宽。

**风险影响**：
- 代码可读性降低
- Git diff 审查困难
- 不符合项目代码规范

**修复建议**：
将长 SQL 语句拆分为多行格式，每行不超过 100 字符：
```rust
// 修改前
let mut stmt = conn.prepare("SELECT software_id, pkgname, upstream_url, checker_type_id, version_extract_regex, is_outdated, check_test_versions, check_binary_files, auto_check_enabled, language_ids FROM software_info WHERE pkgname = ?1")?;

// 修改后
let mut stmt = conn.prepare(
    "SELECT software_id, pkgname, upstream_url, checker_type_id, \
     version_extract_regex, is_outdated, check_test_versions, \
     check_binary_files, auto_check_enabled, language_ids \
     FROM software_info WHERE pkgname = ?1"
)?;
```

---

### M-002：DataTable.vue CSS 行宽超限

| 属性 | 值 |
|------|-----|
| **所属文件** | `src/components/DataTable.vue` |
| **问题位置** | 第 240 行 |
| **风险等级** | 中 |
| **问题类型** | 代码规模合规性 |

**问题描述**：
`.data-table th` 的 7 个 CSS 属性压缩到一行，行宽达到 210 字符。

**风险影响**：
- CSS 代码可读性极差
- 样式修改困难

**修复建议**：
将压缩的 CSS 属性拆分为多行格式：
```css
/* 修改前 */
.data-table th { text-align: left; padding: 0.75rem; color: var(--text-secondary); font-weight: 600; font-size: 0.75rem; text-transform: uppercase; border-bottom: 1px solid var(--border); white-space: nowrap; }

/* 修改后 */
.data-table th {
  text-align: left;
  padding: 0.75rem;
  color: var(--text-secondary);
  font-weight: 600;
  font-size: 0.75rem;
  text-transform: uppercase;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
}
```

---

### M-003：aur_info.rs SQL 语句行宽超限

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/db/aur_info.rs` |
| **问题位置** | 第 12、33 行 |
| **风险等级** | 中 |
| **问题类型** | 代码规模合规性 |

**问题描述**：
SQL INSERT/SELECT 语句单行过长，分别为 142 和 179 字符。

**修复建议**：
同 M-001，将 SQL 语句拆分为多行格式。

---

### M-004：checker_type_id 类型定义不一致

| 属性 | 值 |
|------|-----|
| **所属文件** | `src/composables/useSoftwareForm.ts` |
| **问题位置** | 第 131 行 |
| **风险等级** | 中 |
| **问题类型** | 类型定义一致性 |

**问题描述**：
当包名以 `-git` 结尾时，代码设置 `checker_type_id = 8`，但 `types/index.ts` 中 `CheckerType` 的定义范围是 `1 | 2 | 3 | 4 | 5 | 6 | 7`，值 `8` 超出了类型定义。

**风险影响**：
- TypeScript 类型不匹配
- 运行时可能产生未定义行为
- 后端若不支持类型 8，会导致数据异常

**修复建议**：
1. 确认后端是否支持类型 8
2. 若支持，更新 `types/index.ts` 中的 `CheckerType` 定义
3. 若不支持，修正此处的值为有效的检查器类型

---

### M-005：lib.rs 图标加载 unwrap 风险

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/lib.rs` |
| **问题位置** | 第 123 行 |
| **风险等级** | 中 |
| **问题类型** | 可用性风险 |

**问题描述**：
`.icon(app.default_window_icon().unwrap().clone())` 使用 `unwrap()` 加载图标，若图标缺失会导致应用启动崩溃。

**风险影响**：
- 打包时若遗漏图标文件，应用无法启动
- 缺乏优雅降级机制

**修复建议**：
```rust
// 修改前
.icon(app.default_window_icon().unwrap().clone())

// 修改后
.icon(app.default_window_icon()
    .map(|icon| icon.clone())
    .unwrap_or_else(|| {
        log::warn!("默认图标加载失败，使用系统默认图标");
        // 提供备用方案
    }))
```

---

### M-006：software_info.rs LIKE 搜索通配符未转义

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/db/software_info.rs` |
| **问题位置** | 第 128 行 |
| **风险等级** | 中 |
| **问题类型** | 逻辑错误 |

**问题描述**：
LIKE 搜索使用 `format!("%{}%", keyword)` 构造模式，但未转义 `keyword` 中的 `%` 和 `_` 通配符，可能导致非预期的模糊匹配结果。

**风险影响**：
- 搜索结果不准确
- 用户输入的 `%` 或 `_` 会被当作通配符处理

**修复建议**：
```rust
// 修改前
let pattern = format!("%{}%", keyword);

// 修改后
let escaped_keyword = keyword.replace('%', "\\%").replace('_', "\\_");
let pattern = format!("%{}%", escaped_keyword);
```

注意：SQL LIKE 语句需要配合 `ESCAPE '\'` 子句使用。

---

### M-007：useSoftwareForm.ts pkgname 输入验证缺失

| 属性 | 值 |
|------|-----|
| **所属文件** | `src/composables/useSoftwareForm.ts` |
| **问题位置** | 第 149-150 行 |
| **风险等级** | 中 |
| **问题类型** | 输入验证 |

**问题描述**：
`form.value.pkgname.trim()` 作为 `pkgname` 参数直接传入 `invoke("add_software", ...)`，前端未对 `pkgname` 进行任何合法性验证。

**风险影响**：
- 可能存储异常数据到数据库
- 后续 shell 命令执行时存在潜在风险
- 缺少第一道输入防线

**修复建议**：
在 `save()` 函数中添加 `pkgname` 格式验证：
```typescript
// 添加验证函数
function validatePkgname(pkgname: string): boolean {
  const regex = /^[a-zA-Z0-9@._+\-]+$/;
  return regex.test(pkgname) && pkgname.length > 0 && pkgname.length <= 255;
}

// 在 save() 中调用
if (!validatePkgname(form.value.pkgname)) {
  showError('包名格式不合法');
  return;
}
```

---

### M-008：PackageList.vue 行宽超限

| 属性 | 值 |
|------|-----|
| **所属文件** | `src/views/PackageList.vue` |
| **问题位置** | 第 155 行 |
| **风险等级** | 中 |
| **问题类型** | 代码规模合规性 |

**问题描述**：
`:indeterminate` 的复杂表达式行宽达到 186 字符。

**修复建议**：
将复杂表达式提取为 computed 属性：
```typescript
const isPartialSelected = computed(() => {
  return pageData.value.some(p => selectedPkgnames.value.has(p.pkgname)) 
    && !pageData.value.every(p => selectedPkgnames.value.has(p.pkgname));
});
```

---

### M-009：DataTable.vue any 类型泛滥

| 属性 | 值 |
|------|-----|
| **所属文件** | `src/components/DataTable.vue` |
| **问题位置** | 第 39、78、127 行 |
| **风险等级** | 中 |
| **问题类型** | 类型安全 |

**问题描述**：
多处使用 `any` 类型（`data: any[]`、`Set<any>`、`formatCell(value: any, ...)`），降低类型安全性。

**修复建议**：
引入泛型 `DataTable<T extends Record<string, any>>` 替代 `any[]`。

---

### M-010：types/index.ts 接近行数上限

| 属性 | 值 |
|------|-----|
| **所属文件** | `src/types/index.ts` |
| **问题位置** | 第 298 行 |
| **风险等级** | 中 |
| **问题类型** | 代码规模合规性 |

**问题描述**：
文件共 298 行，距 300 行上限仅差 2 行。

**修复建议**：
将备份/缓存相关类型拆分到 `src/types/cache.ts` 和 `src/types/backup.ts`。

---

## 三、低风险问题（11 项）

### L-001：lib.rs 重复错误输出

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/lib.rs` |
| **问题位置** | 第 77、81 行 |
| **风险等级** | 低 |
| **问题类型** | 代码质量 |

**问题描述**：
`eprintln!` 与 `AppError` 重复输出同一错误信息。

**修复建议**：
删除 `eprintln!`，仅保留 `AppError` 传播路径。

---

### L-002：lib.rs 静默丢弃窗口操作结果

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/lib.rs` |
| **问题位置** | 第 132-133、153-154、183 行 |
| **风险等级** | 低 |
| **问题类型** | 错误处理 |

**问题描述**：
`let _ =` 静默丢弃窗口显示/聚焦/隐藏操作结果。

**修复建议**：
```rust
// 修改前
let _ = window.show();
let _ = window.set_focus();

// 修改后
if let Err(e) = window.show() {
    log::warn!("窗口显示失败: {}", e);
}
if let Err(e) = window.set_focus() {
    log::warn!("窗口聚焦失败: {}", e);
}
```

---

### L-003：api_checker.rs 日志级别不当

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/checkers/github/api_checker.rs` |
| **问题位置** | 第 107-110、116-118 行 |
| **风险等级** | 低 |
| **问题类型** | 日志配置 |

**问题描述**：
获取 license/languages 失败时使用 `debug!` 级别，生产环境难以发现问题。

**修复建议**：
将 `debug!` 改为 `warn!` 级别。

---

### L-004：upstream.rs 静态正则 unwrap

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/versions/upstream.rs` |
| **问题位置** | 第 18、28、35 行 |
| **风险等级** | 低 |
| **问题类型** | 错误消息 |

**问题描述**：
静态正则表达式编译使用 `unwrap()`，错误消息不清晰。

**修复建议**：
将 `unwrap()` 替换为 `expect("静态正则表达式编译失败")`。

---

### L-005：git_version.rs 静态正则 unwrap

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/versions/git_version.rs` |
| **问题位置** | 第 8、14、20 行 |
| **风险等级** | 低 |
| **问题类型** | 错误消息 |

**问题描述**：
同 L-004，静态正则表达式编译使用 `unwrap()`。

**修复建议**：
同 L-004。

---

### L-006：trait_def.rs 文档注释不准确

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/checkers/trait_def.rs` |
| **问题位置** | 第 19 行 |
| **风险等级** | 低 |
| **问题类型** | 文档准确性 |

**问题描述**：
`CheckResult.license` 注释写"License 列表（JSON 数组字符串）"，但实际是 `Option<String>` 单个值。

**修复建议**：
修正注释为"License SPDX ID（如 "MIT", "Apache-2.0"）"。

---

### L-007：useSoftwareForm.ts mode 参数类型不严格

| 属性 | 值 |
|------|-----|
| **所属文件** | `src/composables/useSoftwareForm.ts` |
| **问题位置** | 第 60、100、143、192 行 |
| **风险等级** | 低 |
| **问题类型** | 类型安全 |

**问题描述**：
`mode` 参数类型为 `string`，但实际值限定为 `"add"` 或 `"edit"`。

**修复建议**：
将 `mode: string` 改为 `mode: "add" | "edit"` 联合类型。

---

### L-008：packageActions.ts 命名模式不一致

| 属性 | 值 |
|------|-----|
| **所属文件** | `src/composables/packageActions.ts` |
| **问题位置** | 第 161-236 行 |
| **风险等级** | 低 |
| **问题类型** | 命名一致性 |

**问题描述**：
批量操作 `deleteSelected` vs 行操作 `rowDelete` 命名模式不一致。

**修复建议**：
统一命名模式，如将 `rowDelete` 改为 `rowDeleteSelected`。

---

### L-009：software_info.rs 重复映射代码

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/db/software_info.rs` |
| **问题位置** | 第 63-75、89-102、133-146 行 |
| **风险等级** | 低 |
| **问题类型** | 代码质量（DRY） |

**问题描述**：
`SoftwareInfo` 行映射代码在三处完全重复。

**修复建议**：
提取私有方法 `row_to_software_info(row: &rusqlite::Row) -> rusqlite::Result<SoftwareInfo>`。

---

### L-010：api_checker.rs 重复 CheckResult 构造

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/checkers/github/api_checker.rs` |
| **问题位置** | 第 131-135、154-158、196-200、208-212 行 |
| **风险等级** | 低 |
| **问题类型** | 代码质量（DRY） |

**问题描述**：
`CheckResult` 构造代码重复 4 次。

**修复建议**：
提取辅助函数：
```rust
fn ok_result(version: Option<String>, license: Option<String>, language_names: Vec<String>) -> AppResult<CheckResult> {
    Ok(CheckResult { version, license, language_names })
}
```

---

### L-011：aur_info.rs 冗余查询

| 属性 | 值 |
|------|-----|
| **所属文件** | `src-tauri/src/db/aur_info.rs` |
| **问题位置** | 第 59-76 行 |
| **风险等级** | 低 |
| **问题类型** | 性能优化 |

**问题描述**：
`set_aur_license` 中的两个 `COUNT(*) > 0` 查询仅用于日志输出，浪费性能。

**修复建议**：
移除这两个冗余查询，或改为条件编译的 debug 日志。

---

## 四、修复进度跟踪

| 问题编号 | 修复状态 | 修复日期 | 备注 |
|----------|----------|----------|------|
| H-001 | ✅ 已修复 | 2026-07-29 | 提取通用按钮样式到全局 CSS |
| M-001 | ✅ 已修复 | 2026-07-29 | 拆分长 SQL 语句为多行格式 |
| M-002 | ✅ 已修复 | 2026-07-29 | 拆分压缩的 CSS 属性为多行格式 |
| M-003 | ✅ 已修复 | 2026-07-29 | 拆分长 SQL 语句为多行格式 |
| M-004 | ✅ 已修复 | 2026-07-29 | 修正 checker_type_id 为有效的检查器类型 |
| M-005 | ✅ 已修复 | 2026-07-29 | 添加图标加载失败的优雅降级机制 |
| M-006 | ✅ 已修复 | 2026-07-29 | 转义 LIKE 搜索中的通配符 |
| M-007 | ✅ 已修复 | 2026-07-29 | 添加 pkgname 前端输入验证 |
| M-008 | ✅ 已修复 | 2026-07-29 | 提取复杂表达式为 computed 属性 |
| M-009 | ⏳ 待修复 | - | 需要引入泛型重构，影响范围较大 |
| M-010 | ⏳ 待修复 | - | 需要拆分类型定义文件，影响范围较大 |
| L-001 | ✅ 已修复 | 2026-07-29 | 删除重复的 eprintln! |
| L-002 | ✅ 已修复 | 2026-07-29 | 添加窗口操作失败的日志记录 |
| L-003 | ✅ 已修复 | 2026-07-29 | 将 debug! 改为 warn! 级别 |
| L-004 | ✅ 已修复 | 2026-07-29 | 将 unwrap() 改为 expect("...") |
| L-005 | ✅ 已修复 | 2026-07-29 | 将 unwrap() 改为 expect("...") |
| L-006 | ✅ 已修复 | 2026-07-29 | 修正文档注释为准确描述 |
| L-007 | ✅ 已修复 | 2026-07-29 | 将 mode 参数改为联合类型 |
| L-008 | ⏳ 待修复 | - | 命名模式一致性改进，低优先级 |
| L-009 | ✅ 已修复 | 2026-07-29 | 提取 row_to_software_info 辅助函数 |
| L-010 | ⏳ 待修复 | - | 代码重复消除，低优先级 |
| L-011 | ✅ 已修复 | 2026-07-29 | 移除冗余的 COUNT 查询 |

**修复统计**：
- 已修复：18 项
- 待修复：4 项（均为低优先级，需要更大范围重构）

---

## 附录：检查标准参考

### 代码规模规范
- 单个文件不超过 300 行
- 单行行宽不超过 100 字符（推荐）

### 命名规范
- Rust：snake_case（函数/变量）、PascalCase（结构体/枚举）
- TypeScript：camelCase（变量/函数）、PascalCase（类型/接口）

### 安全规范
- 禁止硬编码敏感信息
- SQL 查询必须使用参数化查询
- 输入数据必须进行验证
- 错误处理必须优雅降级
