<!--
  CacheManager.vue - 缓存管理页面

  功能：
  - 扫描所有启用的缓存目录中的 .pkg.tar.zst 包文件
  - 显示扫描结果（表格形式，支持分页、搜索、选择）
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
import { FOOTER_KEY, addMessage } from "../composables/footer";
import BackupToModal from "../components/backup/BackupToModal.vue";
import type { DeduplicateResult } from "../types";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import CacheToolbar from "../components/cache/CacheToolbar.vue";
import CacheRowActions from "../components/cache/CacheRowActions.vue";

const footer = inject(FOOTER_KEY)!;

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
const showBackupToModal = ref(false);

const sourceDirs = computed(() => {
  return cacheDirs.value.filter((d) => d.path);
});

const selectedFilenames = computed(() => {
  return filteredByDir.value
    .filter((_, i) => selectedIds.value.has(i))
    .map((e) => e.filename);
});

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

const displayData = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  return filteredByDir.value.slice(start, start + pageSize.value);
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

async function handleDedup() {
  if (!backupPath.value) {
    addMessage(footer, "warning", "未设置备份目录，请先在设置中配置备份目录");
    return;
  }
  if (!confirm("确定要对备份目录进行去重吗？将删除旧版本文件。")) return;
  loading.value = true;
  try {
    const result = await invoke<DeduplicateResult>("deduplicate_backups", {
      backupPath: backupPath.value,
    });
    const msg = `去重完成：删除 ${result.removed_files} 个文件，${result.removed_records} 条记录`;
    if (result.errors.length > 0) {
      addMessage(footer, "warning", `${msg}，错误: ${result.errors.join("; ")}`);
    } else {
      addMessage(footer, "success", msg);
    }
  } catch (e) {
    addMessage(footer, "error", `去重失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function handleBackupNewVersion() {
  if (selectedIds.value.size === 0) {
    addMessage(footer, "warning", "请先选择要备份的缓存包");
    return;
  }
  if (!backupPath.value) {
    addMessage(footer, "warning", "未设置备份目录，请先在设置中配置备份目录");
    return;
  }
  loading.value = true;
  try {
    const [success, errors] = await invoke<[number, string[]]>(
      "backup_cache_to_existing",
      {
        filenames: selectedFilenames.value,
        backupPath: backupPath.value,
      }
    );
    const msg = `备份完成：成功 ${success} 个` + (errors.length > 0 ? `，错误: ${errors.join("; ")}` : "");
    if (errors.length > 0) {
      addMessage(footer, "warning", msg);
    } else {
      addMessage(footer, "success", msg);
    }
    selectedIds.value.clear();
  } catch (e) {
    addMessage(footer, "error", `备份失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

function openBackupToModal() {
  if (selectedIds.value.size === 0) {
    addMessage(footer, "warning", "请先选择要备份的缓存包");
    return;
  }
  if (!backupPath.value) {
    addMessage(footer, "warning", "未设置备份目录，请先在设置中配置备份目录");
    return;
  }
  showBackupToModal.value = true;
}

function handleBackupSuccess(result: [number, string[]]) {
  const [success, errors] = result;
  const msg = `备份完成：成功 ${success} 个` + (errors.length > 0 ? `，错误: ${errors.join("; ")}` : "");
  if (errors.length > 0) {
    addMessage(footer, "warning", msg);
  } else {
    addMessage(footer, "success", msg);
  }
  selectedIds.value.clear();
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
  { key: "filename", title: "文件名" },
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
    <CacheToolbar
      v-model:search-query="searchQuery"
      :source-dir-filter="sourceDirFilter"
      :source-dirs="sourceDirs"
      :loading="loading"
      :scanning="scanning"
      :selected-count="selectedIds.size"
      @update:source-dir-filter="handleSourceDirFilterChange"
      @scan="handleScan"
      @clear-table="handleClearTable"
      @delete-selected="deleteSelected"
      @dedup="handleDedup"
      @backup-new-version="handleBackupNewVersion"
      @backup-to="openBackupToModal"
    />

    <!-- 缓存表格 -->
    <StandardizedTable
      :columns="columns"
      :data="displayData"
      rowKey="filename"
      showCheckbox
      showIndex
      striped
      hoverable
      emptyText="暂无缓存数据"
      @selection-change="handleSelectionChange"
    >
      <template #cell-pkgname="{ row }">
        <strong>{{ row.pkgname }}</strong>
      </template>

      <template #cell-filename="{ row }">
        <span class="cell-filename">{{ row.filename }}</span>
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

<style scoped>
.cell-filename {
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
  font-size: 0.8125rem;
}
</style>