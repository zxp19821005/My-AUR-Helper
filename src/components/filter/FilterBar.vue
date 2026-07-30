<!--
  FilterBar.vue - 筛选器组件

  功能：
  - 提供快速筛选条件（OR 逻辑）：上游URL为空、AUR更新失败、上游更新失败、上游地址异常、License缺失
  - 提供条件筛选（AND 逻辑）：软件包类型、检查器类型
  - 筛选器以弹出面板形式展示，默认折叠
  - 通过 badge 显示活跃筛选条件数量
-->
<script setup lang="ts">
import { Filter, X, Search } from "@lucide/vue";
import type { FilterState } from "../../composables/usePackageList";
import { packageTypeFilterOptions, checkerTypeFilterOptions } from "../../utils/enums";

const props = defineProps<{
  filterState: FilterState;
  show: boolean;
  activeFilterCount: number;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
  (e: "update:filterState", value: FilterState): void;
  (e: "validate-urls"): void;
  (e: "reset-filters"): void;
}>();

const quickFilterOptions = [
  {
    key: "upstreamUrlEmpty" as const,
    label: "上游URL为空",
    description: "未配置上游仓库地址",
  },
  {
    key: "aurUpdateFailed" as const,
    label: "AUR更新失败",
    description: "未获取到AUR版本信息",
  },
  {
    key: "upstreamUpdateFailed" as const,
    label: "上游更新失败",
    description: "未获取到上游版本信息",
  },
  {
    key: "upstreamUrlAbnormal" as const,
    label: "上游地址异常",
    description: "上游URL验证不通过",
  },
  {
    key: "licenseMissing" as const,
    label: "License缺失",
    description: "未获取到上游License信息",
  },
];

function parseSelectValue(event: Event): number | null {
  const value = (event.target as HTMLSelectElement).value;
  return value === "" ? null : Number(value);
}

function toggleQuickFilter(key: keyof FilterState["quickFilters"]) {
  const newState = { ...props.filterState };
  newState.quickFilters = { ...newState.quickFilters };
  newState.quickFilters[key] = !newState.quickFilters[key];
  emit("update:filterState", newState);
}

function updateConditionFilter(
  key: "packageType" | "checkerType",
  value: number | null
) {
  const newState = { ...props.filterState };
  newState.conditionFilters = { ...newState.conditionFilters };
  newState.conditionFilters[key] = value;
  emit("update:filterState", newState);
}

function handleClose() {
  emit("update:show", false);
}

function handleReset() {
  emit("reset-filters");
}

function handleValidateUrls() {
  emit("validate-urls");
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="filter-overlay" @click.self="handleClose">
      <div class="filter-panel">
        <div class="filter-header">
          <div class="filter-title">
            <Filter :size="16" />
            <span>筛选条件</span>
            <span v-if="activeFilterCount > 0" class="filter-badge">
              {{ activeFilterCount }}
            </span>
          </div>
          <button class="btn-icon btn-icon-default" @click="handleClose">
            <X :size="16" />
          </button>
        </div>

        <div class="filter-body">
          <!-- 快速筛选 -->
          <div class="filter-section">
            <div class="filter-section-title">快速筛选（满足任一条件）</div>
            <div class="filter-options">
              <label
                v-for="opt in quickFilterOptions"
                :key="opt.key"
                class="filter-option"
              >
                <input
                  type="checkbox"
                  :checked="filterState.quickFilters[opt.key]"
                  @change="toggleQuickFilter(opt.key)"
                />
                <span class="filter-option-label">{{ opt.label }}</span>
                <span class="filter-option-desc">{{ opt.description }}</span>
              </label>
            </div>
          </div>

          <!-- 条件筛选 -->
          <div class="filter-section">
            <div class="filter-section-title">条件筛选（满足所有条件）</div>
            <div class="filter-row">
              <div class="filter-field">
                <label class="filter-field-label">软件包类型</label>
                <select
                  class="filter-select"
                  :value="filterState.conditionFilters.packageType"
                  @change="updateConditionFilter('packageType', parseSelectValue($event))"
                >
                  <option
                    v-for="opt in packageTypeFilterOptions"
                    :key="opt.value ?? 'all'"
                    :value="opt.value"
                  >
                    {{ opt.label }}
                  </option>
                </select>
              </div>
              <div class="filter-field">
                <label class="filter-field-label">检查器类型</label>
                <select
                  class="filter-select"
                  :value="filterState.conditionFilters.checkerType"
                  @change="updateConditionFilter('checkerType', parseSelectValue($event))"
                >
                  <option
                    v-for="opt in checkerTypeFilterOptions"
                    :key="opt.value ?? 'all'"
                    :value="opt.value"
                  >
                    {{ opt.label }}
                  </option>
                </select>
              </div>
            </div>
          </div>
        </div>

        <div class="filter-footer">
          <button class="btn btn-secondary" @click="handleReset">
            <X :size="14" />
            清空筛选
          </button>
          <button class="btn btn-primary" @click="handleValidateUrls" :disabled="loading">
            <Search :size="14" />
            校验上游URL
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<!-- 样式集中在 src/assets/styles/filter-styles.css -->

