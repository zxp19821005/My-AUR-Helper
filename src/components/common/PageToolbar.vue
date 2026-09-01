<!--
  PageToolbar.vue - 页面统一工具栏（三段式布局）

  布局规范（从左到右）：
    操作按钮(最左) | 分隔符 | 筛选控件(中间·靠左) | 分隔符 | 搜索框(最右)

  区域职责：
  - 左（默认插槽）  ：页面级操作按钮，不伸缩，始终贴最左侧
  - 中（filters 插槽 + 折叠筛选按钮）：占据全部剩余空间，但内容靠左紧邻按钮组
  - 右（内置）      ：搜索框，margin-left:auto 兜底钉在最右侧

  分隔符规则：仅在相邻两区域均有内容时渲染，避免出现孤立竖线。
  刷新按钮归属于父级 TabBar，本工具栏默认不渲染（showRefreshButton 默认 false）。

  适用场景：软件包 / 缓存 / 备份 / 代理 / License / 编程语言 / 仪表盘等列表页顶部工具栏
-->
<script setup lang="ts">
import { computed, ref, useSlots, watch } from "vue";
import { Icon } from "../../icons";

const props = defineProps<{
  modelValue?: string;
  filterActive?: boolean;
  /** 是否显示折叠筛选按钮（当所有筛选已内联到工具栏时为 false） */
  showFilterButton?: boolean;
  /** 是否显示内置刷新按钮（默认 false，由父级 TabBar 统一控制） */
  showRefreshButton?: boolean;
}>();

const slots = useSlots();
const showFilterButton = props.showFilterButton ?? true;
const showRefreshButton = props.showRefreshButton ?? false;

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "refresh"): void;
  (e: "toggle-filter"): void;
}>();

const searchText = ref(props.modelValue || "");
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(searchText, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    emit("update:modelValue", val);
  }, 500);
});

watch(
  () => props.modelValue,
  (val) => {
    if (val !== searchText.value) {
      searchText.value = val || "";
    }
  }
);

function clearSearch() {
  searchText.value = "";
  if (debounceTimer) clearTimeout(debounceTimer);
  emit("update:modelValue", "");
}

/** 左侧是否存在操作按钮（默认插槽被填充即为真） */
const hasActions = computed(() => Boolean(slots.default));

/** 中间是否存在筛选控件（内联下拉框 / 折叠筛选按钮 / 刷新按钮任一） */
const hasFilters = computed(
  () => Boolean(slots.filters) || showFilterButton || showRefreshButton
);

/** 按钮组与筛选区之间的分隔符：两侧都非空才渲染 */
const showLeadingDivider = computed(() => hasActions.value && hasFilters.value);

/** 筛选区与搜索框之间的分隔符：筛选区非空才渲染（搜索框恒存在） */
const showTrailingDivider = computed(() => hasFilters.value);
</script>

<template>
  <div class="page-toolbar">
    <!-- 1. 最左：操作按钮组 -->
    <div class="toolbar-left">
      <slot />
    </div>

    <!-- 2. 分隔符：按钮组 ⇄ 筛选区 -->
    <div v-if="showLeadingDivider" class="toolbar-divider"></div>

    <!-- 3. 中间：筛选控件，内容靠左紧邻按钮组 -->
    <div class="toolbar-center">
      <slot name="filters" />
      <button
        v-if="showFilterButton"
        class="toolbar-icon-btn"
        :class="{ 'btn-icon-warning': filterActive }"
        @click="emit('toggle-filter')"
        title="筛选"
      >
        <slot name="filter-icon" />
      </button>
      <template v-if="showRefreshButton">
        <div class="toolbar-divider"></div>
        <button
          class="toolbar-icon-btn"
          @click="emit('refresh')"
          title="刷新数据"
        >
          <component :is="Icon.actionRefresh" :size="18" />
        </button>
      </template>
    </div>

    <!-- 4. 分隔符：筛选区 ⇄ 搜索框 -->
    <div v-if="showTrailingDivider" class="toolbar-divider"></div>

    <!-- 5. 最右：搜索框 -->
    <div class="toolbar-right">
      <div class="search-box">
        <component :is="Icon.actionSearch" :size="14" class="search-icon" />
        <input
          v-model="searchText"
          type="text"
          class="search-input"
          placeholder="搜索..."
        />
        <button
          v-if="searchText"
          class="search-clear-btn"
          @click="clearSearch"
          title="清除搜索"
        >
          <component :is="Icon.actionClear" :size="12" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page-toolbar {
  display: flex;
  align-items: center;
  padding: 0.5rem 1rem;
  border-bottom: 1px solid var(--border);
  background-color: var(--bg-primary);
  min-height: 48px;
  position: relative;
  z-index: 1001;
  gap: 0.5rem;
}

/* 1. 左侧按钮组：不伸缩，贴最左 */
.toolbar-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 0 0 auto;
}

/* 2. 中间筛选区：吃掉剩余空间，内容靠左紧邻按钮组 */
.toolbar-center {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1 1 auto;
  min-width: 0;
  justify-content: flex-start;
}

/* 3. 右侧搜索框：可压缩但不伸张，强制贴最右 */
.toolbar-right {
  display: flex;
  align-items: center;
  flex: 0 1 auto;
  width: 260px;
  min-width: 160px;
  max-width: 320px;
  margin-left: auto;
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background-color: var(--border);
  margin: 0 0.25rem;
  flex-shrink: 0;
}

.toolbar-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.375rem;
  border-radius: 6px;
  transition: all 0.15s;
  flex-shrink: 0;
}

.toolbar-icon-btn:hover {
  color: var(--text-primary);
  background-color: var(--bg-card);
}

.btn-icon-warning {
  color: var(--warning) !important;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.625rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--bg-card);
  transition: border-color 0.15s;
  width: 100%;
}

.search-box:focus-within {
  border-color: var(--accent);
}

.search-icon {
  color: var(--text-secondary);
  flex-shrink: 0;
}

.search-input {
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: 0.8125rem;
  outline: none;
  width: 100%;
  min-width: 0;
}

.search-input::placeholder {
  color: var(--text-muted);
}

.search-clear-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  flex-shrink: 0;
  transition: all 0.15s;
}

.search-clear-btn:hover {
  color: var(--text-primary);
  background-color: var(--bg-hover);
}
</style>
