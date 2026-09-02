<!--
  PackageList.vue - 软件包列表页面（优化版）

  功能：
  - 显示软件包列表（含 AUR 版本和上游版本）
  - 支持列表/块视图切换
  - 支持搜索、分页、多选
  - 提供批量操作：同步AUR、同步PKGBUILD、检查上游、删除
  - 支持单行操作：查看详情、编辑、同步、检查、删除
  - 支持筛选器：快速筛选（OR）+ 条件筛选（AND）
  - 支持自定义每页显示条数
-->
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { usePackageActions } from "../composables/packageActions";
import { useViewModeStore } from "../stores/viewMode";
import { useRefreshSubscription } from "../stores/refresh";
import * as softwareApi from "@/api/software";
import { usePackageList } from "../composables/usePackageList";
import PageToolbar from "../components/common/PageToolbar.vue";
import FilterBar from "../components/filter/FilterBar.vue";
import SoftwareFormModal from "../components/package/SoftwareFormModal.vue";
import SoftwareDetailModal from "../components/package/SoftwareDetailModal.vue";
import PackageTable from "../components/package/PackageTable.vue";
import BlockView from "../components/package/BlockView.vue";
import { Icon } from "../icons";
import { packageTypeFilterOptions, checkerTypeFilterOptions } from "../utils/enums";

const viewModeStore = useViewModeStore();

const {
  searchQuery,
  selectedPkgnames,
  filterState,
  showFilterBar,
  showModal,
  modalMode,
  modalPkgname,
  showDetailModal,
  detailPkgname,
  pageData,
  pageSize,
  currentPage,
  filteredEntries,
  fetchView,
  refreshEntries,
  openAddModal,
  openEditModal,
  openDetailModal,
  setSelected,
  syncToolbar,
  activeFilterCount,
  activeQuickFilterLabel,
  quickFilterOptions,
  selectQuickFilter,
  clearQuickFilter,
  resetFilters,
} = usePackageList();

const {
  loading,
  isRowLoading,
  batchStats,
  syncFromAur,
  syncFromPkgbuild,
  updateAurInfo,
  checkSelectedUpstream,
  deleteSelected,
  rowSyncFromAur,
  rowSyncFromPkgbuild,
  rowCheckUpstream,
  rowDeleteSelected,
} = usePackageActions(fetchView, refreshEntries, syncToolbar);

const validating = ref(false);

async function handleValidateUrls() {
  validating.value = true;
  try {
    const pkgnameList = pageData.value.map((p) => p.pkgname);
    await softwareApi.validateUpstreamUrls(
      pkgnameList.length > 0 ? pkgnameList : null,
    );
    await fetchView();
  } catch (error) {
    console.error("验证失败:", error);
  } finally {
    validating.value = false;
    showFilterBar.value = false;
  }
}

function handleFilterUpdate(newState: typeof filterState.value) {
  filterState.value = newState;
}

/** 解析下拉框值为条件筛选值（空字符串表示"全部" → null） */
function parseSelectValue(event: Event): number | null {
  const value = (event.target as HTMLSelectElement).value;
  return value === "" ? null : Number(value);
}

/** 更新条件筛选（软件包类型 / 检查器类型），绑定到共享 filterState */
function updateConditionFilter(key: "packageType" | "checkerType", value: number | null) {
  filterState.value = {
    ...filterState.value,
    conditionFilters: { ...filterState.value.conditionFilters, [key]: value },
  };
}

/** 工具栏快速筛选下拉：开关与单选选中（选中后自动收起菜单） */
const quickFilterOpen = ref(false);
function toggleQuickFilterMenu() {
  quickFilterOpen.value = !quickFilterOpen.value;
}
function selectAndClose(key: typeof quickFilterOptions[number]["key"]) {
  selectQuickFilter(key);
  quickFilterOpen.value = false;
}
function closeQuickFilterMenu() {
  quickFilterOpen.value = false;
}

onMounted(async () => {
  await fetchView();
});

/** 订阅 TabBar 刷新按钮的全局刷新事件 */
useRefreshSubscription(async () => {
  await fetchView();
});

