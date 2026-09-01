# My-AUR-Helper 架构改进方案

## 问题诊断与改进方案

### 1. UI 布局问题：PackageList 缺少视图切换

#### 现状分析
- `PackageList.vue` 和 `PackageTable.vue` 仅支持表格视图
- 分页控件通过 `BottomToolbar` 共享，不支持自定义每页条数
- 列配置功能完全缺失

#### 改进方案

**新增文件：**
- `src/components/package/PackageListViewToggle.vue` - 视图切换组件
- `src/components/package/PackageCard.vue` - 卡片视图行组件
- `src/components/package/PackageGrid.vue` - 网格视图容器
- `src/components/common/ColumnConfigPopover.vue` - 列配置弹窗

**修改文件：**
- `src/components/package/PackageTable.vue` - 提取 `columns` 为 props，支持动态列配置
- `src/views/PackageList.vue` - 集成视图切换 + 分页控件内联化

**核心设计：**

```typescript
// PackageTable.vue 新增 props
const props = defineProps<{
  entries: any[];
  searchQuery: string;
  pageSize: number;
  currentPage: number;
  isRowLoading: (pkgname: string, action: string) => boolean;
  viewMode: "table" | "grid";           // 新增
  columns: Column[];                     // 改为 props，允许外部注入
  pageSizeOptions: number[];             // 新增，如 [20, 50, 100]
}>();
```

**视图切换实现：**
```vue
<!-- PackageListViewToggle.vue -->
<div class="view-toggle">
  <button :class="{ active: viewMode === 'table' }" @click="$emit('change', 'table')">
    <LucideList :size="16" />
  </button>
  <button :class="{ active: viewMode === 'grid' }" @click="$emit('change', 'grid')">
    <LucideLayoutGrid :size="16" />
  </button>
</div>
```

**PackageGrid.vue 卡片布局：**
```vue
<div class="package-grid">
  <PackageCard v-for="entry in entries" :key="entry.pkgname" :entry="entry" />
</div>
<style scoped>
.package-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 0.75rem;
}
</style>
```

**列配置存储：**
- 使用设置项 `column_visibility_software` 存储 JSON：`{"pkgname": true, "aur_version": false, ...}`
- 默认值由后端返回，前端持久化到 settings 表

---

### 2. 图标语义重复问题

#### 现状分析
| 图标令牌 | 实际图标 | 问题 |
|---------|---------|------|
| `navDashboard` | LayoutDashboard | ✅ 正确 |
| `navSettings` | Settings | ⚠️ 与 settingsGeneral 相同 |
| `settingsGeneral` | Settings | ⚠️ 导航与子页面用同一图标 |
| `settingsAur` | Globe | ⚠️ 与 navProxy 相同 |

#### 改进方案

**修改文件：** `src/icons/index.ts`

```typescript
export const Icon = {
  // 修改：settingsGeneral 使用 Cog（齿轮）而非 Settings
  settingsGeneral: Cog, // 通用设置
  // 修改：settingsAur 使用 Cloud（云端）而非 Globe
  settingsAur: Cloud,   // AUR 设置
  // 新增：布局切换图标
  viewList: List,       // 列表视图
  viewGrid: LayoutGrid, // 网格视图
};
```

**修改文件：** `src/views/PackageList.vue` - 工具栏加入视图切换按钮
```vue
<template #right>
  <!-- 视图切换 -->
  <button class="btn-icon" @click="viewMode = viewMode === 'table' ? 'grid' : 'table'" title="切换视图">
    <component :is="Icon.viewList" v-if="viewMode === 'grid'" :size="16" />
    <component :is="Icon.viewGrid" v-else :size="16" />
  </button>
  <!-- ... 其他按钮 -->
</template>
```

---

### 3. 设置页键名硬编码问题

#### 现状分析
- `Settings.vue` 使用 `categoryMap` 硬编码路由映射
- `SettingsDynamicSection.vue` 中的 `categoryLabels` 也是硬编码
- 无类型定义，全靠字符串比较

#### 改进方案

**新增文件：** `src/types/settings-meta.ts`

```typescript
/** 设置分类元数据 */
export interface SettingCategoryMeta {
  /** 分类 key（与路由 path 对应） */
  key: string;
  /** 显示标签 */
  label: string;
  /** 图标令牌 */
  icon: string;
  /** 描述 */
  description?: string;
}

/** 所有设置分类的元数据定义 */
export const SETTING_CATEGORIES: SettingCategoryMeta[] = [
  { key: "general", label: "通用设置", icon: "settingsGeneral", description: "主题、字体等外观设置" },
  { key: "list", label: "列表设置", icon: "settingsList", description: "软件包列表显示相关配置" },
  { key: "aur", label: "AUR 设置", icon: "settingsAur", description: "AUR 同步相关配置" },
  { key: "checker", label: "上游检查器", icon: "settingsChecker", description: "上游版本检查配置" },
  { key: "backup", label: "备份管理", icon: "settingsBackup", description: "备份目录与策略配置" },
  { key: "cache", label: "缓存目录", icon: "settingsCache", description: "缓存路径配置" },
  { key: "memory_cache", label: "内存缓存", icon: "settingsMemoryCache", description: "内存缓存行为配置" },
  { key: "proxy", label: "代理管理", icon: "settingsProxy", description: "网络代理配置" },
  { key: "log", label: "日志管理", icon: "settingsLog", description: "日志级别与输出配置" },
];

/** 按 key 查找分类元数据 */
export function getSettingCategory(key: string): SettingCategoryMeta | undefined {
  return SETTING_CATEGORIES.find((c) => c.key === key);
}
```

