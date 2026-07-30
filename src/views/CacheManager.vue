<!--
  CacheManager.vue - 缓存管理页面

  功能：
  - 扫描所有启用的缓存目录中的 .pkg.tar.zst 包文件
  - 显示扫描结果（表格形式，支持分页、搜索、选择，不含文件名列）
  - 按缓存目录筛选
  - 批量操作：清空缓存表、去重、备份新版、备份到、删除缓存
  - 单行操作：删除缓存

  使用组件：
  - CacheToolbar: 工具栏组件
  - CacheRowActions: 行操作按钮组
  - StandardizedTable: 表格组件
  - BackupToModal: 备份弹窗组件
-->
<script setup lang="ts">
import { ref, computed, onMounted, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useCacheList, formatSize } from "../composables/useCacheList";
import { loadEnabledCacheDirs } from "../composables/useCacheDirs";
import { useCacheBackupActions } from "../composables/useCacheBackupActions";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import BackupToModal from "../components/backup/BackupToModal.vue";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import CacheRowActions from "../components/cache/CacheRowActions.vue";
import { Trash2, Scan, Copy, GitBranch, Filter, X } from "@lucide/vue";

const footer = inject(FOOTER_KEY)!;

const showFilterBar = ref(false);
const activeFilterCount = computed(() => sourceDirFilter.value ? 1 : 0);

const {
  searchQuery,
  selectedIds,
  loading,
  filteredEntries,
  pageSize,
  currentPage,
  loadEntries,
  fetchEntries,
} = useCacheList();

const scanning = ref(false);
const sourceDirFilter = ref("");
const cacheDirs = ref<{ name: string; path: string }[]>([]);
const backupPath = ref("");
const backupSubdirectories = ref<string[]>([]);

const sourceDirs = computed(() => {
  return cacheDirs.value.filter((d) => d.path);
});

const selectedFilenames = computed(() => {
  return filteredByDir.value
    .filter((_, i) => selectedIds.value.has(i))
    .map((e) => e.filename);
});

const {
  showBackupToModal,
  handleDedup,
  handleBackupNewVersion,
  openBackupToModal,
  handleBackupSuccess,
} = useCacheBackupActions(footer, backupPath, selectedIds, selectedFilenames, loading);

onMounted(async () => {
  try {
    cacheDirs.value = await loadEnabledCacheDirs();
  } catch (e) {
    console.error("加载缓存目录失败:", e);
  }
  try {
    const setting = await invoke<{ value: string } | null>("get_setting", {
      key: "backup_dir",
    });
    if (setting) backupPath.value = setting.value;
  } catch { /* ignore */ }
  try {
    backupSubdirectories.value = await invoke<string[]>("list_backup_subdirectories");
  } catch { /* ignore */ }
});

const filteredByDir = computed(() => {
  if (!sourceDirFilter.value) return filteredEntries.value;
  return filteredEntries.value.filter((e) => e.source_dir === sourceDirFilter.value);
});

function handleSourceDirFilterChange(dir: string | number) {
  sourceDirFilter.value = String(dir);
  currentPage.value = 1;
  selectedIds.value = new Set();
}

async function handleScan() {
  scanning.value = true;
  try {
    await fetchEntries();
  } catch (e) {
    console.error("[缓存管理] 扫描失败:", e);
    addMessage(footer, "error", `扫描缓存目录失败: ${e}`);
  } finally {
    scanning.value = false;
  }
}

