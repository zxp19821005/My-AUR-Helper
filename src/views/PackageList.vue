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
  - StandardizedBadge: 状态徽章
  - PageToolbar: 页面工具栏
  - FilterBar: 筛选器
  - SoftwareFormModal: 软件表单弹窗
  - SoftwareDetailModal: 软件详情弹窗
-->
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePackageStore } from "../stores/packages";
import { usePackageActions } from "../composables/packageActions";
import { usePackageList, fmtTimestamp } from "../composables/usePackageList";
import PageToolbar from "../components/common/PageToolbar.vue";
import FilterBar from "../components/filter/FilterBar.vue";
import SoftwareFormModal from "../components/package/SoftwareFormModal.vue";
import SoftwareDetailModal from "../components/package/SoftwareDetailModal.vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedBadge from "../components/base/StandardizedBadge.vue";
import {
  RefreshCw,
  Plus,
  Trash2,
  Eye,
  Pencil,
  Info,
  Download,
  Filter,
} from "@lucide/vue";
import type { ValidateResult } from "../types";
import type { Column } from "../composables/useTableState";

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
    const results = await invoke<ValidateResult[]>("validate_upstream_urls", {
      pkgnameList: pkgnameList.length > 0 ? pkgnameList : null,
    });
    await fetchView();
    console.log(`验证完成: ${results.length} 个软件包`);
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

/** 表格列配置 */
const columns: Column[] = [
  {
    key: "pkgname",
    title: "包名",
    sortable: true,
  },
  {
    key: "aur_version",
    title: "AUR 版本",
    sortable: true,
  },
  {
    key: "aur_last_updated",
    title: "AUR 最后提交",
    formatter: (value: any) => fmtTimestamp(value),
  },
  {
    key: "upstream_version",
    title: "上游版本",
    sortable: true,
  },
  {
    key: "upstream_last_checked",
    title: "上游检查日期",
    formatter: (value: any) => fmtTimestamp(value),
  },
];

/** 处理行点击 */
function handleRowClick(row: any) {
  openDetailModal(row.pkgname);
}

/** 处理选择变化 */
function handleSelectionChange(selectedRows: any[]) {
  // 更新选中状态（由StandardizedTable内部管理）
  console.log(`已选中 ${selectedRows.length} 个软件包`);
}

onMounted(async () => {
  await Promise.all([fetchView(), pkgStore.fetchPackages()]);
});
</script>

<template>
  <div>
    <PageToolbar v-model="searchQuery" @refresh="fetchView">
      <template #right>
        <button
          class="btn-icon"
          :class="activeFilterCount > 0 ? 'btn-icon-warning' : 'btn-icon-default'"
          @click="showFilterBar = !showFilterBar"
          title="筛选"
        >
          <Filter :size="16" />
          <span v-if="activeFilterCount > 0" class="filter-count-badge">{{ activeFilterCount }}</span>
        </button>
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

    <!-- 使用StandardizedTable替换原有表格 -->
    <StandardizedTable
      :key="`table-${filteredEntries.length}`"
      :columns="columns"
      :data="filteredEntries"
      :pageSize="pageSize"
      :searchQuery="searchQuery"
      :searchFields="['pkgname', 'aur_version', 'upstream_version']"
      :currentPage="currentPage"
      rowKey="pkgname"
      showCheckbox
      showIndex
      striped
      hoverable
      clickable
      :showPagination="false"
      emptyText="暂无软件包"
      @selection-change="handleSelectionChange"
      @row-click="handleRowClick"
    >
      <!-- 自定义包名列 -->
      <template #cell-pkgname="{ row }">
        <strong :class="{ 'pkg-outdated': row.is_outdated }">
          {{ row.pkgname }}
        </strong>
        <StandardizedBadge
          v-if="row.is_outdated"
          type="warning"
          text="需更新"
          size="sm"
          variant="soft"
          class="ml-2"
        />
      </template>

      <!-- 自定义AUR版本列 -->
      <template #cell-aur_version="{ row }">
        {{ row.aur_version || "-" }}
      </template>

      <!-- 自定义AUR最后提交列 -->
      <template #cell-aur_last_updated="{ row }">
        {{ fmtTimestamp(row.aur_last_updated) }}
      </template>

      <!-- 自定义上游版本列 -->
      <template #cell-upstream_version="{ row }">
        {{ row.upstream_version || "-" }}
      </template>

      <!-- 自定义上游检查日期列 -->
      <template #cell-upstream_last_checked="{ row }">
        {{ fmtTimestamp(row.upstream_last_checked) }}
      </template>

      <!-- 操作列 -->
      <template #actions="{ row }">
        <button
          class="btn-icon btn-icon-default"
          @click.stop="openDetailModal(row.pkgname)"
          title="查看详情"
        >
          <Eye :size="14" />
        </button>
        <button
          class="btn-icon btn-icon-accent"
          @click.stop="openEditModal(row.pkgname)"
          title="软件编辑"
        >
          <Pencil :size="14" />
        </button>
        <button
          class="btn-icon btn-icon-accent"
          @click.stop="rowSyncFromAur(row.pkgname)"
          :disabled="isRowLoading(row.pkgname, 'sync-aur')"
          title="从AUR同步"
        >
          <RefreshCw :size="14" />
        </button>
        <button
          class="btn-icon btn-icon-accent"
          @click.stop="rowSyncFromPkgbuild(row.pkgname)"
          :disabled="isRowLoading(row.pkgname, 'sync-pkgbuild')"
          title="从PKGBUILD同步"
        >
          <Download :size="14" />
        </button>
        <button
          class="btn-icon btn-icon-info"
          @click.stop="rowCheckUpstream(row.pkgname)"
          :disabled="isRowLoading(row.pkgname, 'check-upstream')"
          title="更新上游信息"
        >
          <RefreshCw :size="14" />
        </button>
        <button
          class="btn-icon btn-icon-danger"
          @click.stop="rowDelete(row.pkgname, selectedPkgnames, setSelected)"
          :disabled="isRowLoading(row.pkgname, 'delete')"
          title="删除"
        >
          <Trash2 :size="14" />
        </button>
      </template>
    </StandardizedTable>

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
.pkg-outdated {
  color: var(--warning);
}

.ml-2 {
  margin-left: 0.5rem;
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

/* 操作列按钮间距 */
:deep(.actions-cell) {
  gap: 0.25rem;
}
</style>