**修改文件：** `src/views/Settings.vue`

```typescript
import { computed } from "vue";
import { useRoute } from "vue-router";
import { SETTING_CATEGORIES, getSettingCategory } from "../types/settings-meta";
// ...

const route = useRoute();
const category = computed(() => getSettingCategory(route.path.replace("/settings", ""))?.key || "general");
```

**同时修改 `SettingsDynamicSection.vue`：**
```typescript
import { computed } from "vue";
import { getSettingCategory } from "../../types/settings-meta";

const categoryMeta = computed(() => getSettingCategory(props.category));
const title = computed(() => categoryMeta.value?.label || props.category);
```

---

### 4. 日志内存限制与高频事件防抖

#### 现状分析
```typescript
// LogViewer.vue:49
const MAX_LOG_ENTRIES = 1000;

// 每个日志事件直接 unshift，无防抖
logs.value.unshift({ ...payload, _id: nextId++ });
if (logs.value.length > MAX_LOG_ENTRIES) {
  logs.value.length = MAX_LOG_ENTRIES;
}
```

#### 改进方案

**修改文件：** `src/views/LogViewer.vue`

```typescript
// 新增：防抖缓冲
let logBuffer: LogEntry[] = [];
let bufferTimer: ReturnType<typeof setTimeout> | null = null;
const DEBOUNCE_MS = 100; // 100ms 批量合并

/** 监听 Tauri 日志事件（带防抖） */
async function startLogListener() {
  unlisten = await listen("log-entry", (event) => {
    const payload = event.payload as LogEntry;
    if (payload) {
      logBuffer.push({ ...payload, _id: nextId++ });

      // 防抖：100ms 内合并，然后一次性批量插入
      if (bufferTimer) clearTimeout(bufferTimer);
      bufferTimer = setTimeout(flushLogBuffer, DEBOUNCE_MS);
    }
  });
}

/** 将缓冲区日志批量 flush 到 logs */
function flushLogBuffer() {
  if (logBuffer.length === 0) return;

  const batch = logBuffer;
  logBuffer = [];

  // 批量 unshift（保持倒序）
  logs.value = [...batch.reverse(), ...logs.value];

  // 限制内存
  if (logs.value.length > MAX_LOG_ENTRIES) {
    logs.value = logs.value.slice(0, MAX_LOG_ENTRIES);
  }
}
```

**同时优化 watch：**
```typescript
// 原代码：监听 logs.value.length（每次 unshift 都触发）
watch([levelFilter, searchQuery, () => logs.value.length], () => {
  syncFooter();
});

// 优化：只监听过滤条件变化，不监听日志数量
watch([levelFilter, searchQuery], () => {
  syncFooter();
});
```

**新增设置项：** `log_max_entries`（默认 1000，可配置）

---

### 5. 启动延迟问题：并行化初始化

#### 现状分析

**useCacheManagerInit.ts（串行）：**
```typescript
// 当前：5 个异步操作串行执行
await deps.loadEntries();          // 任务 1
deps.setSourceDirs(await ...);     // 任务 2
const setting = await ...;         // 任务 3
backupSubdirectories.value = await... // 任务 4
await deps.checkSudoers();         // 任务 5
```

**packages.ts store（冗余加载）：**
- `fetchPackages()` 调用 `list_software` 全量加载（1932 条）
- 但 `PackageList.vue` 从未使用 packages store，纯属冗余开销

#### 改进方案

**修改文件：** `src/composables/useCacheManagerInit.ts`

```typescript
onMounted(async () => {
  // 并行执行所有独立任务
  await Promise.allSettled([
    deps.loadEntries().catch(e => {
      console.error("加载缓存数据失败:", e);
      addMessage(footer, "error", `加载缓存数据失败: ${e}`);
    }),
    (async () => {
      deps.setSourceDirs(await loadEnabledCacheDirs());
    })(),
    (async () => {
      try {
        const setting = await settingsApi.getSetting("backup_dir");
        if (setting) backupPath.value = setting.value;
      } catch (e) {
        console.error("加载备份目录设置失败:", e);
      }
    })(),
    (async () => {
      try {
        backupSubdirectories.value = await backupApi.listBackupSubdirectories();
      } catch (e) {
        console.error("加载备份子目录失败:", e);
      }
    })(),
    deps.checkSudoers().catch(e => {
      console.error("检查 sudoers 配置失败:", e);
    }),
  ]);
});
```

**删除冗余代码：**
- 移除 `packages.ts` 中的 `fetchPackages()` 调用
- `PackageList.vue` 已注释说明不调用此方法，确认无需修改
