<!--
  CacheManager.vue - 缓存管理页面

  功能：
  - 扫描所有启用的缓存目录中的 .pkg.tar.zst 包文件
  - 显示扫描结果（表格形式，支持分页、搜索、选择）
  - 按缓存目录筛选
  - 批量操作：清空缓存表、删除缓存
  - 单行操作：删除缓存
-->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useCacheList, formatSize } from "../composables/useCacheList";
import PageToolbar from "../components/PageToolbar.vue";
import { Trash2, Scan } from "@lucide/vue";
import type { CacheDir } from "../types";

const {
  searchQuery, selectedIds, loading,
  entries, pageSize, currentPage,
  fetchEntries, toggleSelect, toggleSelectAll,
} = useCacheList();

const scanning = ref(false);
const sourceDirFilter = ref("");
const cacheDirs = ref<CacheDir[]>([]);

// 所有可用的来源目录（从数据库读取）
const sourceDirs = computed(() => {
  return cacheDirs.value
    .filter(d => d.is_enabled)
    .sort((a, b) => a.sort_order - b.sort_order);
});

onMounted(async () => {
  try {
    cacheDirs.value = await invoke("list_cache_dirs");
  } catch (e) {
    console.error("加载缓存目录失败:", e);
  }
});

// 根据来源目录筛选
const filteredByDir = computed(() => {
  if (!sourceDirFilter.value) return entries.value;
  return entries.value.filter(e => e.source_dir === sourceDirFilter.value);
});

// 覆盖 pageData 使用 filteredByDir
const displayData = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  return filteredByDir.value.slice(start, start + pageSize.value);
});

async function handleScan() {
  scanning.value = true;
  try {
    await fetchEntries();
  } finally {
    scanning.value = false;
  }
}

async function handleClearTable() {
  if (!confirm("确定要清空缓存表吗？")) return;
  loading.value = true;
  try {
    const count = await invoke<number>("clear_cache_software");
    alert(`已清空缓存表，删除 ${count} 条记录`);
  } catch (e) {
    alert(`清空失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

function deleteSelected() {
  if (selectedIds.value.size === 0) return;
  if (!confirm(`确定要删除选中的 ${selectedIds.value.size} 个缓存文件吗？`)) return;
  alert("批量删除功能开发中");
}

function rowDelete(filename: string) {
  if (!confirm(`确定要删除缓存文件 ${filename} 吗？`)) return;
  alert("删除功能开发中");
}

function updateFilter(dir: string) {
  sourceDirFilter.value = dir;
  currentPage.value = 1;
}
</script>

<template>
  <div>
    <PageToolbar v-model="searchQuery" @refresh="handleScan">
      <template #right>
        <select :value="sourceDirFilter" @change="updateFilter(($event.target as HTMLSelectElement).value)" class="source-dir-select">
          <option value="">全部缓存目录</option>
          <option v-for="dir in sourceDirs" :key="dir.name" :value="dir.name">{{ dir.name }}</option>
        </select>
      </template>
      <button class="btn-icon btn-icon-danger" @click="handleClearTable" :disabled="loading" title="清空缓存表">
        <Trash2 :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" @click="handleScan" :disabled="loading || scanning" title="扫描所有缓存目录">
        <Scan :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" @click="deleteSelected" :disabled="selectedIds.size === 0" title="删除选中">
        <Trash2 :size="16" />
      </button>
    </PageToolbar>

    <div class="card" style="overflow-x: auto; padding: 0">
      <table class="pkg-table">
        <thead>
          <tr>
            <th style="width: 2rem">
              <input type="checkbox"
                :checked="displayData.length > 0 && displayData.every((_, i) => selectedIds.has((currentPage - 1) * pageSize + i))"
                :indeterminate="displayData.some((_, i) => selectedIds.has((currentPage - 1) * pageSize + i)) && !displayData.every((_, i) => selectedIds.has((currentPage - 1) * pageSize + i))"
                @change="toggleSelectAll" />
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
          <tr v-for="(pkg, i) in displayData" :key="pkg.filename"
            :class="{ 'row-selected': selectedIds.has((currentPage - 1) * pageSize + i) }">
            <td @click.stop>
              <input type="checkbox" :checked="selectedIds.has((currentPage - 1) * pageSize + i)"
                @change="toggleSelect((currentPage - 1) * pageSize + i)" />
            </td>
            <td><strong>{{ pkg.name }}</strong></td>
            <td class="cell-filename">{{ pkg.filename }}</td>
            <td>{{ pkg.version }}</td>
            <td>{{ pkg.pkgrel }}</td>
            <td>{{ pkg.arch }}</td>
            <td>{{ formatSize(pkg.size) }}</td>
            <td>{{ pkg.source_dir || "-" }}</td>
            <td>
              <div class="row-actions">
                <button class="btn-icon btn-icon-danger" @click.stop="rowDelete(pkg.filename)" :disabled="loading" title="删除">
                  <Trash2 :size="14" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.pkg-table { width: 100%; border-collapse: collapse; table-layout: auto; }
.pkg-table th { text-align: center; padding: 0.75rem; color: var(--text-secondary); font-weight: 600; font-size: 0.75rem; text-transform: uppercase; border-bottom: 1px solid var(--border); white-space: nowrap; background-color: var(--bg-secondary); }
.pkg-table td { padding: 0.75rem; border-bottom: 1px solid var(--border); font-size: 0.875rem; }
.pkg-table tbody tr { cursor: pointer; transition: background-color 0.15s; }
.pkg-table tbody tr:hover { background-color: rgba(108, 99, 255, 0.05); }
.pkg-table tbody tr.row-selected { background-color: rgba(108, 99, 255, 0.1); }
.row-actions { display: flex; gap: 0.25rem; flex-wrap: nowrap; align-items: center; }
.cell-filename { max-width: 280px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-secondary); font-size: 0.8125rem; }
.source-dir-select { padding: 0.25rem 0.5rem; border: 1px solid var(--border); border-radius: 6px; background-color: var(--bg-card); color: var(--text-primary); font-size: 0.8125rem; outline: none; cursor: pointer; min-width: 120px; }
.source-dir-select:focus { border-color: var(--accent); }
</style>