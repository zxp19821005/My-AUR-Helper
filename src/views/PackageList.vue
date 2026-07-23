<!--
  PackageList.vue - 软件包列表页面

  功能：
  - 显示软件包列表（含 AUR 版本和上游版本）
  - 支持搜索、分页、多选
  - 提供批量操作：同步AUR、同步PKGBUILD、检查上游、删除
  - 支持单行操作：查看详情、编辑、同步、检查、删除
  - 支持筛选器：快速筛选（OR）+ 条件筛选（AND）
-->
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePackageStore } from "../stores/packages";
import { usePackageActions } from "../composables/packageActions";
import { usePackageList, fmtTimestamp } from "../composables/usePackageList";
import PageToolbar from "../components/PageToolbar.vue";
import FilterBar from "../components/FilterBar.vue";
import SoftwareFormModal from "../components/SoftwareFormModal.vue";
import SoftwareDetailModal from "../components/SoftwareDetailModal.vue";
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
  fetchView,
  toggleSelect,
  toggleSelectAll,
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
    // 获取当前筛选后的包名列表
    const pkgnameList = pageData.value.map((p) => p.pkgname);
    const results = await invoke<ValidateResult[]>("validate_upstream_urls", {
      pkgnameList: pkgnameList.length > 0 ? pkgnameList : null,
    });
    // 刷新列表以显示更新后的状态
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

    <div class="card" style="overflow-x: auto; padding: 0">
      <table class="pkg-table">
        <thead>
          <tr>
            <th style="width: 2rem">
              <input type="checkbox"
                :checked="pageData.length > 0 && pageData.every(p => selectedPkgnames.has(p.pkgname))"
                :indeterminate="pageData.some(p => selectedPkgnames.has(p.pkgname)) && !pageData.every(p => selectedPkgnames.has(p.pkgname))"
                @change="toggleSelectAll" />
            </th>
            <th>包名</th>
            <th>AUR 版本</th>
            <th>AUR 最后提交</th>
            <th>上游版本</th>
            <th>上游检查日期</th>
            <th style="min-width: 200px">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="pkg in pageData" :key="pkg.pkgname"
            :class="{ 'row-selected': selectedPkgnames.has(pkg.pkgname) }">
            <td @click.stop>
              <input type="checkbox" :checked="selectedPkgnames.has(pkg.pkgname)"
                @change="toggleSelect(pkg.pkgname)" />
            </td>
            <td>
              <strong :class="{ 'pkg-outdated': pkg.is_outdated }">{{ pkg.pkgname }}</strong>
            </td>
            <td>{{ pkg.aur_version || "-" }}</td>
            <td>{{ fmtTimestamp(pkg.aur_last_updated) }}</td>
            <td>{{ pkg.upstream_version || "-" }}</td>
            <td>{{ fmtTimestamp(pkg.upstream_last_checked) }}</td>
            <td>
              <div class="row-actions">
                <button class="btn-icon btn-icon-default" @click.stop="openDetailModal(pkg.pkgname)" title="查看详情">
                  <Eye :size="14" />
                </button>
                <button class="btn-icon btn-icon-accent" @click.stop="openEditModal(pkg.pkgname)" title="软件编辑">
                  <Pencil :size="14" />
                </button>
                <button class="btn-icon btn-icon-accent" @click.stop="rowSyncFromAur(pkg.pkgname)" :disabled="isRowLoading(pkg.pkgname, 'sync-aur')" title="从AUR同步">
                  <RefreshCw :size="14" />
                </button>
                <button class="btn-icon btn-icon-accent" @click.stop="rowSyncFromPkgbuild(pkg.pkgname)" :disabled="isRowLoading(pkg.pkgname, 'sync-pkgbuild')" title="从PKGBUILD同步">
                  <Download :size="14" />
                </button>
                <button class="btn-icon btn-icon-info" @click.stop="rowCheckUpstream(pkg.pkgname)" :disabled="isRowLoading(pkg.pkgname, 'check-upstream')" title="更新上游信息">
                  <RefreshCw :size="14" />
                </button>
                <button class="btn-icon btn-icon-danger" @click.stop="rowDelete(pkg.pkgname, selectedPkgnames, setSelected)" :disabled="isRowLoading(pkg.pkgname, 'delete')" title="删除">
                  <Trash2 :size="14" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

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
.pkg-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: auto;
}
.pkg-table th {
  text-align: center;
  padding: 0.75rem;
  color: var(--text-secondary);
  font-weight: 600;
  font-size: 0.75rem;
  text-transform: uppercase;
  border-bottom: 1px solid var(--border);
  white-space: nowrap;
  background-color: var(--bg-secondary);
}
.pkg-table td {
  padding: 0.75rem;
  border-bottom: 1px solid var(--border);
  font-size: 0.875rem;
}
.pkg-table tbody tr {
  cursor: pointer;
  transition: background-color 0.15s;
}
.pkg-table tbody tr:hover {
  background-color: rgba(108, 99, 255, 0.05);
}
.pkg-table tbody tr.row-selected {
  background-color: rgba(108, 99, 255, 0.1);
}

.row-actions {
  display: flex;
  gap: 0.25rem;
  flex-wrap: nowrap;
  align-items: center;
}

.pkg-outdated {
  color: var(--warning);
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