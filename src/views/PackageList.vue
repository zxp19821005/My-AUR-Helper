<!--
  PackageList.vue - 软件包列表页面。列表/块视图切换、搜索分页多选、批量与单行操作、筛选器。
  AUR 搜索导入弹窗 → AurSearchModal.vue；批量按钮组 → PackageBatchToolbar.vue；
  筛选/视图切换区 → PackageListFilterToolbar.vue（内含 QuickFilterDropdown.vue）；样式 → PackageList.css。
-->
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { usePackageActions } from "../composables/usePackageActions";
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
import AurSearchModal from "../components/package/AurSearchModal.vue";
import PackageBatchToolbar from "../components/package/PackageBatchToolbar.vue";
import PackageListFilterToolbar from "../components/package/PackageListFilterToolbar.vue";
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
  syncByMaintainer,
  syncFromPkgbuild,
  updateAurInfo,
  checkSelectedUpstream,
  checkLoading,
  deleteSelected,
  rowSyncFromAur,
  rowSyncFromPkgbuild,
  rowCheckUpstream,
  rowDeleteSelected,
} = usePackageActions(fetchView, refreshEntries, syncToolbar);

const aurSearchOpen = ref(false);

async function handleValidateUrls() {
  try {
    const pkgnameList = pageData.value.map((p) => p.pkgname);
    await softwareApi.validateUpstreamUrls(
      pkgnameList.length > 0 ? pkgnameList : null,
    );
    await fetchView();
  } catch (error) {
    console.error("验证失败:", error);
  } finally {
    showFilterBar.value = false;
  }
}

function handleFilterUpdate(newState: typeof filterState.value) {
  filterState.value = newState;
}

/** 更新条件筛选（软件包类型 / 检查器类型），绑定到共享 filterState */
function updateConditionFilter(key: "packageType" | "checkerType", value: number | null) {
  filterState.value = {
    ...filterState.value,
    conditionFilters: { ...filterState.value.conditionFilters, [key]: value },
  };
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

/** AUR 导入成功回调：刷新列表 */
async function handleAurImported() {
  await fetchView();
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
      <!-- 左侧：常用工具按钮（已抽为纯 props/emits 组件 PackageBatchToolbar） -->
      <PackageBatchToolbar
        :loading="loading"
        :check-loading="checkLoading"
        :selected-count="selectedPkgnames.size"
        @sync-aur="syncFromAur(selectedPkgnames)"
        @sync-by-maintainer="syncByMaintainer"
        @open-aur-import="aurSearchOpen = true"
        @sync-pkgbuild="syncFromPkgbuild(selectedPkgnames)"
        @add="openAddModal"
        @update-aur="updateAurInfo(selectedPkgnames)"
        @check-upstream="checkSelectedUpstream(selectedPkgnames)"
        @delete="deleteSelected(selectedPkgnames, setSelected)"
      />

      <!-- 中间：筛选器和视图切换（已抽为纯 props/emits 组件 PackageListFilterToolbar） -->
      <template #filters>
        <PackageListFilterToolbar
          :condition-filters="filterState.conditionFilters"
          :package-type-options="packageTypeFilterOptions"
          :checker-type-options="checkerTypeFilterOptions"
          :quick-filter-options="quickFilterOptions"
          :quick-filter-key="filterState.quickFilter"
          :quick-filter-label="activeQuickFilterLabel"
          :view-mode="viewModeStore.mode"
          @update-condition-filter="updateConditionFilter"
          @select-quick-filter="selectQuickFilter"
          @clear-quick-filter="clearQuickFilter"
          @set-view-mode="viewModeStore.setMode"
        />
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
      :loading="loading"
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

    <!-- AUR 搜索导入对话框 -->
    <AurSearchModal v-model:open="aurSearchOpen" @imported="handleAurImported" />
  </div>
</template>

<style scoped src="./PackageList.css"></style>
