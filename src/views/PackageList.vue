<!--
  PackageList.vue - 软件包列表页面

  功能：
  - 显示软件包列表（含 AUR 版本和上游版本）
  - 支持搜索、分页、多选
  - 提供批量操作：同步AUR、同步PKGBUILD、检查上游、删除
  - 支持单行操作：查看详情、编辑、同步、检查、删除
  - 支持筛选器：快速筛选（OR）+ 条件筛选（AND）

  使用组件：
  - StandardizedTable: 数据表格
  - PageToolbar: 页面工具栏
  - FilterBar: 筛选器
  - PackageRowActions: 行操作按钮组
  - SoftwareFormModal: 软件表单弹窗
  - SoftwareDetailModal: 软件详情弹窗
-->
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePackageStore } from "../stores/packages";
import { usePackageActions } from "../composables/packageActions";
import { usePackageList } from "../composables/usePackageList";
import PageToolbar from "../components/common/PageToolbar.vue";
import FilterBar from "../components/filter/FilterBar.vue";
import SoftwareFormModal from "../components/package/SoftwareFormModal.vue";
import SoftwareDetailModal from "../components/package/SoftwareDetailModal.vue";
import PackageTable from "../components/package/PackageTable.vue";
import { RefreshCw, Plus, Trash2, Info, Download, Filter } from "@lucide/vue";
import { packageTypeFilterOptions, checkerTypeFilterOptions } from "../utils/enums";
import type { ValidateResult } from "../types";

const pkgStore = usePackageStore();

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
  openAddModal,
  openEditModal,
  openDetailModal,
  onModalSaved,
  setSelected,
  syncToolbar,
  activeFilterCount,
  resetFilters,
} = usePackageList();

const {
  loading,
  isRowLoading,
  syncFromAur,
  syncFromPkgbuild,
  updateAurInfo,
  checkSelectedUpstream,
  deleteSelected,
  rowSyncFromAur,
  rowSyncFromPkgbuild,
  rowCheckUpstream,
  rowDelete,
} = usePackageActions(fetchView, syncToolbar);

const validating = ref(false);

async function handleValidateUrls() {
  validating.value = true;
  try {
    const pkgnameList = pageData.value.map((p) => p.pkgname);
    await invoke<ValidateResult[]>("validate_upstream_urls", {
      pkgnameList: pkgnameList.length > 0 ? pkgnameList : null,
    });
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

onMounted(async () => {
  await Promise.all([fetchView(), pkgStore.fetchPackages()]);
});
</script>

<template>
  <div>
    <PageToolbar 
      v-model="searchQuery" 
      @refresh="fetchView"
      :filter-active="activeFilterCount > 0"
      @toggle-filter="showFilterBar = !showFilterBar"
    >
      <template #filter-icon>
        <Filter :size="16" />
        <span v-if="activeFilterCount > 0" class="filter-count-badge">{{ activeFilterCount }}</span>
      </template>

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
              {{ opt.value === null ? '全部类型' : opt.label }}
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
        </div>
      </template>
      <button class="btn-icon btn-icon-accent" @click="syncFromAur(selectedPkgnames)" :disabled="loading" title="从AUR同步">
        <RefreshCw :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" @click="syncFromPkgbuild(selectedPkgnames)" :disabled="loading" title="从PKGBUILD同步">
        <Download :size="16" />
      </button>
      <button class="btn-icon btn-icon-success" @click="openAddModal" title="添加软件">
        <Plus :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" @click="updateAurInfo(selectedPkgnames)" :disabled="loading" title="更新AUR信息">
        <Info :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" @click="checkSelectedUpstream(selectedPkgnames)" :disabled="loading" title="更新上游信息">
        <RefreshCw :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" @click="deleteSelected(selectedPkgnames, setSelected)" :disabled="selectedPkgnames.size === 0" title="删除选中">
        <Trash2 :size="16" />
      </button>
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

    <!-- 软件包数据表格 -->
    <PackageTable
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
      @delete="(pkgname) => rowDelete(pkgname, selectedPkgnames, setSelected)"
    />

    <SoftwareFormModal
      :show="showModal"
      :mode="modalMode"
      :pkgname="modalPkgname"
      @close="showModal = false"
      @saved="onModalSaved"
    />

    <SoftwareDetailModal
      :show="showDetailModal"
      :pkgname="detailPkgname"
      @close="showDetailModal = false"
      @navigate="detailPkgname = $event"
    />
  </div>
</template>

<style scoped>
/* 顶部工具栏内联筛选（软件包类型 / 检查器类型） */
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
</style>