async function handleClearTable() {
  if (!confirm("确定要清空缓存表吗？")) return;
  loading.value = true;
  try {
    const count = await invoke<number>("clear_cache_software");
    await loadEntries();
    addMessage(footer, "success", `已清空缓存表，删除 ${count} 条记录`);
  } catch (e) {
    addMessage(footer, "error", `清空失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

function deleteSelected() {
  if (selectedIds.value.size === 0) return;
  if (!confirm(`确定要删除选中的 ${selectedIds.value.size} 个缓存文件吗？`)) return;
  addMessage(footer, "info", "批量删除功能开发中");
}

function rowDelete(filename: string) {
  if (!confirm(`确定要删除缓存文件 ${filename} 吗？`)) return;
  addMessage(footer, "info", "删除功能开发中");
}

function handleSelectionChange(selectedRows: any[]) {
  const newSelected = new Set<number>();
  selectedRows.forEach((row) => {
    const idx = filteredByDir.value.findIndex(
      (e) => e.filename === row.filename
    );
    if (idx !== -1) newSelected.add(idx);
  });
  selectedIds.value = newSelected;
}

const columns = [
  { key: "pkgname", title: "包名" },
  { key: "version", title: "版本" },
  { key: "pkgrel", title: "PkgRel" },
  { key: "arch", title: "架构" },
  { key: "size", title: "大小" },
  { key: "source_dir", title: "来源目录" },
];
</script>

<template>
  <div>
    <!-- 工具栏 -->
    <PageToolbar 
      v-model="searchQuery" 
      @refresh="handleScan"
      :filter-active="activeFilterCount > 0"
      @toggle-filter="showFilterBar = !showFilterBar"
    >
      <template #filter-icon>
        <Filter :size="16" />
        <span v-if="activeFilterCount > 0" class="filter-count-badge">{{ activeFilterCount }}</span>
      </template>
      <button class="btn-icon btn-icon-accent" :disabled="loading || scanning" @click="handleScan" title="扫描所有缓存目录">
        <Scan :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" :disabled="loading" @click="handleClearTable" title="清空缓存表">
        <Trash2 :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" :disabled="selectedIds.size === 0" @click="deleteSelected" title="删除选中">
        <Trash2 :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" :disabled="loading" @click="handleDedup" title="去重（保留最新版本）">
        <GitBranch :size="16" />
      </button>
      <button class="btn-icon btn-icon-success" :disabled="loading || selectedIds.size === 0" @click="handleBackupNewVersion" title="备份新版（备份到已有位置）">
        <Copy :size="16" />
      </button>
      <button class="btn-icon btn-icon-success" :disabled="loading || selectedIds.size === 0" @click="openBackupToModal" title="备份到（选择子目录）">
        <Copy :size="16" />
      </button>
    </PageToolbar>

    <!-- 筛选面板 -->
    <Teleport to="body">
      <div v-if="showFilterBar" class="filter-overlay" @click.self="showFilterBar = false">
        <div class="filter-panel">
          <div class="filter-header">
            <div class="filter-title">
              <Filter :size="16" />
              <span>筛选条件</span>
              <span v-if="activeFilterCount > 0" class="filter-badge">{{ activeFilterCount }}</span>
            </div>
            <button class="btn-icon btn-icon-default" @click="showFilterBar = false">
              <X :size="16" />
            </button>
          </div>
          <div class="filter-body">
            <div class="filter-section">
              <div class="filter-row">
                <div class="filter-field">
                  <label class="filter-field-label">缓存目录</label>
                  <select 
                    class="filter-select"
                    :value="sourceDirFilter"
                    @change="handleSourceDirFilterChange(($event.target as HTMLSelectElement).value)"
                  >
                    <option value="">全部缓存目录</option>
                    <option v-for="dir in sourceDirs" :key="dir.name" :value="dir.name">
                      {{ dir.name }}
                    </option>
                  </select>
                </div>
              </div>
            </div>
          </div>
          <div class="filter-footer">
            <button class="btn btn-secondary" @click="handleSourceDirFilterChange('')">
              清空筛选
            </button>
            <button class="btn btn-primary" @click="showFilterBar = false">
              应用筛选
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- 缓存表格 -->
    <StandardizedTable
      :key="`table-${filteredEntries.length}`"
      :columns="columns"
      :data="filteredEntries"
      :pageSize="pageSize"
      :currentPage="currentPage"
      rowKey="filename"
      showCheckbox
      showIndex
      striped
      hoverable
      :showPagination="false"
      emptyText="暂无缓存数据"
      @selection-change="handleSelectionChange"
    >
      <template #cell-pkgname="{ row }">
        <strong>{{ row.pkgname }}</strong>
      </template>

      <template #cell-size="{ row }">
        {{ formatSize(row.size) }}
      </template>

      <template #cell-source_dir="{ row }">
        {{ row.source_dir || "-" }}
      </template>

      <template #actions="{ row }">
        <CacheRowActions
          :loading="loading"
          @delete="rowDelete(row.filename)"
        />
      </template>
    </StandardizedTable>

    <!-- 备份到弹窗 -->
    <BackupToModal
      :show="showBackupToModal"
      :selected-filenames="selectedFilenames"
      :backup-path="backupPath"
      :subdirectories="backupSubdirectories"
      @close="showBackupToModal = false"
      @success="handleBackupSuccess"
    />
  </div>
</template>