async function handleFormSaved() {
  const pkgnames =
    modalMode.value === "edit" && modalPkgname.value
      ? [modalPkgname.value]
      : [];
  await refreshEntries(pkgnames);
}
</script>

<template>
  <div>
    <!-- 批量操作进度提示条 -->
    <div v-if="loading && batchStats.total > 0" class="batch-progress-bar">
      <div class="progress-track">
        <div class="progress-fill" :style="{ width: `${batchStats.total > 0 ? (batchStats.success + batchStats.failed) / batchStats.total * 100 : 0}%` }"></div>
      </div>
      <span class="progress-text">
        {{ batchStats.success }} 成功，{{ batchStats.failed }} 失败，共 {{ batchStats.total }} 个
      </span>
    </div>

    <PageToolbar
      v-model="searchQuery"
      :filter-active="activeFilterCount > 0"
      @toggle-filter="showFilterBar = !showFilterBar"
    >
      <!-- 左侧：常用工具按钮（彩色图标） -->
      <button class="btn-icon btn-color-aur" @click="syncFromAur(selectedPkgnames)" :disabled="loading" title="从AUR同步">
        <component :is="Icon.syncAur" :size="16" />
      </button>
      <button class="btn-icon btn-color-pkgbuild" @click="syncFromPkgbuild(selectedPkgnames)" :disabled="loading" title="从PKGBUILD同步">
        <component :is="Icon.syncPkgbuild" :size="16" />
      </button>
      <button class="btn-icon btn-color-add" @click="openAddModal" title="添加软件">
        <component :is="Icon.actionAdd" :size="16" />
      </button>
      <button class="btn-icon btn-color-update" @click="updateAurInfo(selectedPkgnames)" :disabled="loading" title="更新AUR信息">
        <component :is="Icon.actionRefresh" :size="16" />
      </button>
      <button class="btn-icon btn-color-check" @click="checkSelectedUpstream(selectedPkgnames)" :disabled="loading" title="检查上游">
        <component :is="Icon.actionSearch" :size="16" />
      </button>
      <button class="btn-icon btn-color-delete" @click="deleteSelected(selectedPkgnames, setSelected)" :disabled="selectedPkgnames.size === 0" title="删除选中">
        <component :is="Icon.actionDelete" :size="16" />
      </button>

      <!-- 中间：筛选器和视图切换 -->
      <template #filters>
        <div class="toolbar-filter-group">
          <select
            class="toolbar-filter-select"
            :value="filterState.conditionFilters.packageType === null ? '' : filterState.conditionFilters.packageType"
            @change="updateConditionFilter('packageType', parseSelectValue($event))"
          >
            <option
              v-for="opt in packageTypeFilterOptions"
              :key="opt.value ?? 'all'"
              :value="opt.value === null ? '' : opt.value"
            >
              {{ opt.value === null ? '全部软件包' : opt.label }}
            </option>
          </select>
          <select
            class="toolbar-filter-select"
            :value="filterState.conditionFilters.checkerType === null ? '' : filterState.conditionFilters.checkerType"
            @change="updateConditionFilter('checkerType', parseSelectValue($event))"
          >
            <option
              v-for="opt in checkerTypeFilterOptions"
              :key="opt.value ?? 'all'"
              :value="opt.value === null ? '' : opt.value"
            >
              {{ opt.value === null ? '全部检查器' : opt.label }}
            </option>
          </select>

          <!-- 单选快速筛选下拉：AUR更新失败/上游更新失败/Github地址失效/无AUR版本/无上游版本...（不支持多选）
               恒为纯图标（漏斗 + 下拉箭头，与「条件筛选」按钮区分）；激活时图标整体转为橙色，
               条件名仅通过 title 提示与菜单内的高亮项表达——工具栏横向空间有限，追加文字会挤压换行 -->
          <div class="toolbar-quickfilter">
            <button
              class="btn-icon qf-trigger"
              :class="{ 'qf-active': !!activeQuickFilterLabel }"
              @click="toggleQuickFilterMenu"
              :title="activeQuickFilterLabel ? `快速筛选：${activeQuickFilterLabel}` : '快速筛选'"
            >
              <component :is="Icon.actionFilter" :size="16" />
              <component :is="Icon.actionDropdown" :size="12" class="qf-caret" />
            </button>
            <button
              v-if="activeQuickFilterLabel"
              class="btn-icon qf-clear"
              @click="clearQuickFilter"
              :title="`清除快速筛选：${activeQuickFilterLabel}`"
            >
              <component :is="Icon.actionClear" :size="14" />
            </button>
            <div v-if="quickFilterOpen" class="qf-menu">
              <button
                v-for="opt in quickFilterOptions"
                :key="opt.key"
                class="qf-menu-item"
                :class="{ active: filterState.quickFilter === opt.key }"
                @click="selectAndClose(opt.key)"
              >
                {{ opt.label }}
              </button>
            </div>
          </div>
          <div v-if="quickFilterOpen" class="qf-backdrop" @click="closeQuickFilterMenu"></div>

          <div class="toolbar-divider"></div>
          <button
            class="btn-icon"
            :class="viewModeStore.mode === 'list' ? 'btn-active' : ''"
            @click="viewModeStore.setMode('list')"
            title="列表视图"
          >
            <component :is="Icon.listView" :size="16" />
          </button>
          <button
            class="btn-icon"
            :class="viewModeStore.mode === 'grid' ? 'btn-active' : ''"
            @click="viewModeStore.setMode('grid')"
            title="块视图"
          >
            <component :is="Icon.gridView" :size="16" />
          </button>
        </div>
      </template>

      <template #filter-icon>
        <component :is="Icon.actionFilter" :size="16" />
        <span v-if="activeFilterCount > 0" class="filter-count-badge">{{ activeFilterCount }}</span>
      </template>
    </PageToolbar>

    <FilterBar
      :show="showFilterBar"
      :filter-state="filterState"
      :active-filter-count="activeFilterCount"
      :loading="loading || validating"
      @update:show="showFilterBar = $event"
      @update:filter-state="handleFilterUpdate"
      @validate-urls="handleValidateUrls"
      @reset-filters="resetFilters"
    />

    <!-- 列表视图 -->
    <PackageTable
      v-if="viewModeStore.mode === 'list'"
      :entries="filteredEntries"
      :search-query="searchQuery"
      :page-size="pageSize"
      :current-page="currentPage"
      :is-row-loading="isRowLoading"
      @row-click="openDetailModal"
      @view="openDetailModal"
      @edit="openEditModal"
      @sync-aur="rowSyncFromAur"
      @sync-pkgbuild="rowSyncFromPkgbuild"
      @check-upstream="rowCheckUpstream"
      @delete="(pkgname) => rowDeleteSelected(pkgname, selectedPkgnames, setSelected)"
      @selection-change="(pkgnames: Set<string>) => setSelected(pkgnames)"
    />

    <!-- 块视图 -->
    <BlockView
      v-else
      :entries="filteredEntries"
      :selected-pkgnames="selectedPkgnames"
      :is-row-loading="isRowLoading"
      :page-size="pageSize"
      :current-page="currentPage"
      @select="(pkgname: string) => setSelected(new Set([...selectedPkgnames, pkgname]))"
      @select-all="() => setSelected(new Set(filteredEntries.map(e => e.pkgname)))"
      @view="openDetailModal"
      @edit="openEditModal"
      @sync-aur="rowSyncFromAur"
      @sync-pkgbuild="rowSyncFromPkgbuild"
      @check-upstream="rowCheckUpstream"
      @delete="(pkgname: string) => rowDeleteSelected(pkgname, selectedPkgnames, setSelected)"
      @page-change="(page: number) => { currentPage = page; }"
    />

    <SoftwareFormModal
      :show="showModal"
      :mode="modalMode"
      :pkgname="modalPkgname"
      @close="showModal = false"
      @saved="handleFormSaved"
    />

    <SoftwareDetailModal
      :show="showDetailModal"
      :pkgname="detailPkgname"
      @close="showDetailModal = false"
      @navigate="detailPkgname = $event"
      @entry-updated="(p: string) => refreshEntries([p])"
    />
  </div>
