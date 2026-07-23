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
import type { FilterState } from "../composables/usePackageList";

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
  { key: "upstreamUrlEmpty" as const, label: "上游URL为空", description: "未配置上游仓库地址" },
  { key: "aurUpdateFailed" as const, label: "AUR更新失败", description: "未获取到AUR版本信息" },
  { key: "upstreamUpdateFailed" as const, label: "上游更新失败", description: "未获取到上游版本信息" },
  { key: "upstreamUrlAbnormal" as const, label: "上游地址异常", description: "上游URL验证不通过" },
  { key: "licenseMissing" as const, label: "License缺失", description: "未获取到上游License信息" },
];

const packageTypeOptions = [
  { value: null, label: "全部" },
  { value: 1, label: "编译安装" },
  { value: 2, label: "二进制包" },
  { value: 3, label: "Git仓库" },
  { value: 4, label: "AppImage" },
];

const checkerTypeOptions = [
  { value: null, label: "全部" },
  { value: 1, label: "GitHub Release" },
  { value: 2, label: "GitHub Tag" },
  { value: 3, label: "Gitee" },
  { value: 4, label: "GitLab" },
  { value: 5, label: "重定向" },
  { value: 6, label: "HTTP页面" },
  { value: 7, label: "手动" },
];

function toggleQuickFilter(key: keyof FilterState["quickFilters"]) {
  const newState = { ...props.filterState };
  newState.quickFilters = { ...newState.quickFilters };
  newState.quickFilters[key] = !newState.quickFilters[key];
  emit("update:filterState", newState);
}

function updateConditionFilter(key: "packageType" | "checkerType", value: number | null) {
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
            <span v-if="activeFilterCount > 0" class="filter-badge">{{ activeFilterCount }}</span>
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
                  @change="updateConditionFilter('packageType', ($event.target as HTMLSelectElement).value === '' ? null : Number(($event.target as HTMLSelectElement).value))"
                >
                  <option v-for="opt in packageTypeOptions" :key="opt.value ?? 'all'" :value="opt.value">
                    {{ opt.label }}
                  </option>
                </select>
              </div>
              <div class="filter-field">
                <label class="filter-field-label">检查器类型</label>
                <select
                  class="filter-select"
                  :value="filterState.conditionFilters.checkerType"
                  @change="updateConditionFilter('checkerType', ($event.target as HTMLSelectElement).value === '' ? null : Number(($event.target as HTMLSelectElement).value))"
                >
                  <option v-for="opt in checkerTypeOptions" :key="opt.value ?? 'all'" :value="opt.value">
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

<style scoped>
.filter-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 1000;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 80px;
}

.filter-panel {
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 12px;
  width: 340px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.filter-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}

.filter-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 0.875rem;
}

.filter-badge {
  background: var(--primary);
  color: white;
  font-size: 0.7rem;
  padding: 1px 6px;
  border-radius: 10px;
  min-width: 18px;
  text-align: center;
}

.filter-body {
  padding: 12px 16px;
  max-height: 400px;
  overflow-y: auto;
}

.filter-section {
  margin-bottom: 16px;
}

.filter-section:last-child {
  margin-bottom: 0;
}

.filter-section-title {
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-bottom: 8px;
  font-weight: 500;
}

.filter-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.filter-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.15s;
}

.filter-option:hover {
  background-color: var(--bg-secondary);
}

.filter-option input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: var(--primary);
  cursor: pointer;
}

.filter-option-label {
  font-size: 0.875rem;
  flex-shrink: 0;
}

.filter-option-desc {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.filter-row {
  display: flex;
  gap: 12px;
}

.filter-field {
  flex: 1;
}

.filter-field-label {
  display: block;
  font-size: 0.75rem;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.filter-select {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 0.875rem;
}

.filter-footer {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border);
  justify-content: flex-end;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border: none;
}

.btn-primary {
  background: var(--primary);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-secondary {
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border);
}

.btn-secondary:hover {
  background: var(--bg-tertiary);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
