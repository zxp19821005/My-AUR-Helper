# 工具栏三段式布局改造

日期：2026-09-01
范围：My-AUR-Helper 前端 `PageToolbar` 系工具栏（7 个页面）

## 需求

工具栏统一为从左到右的固定顺序：

```
操作按钮(最左侧) | 分隔符 | 筛选控件(中间·靠左) | 分隔符 | 搜索框(最右侧)
```

## 改动清单

### 1. `src/components/common/PageToolbar.vue`（核心）

| 项 | 改前 | 改后 |
|----|------|------|
| 父容器间距 | `gap: 0.75rem` | `gap: 0.5rem`（配合分隔符自身外边距，视觉间距保持 0.75rem） |
| 中间区 `.toolbar-center` | `flex: 1` + `max-width: 400px` | `flex: 1 1 auto`，**移除 max-width**，新增 `justify-content: flex-start` |
| 右侧区 `.toolbar-right` | `flex: 0 0 auto; min-width: 200px; max-width: 320px` | `flex: 0 1 auto; width: 260px; min-width: 160px; max-width: 320px; margin-left: auto` |
| 分隔符 | 仅刷新按钮前有一个 | 新增两个条件渲染分隔符（按钮组⇄筛选区、筛选区⇄搜索框） |

分隔符渲染规则（通过 `useSlots()` 探测插槽填充情况）：

- `showLeadingDivider = hasActions && hasFilters`
- `showTrailingDivider = hasFilters`

即只有相邻两区域都非空才渲染竖线，避免出现孤立分隔符。

刷新按钮：**保持原样**（`showRefreshButton` 默认 false，刷新由父级 `TabBar` 统一提供，见 `TabBar.vue:103`），不纳入本次布局调整。

### 2. `src/views/PackageList.vue`

删除 scoped 内重复的 `.toolbar-filter-select` 定义（与全局 `assets/styles/filter-styles.css:151` 重复，且缺 `height: 32px` / `min-width: 110px`，导致该页下拉框高度与其他页不一致）。统一走全局样式。

### 3. 关闭孤立筛选按钮（3 个页面）

`Dashboard` / `LicenseManager` / `LanguageManager` 未挂 `FilterBar`、无 `@toggle-filter` 处理函数，但 `showFilterButton` 默认 true，导致中间区渲染一个点击无响应的按钮。已显式传 `:show-filter-button="false"`。

## 各页面最终布局

| 页面 | 左侧按钮组 | 中间筛选区 | 右侧 |
|------|-----------|-----------|------|
| 软件包 PackageList | 6 个彩色操作按钮 | 2 个下拉 + 分隔符 + 视图切换 + 折叠筛选按钮（带计数徽标） | 搜索框 |
| 缓存 CacheManager | 7 个操作按钮 | 2 个下拉（缓存目录 / 架构） | 搜索框 |
| 备份 BackupManager | 5 个操作按钮 | 2 个下拉（子目录 / 架构） | 搜索框 |
| 代理 ProxySettings | 5 个操作按钮 | 1 个下拉（代理类型） | 搜索框 |
| License / 编程语言 | 1–2 个操作按钮 | 无（分隔符随之隐藏） | 搜索框 |
| 仪表盘 Dashboard | 无 | 无 | 搜索框 |

## 验证

- `npx vue-tsc --noEmit` → 0 错误
- `npx vite build` → 成功（2.72s）
