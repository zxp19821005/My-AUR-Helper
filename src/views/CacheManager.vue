<!--
  CacheManager.vue - 缓存管理页面

  功能：
  - 扫描所有启用的缓存目录中的 .pkg.tar.zst 包文件
  - 显示扫描结果（表格形式，支持分页、搜索、选择）
  - 按缓存目录筛选
  - 批量操作：清空缓存表、去重、备份新版、备份到、删除缓存
  - 单行操作：删除缓存
-->
<script setup lang="ts">
import { ref, computed, onMounted, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useCacheList, formatSize } from "../composables/useCacheList";
import { loadEnabledCacheDirs } from "../composables/useCacheDirs";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import PageToolbar from "../components/PageToolbar.vue";
import BackupToModal from "../components/BackupToModal.vue";
import { Trash2, Scan, Copy, GitBranch } from "@lucide/vue";
import type { DeduplicateResult } from "../types";

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
  toggleSelect,
  toggleSelectAll,
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
  // 选中索引是相对于「目录筛选后列表」的全局行号，需与勾选逻辑保持同一数据源
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
  } catch {
    /* ignore */
  }
  try {
    backupSubdirectories.value = await invoke<string[]>("list_backup_subdirectories");
  } catch {
    /* ignore */
  }
});

const filteredByDir = computed(() => {
  // 基于搜索过滤后的列表再按来源目录筛选，保证搜索框对表格生效
  if (!sourceDirFilter.value) return filteredEntries.value;
  return filteredEntries.value.filter((e) => e.source_dir === sourceDirFilter.value);
});

const displayData = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  return filteredByDir.value.slice(start, start + pageSize.value);
});

function updateFilter(dir: string) {
  sourceDirFilter.value = dir;
  currentPage.value = 1;
  // 数据源变化后原选中行号不再有效，清空选择
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
</script>

<template>
  <div>
    <PageToolbar v-model="searchQuery" @refresh="handleScan">
      <template #right>
        <select
          :value="sourceDirFilter"
          @change="updateFilter(($event.target as HTMLSelectElement).value)"
          class="source-dir-select"
        >
          <option value="">全部缓存目录</option>
          <option v-for="dir in sourceDirs" :key="dir.name" :value="dir.name">
            {{ dir.name }}
          </option>
        </select>
      </template>
      <button
        class="btn-icon btn-icon-danger"
        @click="handleClearTable"
        :disabled="loading"
        title="清空缓存表"
      >
        <Trash2 :size="16" />
      </button>
      <button
        class="btn-icon btn-icon-accent"
        @click="handleScan"
        :disabled="loading || scanning"
        title="扫描所有缓存目录"
      >
        <Scan :size="16" />
      </button>
      <button
        class="btn-icon btn-icon-danger"
        @click="deleteSelected"
        :disabled="selectedIds.size === 0"
        title="删除选中"
      >
        <Trash2 :size="16" />
      </button>
      <button
        class="btn-icon btn-icon-accent"
        @click="handleDedup"
        :disabled="loading"
        title="去重（保留最新版本）"
      >
        <GitBranch :size="16" />
      </button>
      <button
        class="btn-icon btn-icon-accent"
        @click="handleBackupNewVersion"
        :disabled="loading || selectedIds.size === 0"
        title="备份新版（备份到已有位置）"
      >
        <Copy :size="16" />
      </button>
      <button
        class="btn-icon btn-icon-accent"
        @click="openBackupToModal"
        :disabled="loading || selectedIds.size === 0"
        title="备份到（选择子目录）"
      >
        <Copy :size="16" />
      </button>
    </PageToolbar>

    <div class="card" style="overflow-x: auto; padding: 0">
      <table class="pkg-table">
        <thead>
          <tr>
            <th style="width: 2rem">
              <input
                type="checkbox"
                :checked="
                  displayData.length > 0 &&
                  displayData.every((_, i) =>
                    selectedIds.has((currentPage - 1) * pageSize + i)
                  )
                "
                :indeterminate="
                  displayData.some((_, i) =>
                    selectedIds.has((currentPage - 1) * pageSize + i)
                  ) &&
                  !displayData.every((_, i) =>
                    selectedIds.has((currentPage - 1) * pageSize + i)
                  )
                "
                @change="toggleSelectAll"
              />
            </th>
            <th>包名</th>
            <th>文件名</th>
            <th>版本</th>
            <th>PkgRel</th>
            <th>架构</th>
            <th>大小</th>
            <th>来源目录</th>
            <th style="min-width: 60px">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(pkg, i) in displayData"
            :key="pkg.filename"
            :class="{
              'row-selected': selectedIds.has((currentPage - 1) * pageSize + i),
            }"
          >
            <td @click.stop>
              <input
                type="checkbox"
                :checked="selectedIds.has((currentPage - 1) * pageSize + i)"
                @change="toggleSelect((currentPage - 1) * pageSize + i)"
              />
            </td>
            <td><strong>{{ pkg.pkgname }}</strong></td>
            <td class="cell-filename">{{ pkg.filename }}</td>
            <td>{{ pkg.version }}</td>
            <td>{{ pkg.pkgrel }}</td>
            <td>{{ pkg.arch }}</td>
            <td>{{ formatSize(pkg.size) }}</td>
            <td>{{ pkg.source_dir || "-" }}</td>
            <td>
              <div class="row-actions">
                <button
                  class="btn-icon btn-icon-danger"
                  @click.stop="rowDelete(pkg.filename)"
                  :disabled="loading"
                  title="删除"
                >
                  <Trash2 :size="14" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

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
.cell-filename {
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
  font-size: 0.8125rem;
}
.source-dir-select {
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--bg-card);
  color: var(--text-primary);
  font-size: 0.8125rem;
  outline: none;
  cursor: pointer;
  min-width: 120px;
}
.source-dir-select:focus {
  border-color: var(--accent);
}
</style>