</template>

<style scoped>
.toolbar-filter-group {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.filter-count-badge {
  background: var(--warning);
  color: white;
  font-size: 0.65rem;
  padding: 1px 5px;
  border-radius: 8px;
  margin-left: 2px;
  min-width: 16px;
  text-align: center;
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background-color: var(--border);
  margin: 0 0.375rem;
}

/* 工具栏单选快速筛选下拉 */
.toolbar-quickfilter {
  position: relative;
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.qf-trigger {
  gap: 0.25rem;
}

/* 漏斗与箭头属同一视觉整体，负外边距抵消 .qf-trigger 的 gap，使两者贴合为一个组合图标 */
.qf-caret {
  margin-left: -0.125rem;
}

/* 激活态：不追加文字，改为图标整体转橙色（--warning），避免工具栏横向空间不足导致文字纵向折行。
   特异性需覆盖 .btn-icon:hover:not(:disabled)，故补写 :hover 变体 */
.qf-trigger.qf-active,
.qf-trigger.qf-active:hover:not(:disabled) {
  color: var(--warning);
  background-color: var(--warning-bg);
}

.qf-clear {
  padding: 0.25rem;
}

/* 点击下拉外部关闭菜单的透明遮罩 */
.qf-backdrop {
  position: fixed;
  inset: 0;
  z-index: 50;
}

.qf-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 168px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
  padding: 0.25rem;
  z-index: 60;
  display: flex;
  flex-direction: column;
}

.qf-menu-item {
  text-align: left;
  padding: 0.4rem 0.6rem;
  border: none;
  background: none;
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 0.8125rem;
  cursor: pointer;
  white-space: nowrap;
}

.qf-menu-item:hover {
  background: var(--bg-card);
}

/* 选中项与触发器统一使用橙色（--warning），保持「已启用快速筛选」的视觉语义一致 */
.qf-menu-item.active {
  background: var(--warning-bg);
  color: var(--warning);
  font-weight: 600;
}

/* 注：.toolbar-filter-select 由全局 assets/styles/filter-styles.css 统一提供，
   此处不再重复定义，避免各页下拉框高度/底色不一致 */

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

.btn-icon:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-active {
  background-color: var(--accent-light, rgba(59, 130, 246, 0.1));
  color: var(--accent);
}

/* 彩色操作按钮 - 每个按钮使用独特颜色 */
.btn-color-aur {
  color: #1793d1; /* AUR 蓝 */
}
.btn-color-aur:hover:not(:disabled) {
  background-color: rgba(23, 147, 209, 0.1);
  color: #1793d1;
}

.btn-color-pkgbuild {
  color: #f59e0b; /* PKGBUILD 橙 */
}
.btn-color-pkgbuild:hover:not(:disabled) {
  background-color: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.btn-color-add {
  color: #10b981; /* 添加 绿 */
}
.btn-color-add:hover:not(:disabled) {
  background-color: rgba(16, 185, 129, 0.1);
  color: #10b981;
}

.btn-color-update {
  color: #3b82f6; /* 更新 蓝 */
}
.btn-color-update:hover:not(:disabled) {
  background-color: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.btn-color-check {
  color: #8b5cf6; /* 检查 紫 */
}
.btn-color-check:hover:not(:disabled) {
  background-color: rgba(139, 92, 246, 0.1);
  color: #8b5cf6;
}

.btn-color-delete {
  color: #ef4444; /* 删除 红 */
}
.btn-color-delete:hover:not(:disabled) {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.batch-progress-bar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 1rem;
  margin-bottom: 0.5rem;
  background: var(--muted-bg, #f3f4f6);
  border-radius: 6px;
  border: 1px solid var(--border, #e5e7eb);
}

.progress-track {
  flex: 1;
  height: 6px;
  background: var(--border, #e5e7eb);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--success, #22c55e);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 0.75rem;
  color: var(--text-muted, #6b7280);
  white-space: nowrap;
}
</style>
