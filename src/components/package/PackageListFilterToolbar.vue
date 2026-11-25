<!--
  PackageListFilterToolbar.vue - 软件包列表页「筛选 / 视图切换区」

  功能：渲染工具栏中间的两个条件下拉（软件包类型 / 检查器类型）、单选快速筛选
        下拉（QuickFilterDropdown）、分隔线以及列表 / 块视图切换按钮。
  工作流程：纯展示组件，筛选状态与下拉选项全部由父组件通过 props 传入；
        条件下拉的 Event 在组件内解析成 number|null 后 emit，快速筛选与视图切换
        仅 emit 语义事件，状态写回仍由父组件负责。组件不 import 任何
        composable / store（筛选类型以等价结构类型内联声明），与阶段 B 的
        composable 重命名彻底解耦。
  适用场景：PageToolbar 的 #filters 插槽；由父组件透传 usePackageList 与
        viewModeStore 的状态，保证列表 / 块视图与筛选行为与拆分前一致。
-->
<script setup lang="ts">
import QuickFilterDropdown from "./QuickFilterDropdown.vue";
import { Icon } from "@/icons";
import type { SelectOption } from "@/utils/enums";

/** 条件筛选键（软件包类型 / 检查器类型） */
type ConditionFilterKey = "packageType" | "checkerType";

/** 条件筛选值，与父组件 filterState.conditionFilters 结构一致 */
interface ConditionFilters {
  packageType: number | null;
  checkerType: number | null;
}

/**
 * 单选快速筛选条件键。
 * 与 filterConfig.ts / QuickFilterDropdown.vue 中的 QuickFilterKey 为等价的
 * 字符串字面量联合（TS 结构类型互相兼容），此处内联以避免引入 composable 依赖。
 */
type QuickFilterKey =
  | "aur_sync_failed"
  | "upstream_check_failed"
  | "github_url_invalid"
  | "no_aur_version"
  | "no_upstream_version"
  | "no_upstream_url"
  | "license_missing";

/** 视图模式，与 viewMode.ts 的 ViewMode 结构一致 */
type ViewMode = "list" | "grid";

const props = defineProps<{
  /** 当前条件筛选值，绑定到父组件 filterState.conditionFilters */
  conditionFilters: ConditionFilters;
  /** 软件包类型下拉选项 */
  packageTypeOptions: SelectOption<number | null>[];
  /** 检查器类型下拉选项 */
  checkerTypeOptions: SelectOption<number | null>[];
  /** 单选快速筛选下拉选项 */
  quickFilterOptions: { key: QuickFilterKey; label: string }[];
  /** 当前激活的单选快速筛选键 */
  quickFilterKey: QuickFilterKey | null;
  /** 当前激活的单选快速筛选标签 */
  quickFilterLabel: string;
  /** 当前视图模式 */
  viewMode: ViewMode;
}>();

const emit = defineEmits<{
  (e: "update-condition-filter", key: ConditionFilterKey, value: number | null): void;
  (e: "select-quick-filter", key: QuickFilterKey): void;
  (e: "clear-quick-filter"): void;
  (e: "set-view-mode", mode: ViewMode): void;
}>();

/** 解析下拉框值为条件筛选值（空字符串表示"全部" → null） */
function parseSelectValue(event: Event): number | null {
  const value = (event.target as HTMLSelectElement).value;
  return value === "" ? null : Number(value);
}

/** 转发单选快速筛选下拉的选中事件 */
function onQuickFilterSelect(key: QuickFilterKey): void {
  emit("select-quick-filter", key);
}
</script>

<template>
  <div class="toolbar-filter-group">
    <select
      class="toolbar-filter-select"
      :value="props.conditionFilters.packageType === null ? '' : props.conditionFilters.packageType"
      @change="emit('update-condition-filter', 'packageType', parseSelectValue($event))"
    >
      <option
        v-for="opt in props.packageTypeOptions"
        :key="opt.value ?? 'all'"
        :value="opt.value === null ? '' : opt.value"
      >
        {{ opt.value === null ? '全部软件包' : opt.label }}
      </option>
    </select>
    <select
      class="toolbar-filter-select"
      :value="props.conditionFilters.checkerType === null ? '' : props.conditionFilters.checkerType"
      @change="emit('update-condition-filter', 'checkerType', parseSelectValue($event))"
    >
      <option
        v-for="opt in props.checkerTypeOptions"
        :key="opt.value ?? 'all'"
        :value="opt.value === null ? '' : opt.value"
      >
        {{ opt.value === null ? '全部检查器' : opt.label }}
      </option>
    </select>

    <QuickFilterDropdown
      :options="props.quickFilterOptions"
      :active-key="props.quickFilterKey"
      :active-label="props.quickFilterLabel"
      @select="onQuickFilterSelect"
      @clear="emit('clear-quick-filter')"
    />

    <div class="toolbar-divider"></div>
    <button
      class="btn-icon"
      :class="props.viewMode === 'list' ? 'btn-active' : ''"
      @click="emit('set-view-mode', 'list')"
      title="列表视图"
    >
      <component :is="Icon.listView" :size="16" />
    </button>
    <button
      class="btn-icon"
      :class="props.viewMode === 'grid' ? 'btn-active' : ''"
      @click="emit('set-view-mode', 'grid')"
      title="块视图"
    >
      <component :is="Icon.gridView" :size="16" />
    </button>
  </div>
</template>

<style scoped>
/*
 * 本区块样式原位于 PackageList.css（scoped，作用域为 PackageList.vue）。
 * 抽为子组件后内部元素改为子组件作用域，故在此自带等价 scoped 样式保持视觉一致。
 * .toolbar-filter-select 由全局 filter-styles.css 提供；QuickFilterDropdown 自带样式。
 */
.toolbar-filter-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background-color: var(--border);
  margin: 0 0.375rem;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.375rem;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-icon:hover:not(:disabled) {
  background-color: var(--bg-card);
  color: var(--text-primary);
}

.btn-active {
  background-color: var(--accent-light, rgba(59, 130, 246, 0.1));
  color: var(--accent);
}
</style>
