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
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useCacheList, formatSize } from "../composables/useCacheList";
import PageToolbar from "../components/PageToolbar.vue";
import { Trash2, Scan, Copy, GitBranch } from "@lucide/vue";
import type { DeduplicateResult } from "../types";

const {
  searchQuery, selectedIds, loading,
  entries, pageSize, currentPage,
  fetchEntries, toggleSelect, toggleSelectAll,
} = useCacheList();

const scanning = ref(false);
const sourceDirFilter = ref("");
const cacheDirs = ref<{name: string, path: string}[]>([]);
const backupPath = ref("");
const backupSubdirectories = ref<string[]>([]);

// 备份到弹窗状态
const showBackupToModal = ref(false);
const backupToSubdirectory = ref("");
const backingUp = ref(false);

// 所有可用的来源目录（从 settings 读取）
const sourceDirs = computed(() => {
  return cacheDirs.value.filter(d => d.path);
});

// 获取选中的文件名列表
const selectedFilenames = computed(() => {
  return entries.value
    .filter((_, i) => selectedIds.value.has(i))
    .map(e => e.filename);
});

onMounted(async () => {
  try {
    await loadCacheDirs();
  } catch (e) {
    console.error("加载缓存目录失败:", e);
  }
  try {
    const setting = await invoke<{ value: string } | null>("get_setting", { key: "backup_dir" });
    if (setting) backupPath.value = setting.value;
  } catch { /* ignore */ }
  try {
    backupSubdirectories.value = await invoke<string[]>("list_backup_subdirectories");
  } catch { /* ignore */ }
});

async function loadCacheDirs() {
  const dirs: {name: string, path: string}[] = [];
  
  const systemDir = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_system" });
  if (systemDir?.value) {
    dirs.push({ name: "系统缓存", path: systemDir.value });
  }
  
  const paruDir = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_paru" });
  if (paruDir?.value) {
    dirs.push({ name: "paru 缓存", path: paruDir.value });
  }
  
  const yayDir = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_yay" });
  if (yayDir?.value) {
    dirs.push({ name: "yay 缓存", path: yayDir.value });
  }
  
  cacheDirs.value = dirs;
}

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

function updateFilter(dir: string) {
  sourceDirFilter.value = dir;
  currentPage.value = 1;
}

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

async function handleDedup() {
  if (!backupPath.value) {
    alert("未设置备份目录，请先在设置中配置备份目录");
    return;
  }
  if (!confirm("确定要对备份目录进行去重吗？将删除旧版本文件。")) return;
  loading.value = true;
  try {
    const result = await invoke<DeduplicateResult>("deduplicate_backups", {
      backupPath: backupPath.value,
    });
    alert(
      `去重完成\n\n删除文件: ${result.removed_files} 个\n删除记录: ${result.removed_records} 条` +
      (result.errors.length > 0 ? `\n\n错误:\n${result.errors.join("\n")}` : "")
    );
  } catch (e) {
    alert(`去重失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function handleBackupNewVersion() {
  if (selectedIds.value.size === 0) {
    alert("请先选择要备份的缓存包");
    return;
  }
  if (!backupPath.value) {
    alert("未设置备份目录，请先在设置中配置备份目录");
    return;
  }
  loading.value = true;
  try {
    const [success, errors] = await invoke<[number, string[]]>("backup_cache_to_existing", {
      filenames: selectedFilenames.value,
      backupPath: backupPath.value,
    });
    const msg = `备份完成\n\n成功: ${success} 个` +
      (errors.length > 0 ? `\n\n详细:\n${errors.join("\n")}` : "");
    alert(msg);
    selectedIds.value.clear();
  } catch (e) {
    alert(`备份失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

function openBackupToModal() {
  if (selectedIds.value.size === 0) {
    alert("请先选择要备份的缓存包");
    return;
  }
  if (!backupPath.value) {
    alert("未设置备份目录，请先在设置中配置备份目录");
    return;
  }
  backupToSubdirectory.value = "";
  showBackupToModal.value = true;
}

async function handleBackupTo() {
  if (!backupToSubdirectory.value && backupSubdirectories.value.length > 0) {
    alert("请选择一个备份子目录");
    return;
  }
  backingUp.value = true;
  try {
    const [success, errors] = await invoke<[number, string[]]>("backup_cache_to_subdirectory", {
      filenames: selectedFilenames.value,
      backupPath: backupPath.value,
      subdirectory: backupToSubdirectory.value,
    });
    const msg = `备份完成\n\n成功: ${success} 个` +
      (errors.length > 0 ? `\n\n详细:\n${errors.join("\n")}` : "");
    alert(msg);
    selectedIds.value.clear();
    showBackupToModal.value = false;
  } catch (e) {
    alert(`备份失败: ${e}`);
  } finally {
    backingUp.value = false;
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
      <button class="btn-icon btn-icon-accent" @click="handleDedup" :disabled="loading" title="去重（保留最新版本）">
        <GitBranch :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" @click="handleBackupNewVersion" :disabled="loading || selectedIds.size === 0" title="备份新版（备份到已有位置）">
        <Copy :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" @click="openBackupToModal" :disabled="loading || selectedIds.size === 0" title="备份到（选择子目录）">
        <Copy :size="16" />
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

    <!-- 备份到弹窗 -->
    <Teleport to="body">
      <div v-if="showBackupToModal" class="modal-overlay" @click.self="showBackupToModal = false">
        <div class="modal-content">
          <div class="modal-header">
            <h3>备份到子目录</h3>
            <button class="btn-icon" @click="showBackupToModal = false">
              <span>&times;</span>
            </button>
          </div>
          <div class="modal-body">
            <p>选中 {{ selectedFilenames.length }} 个缓存包</p>
            <div class="form-group">
              <label>选择备份子目录：</label>
              <select v-model="backupToSubdirectory" class="backup-dir-select">
                <option value="">根目录</option>
                <option v-for="dir in backupSubdirectories" :key="dir" :value="dir">{{ dir }}</option>
              </select>
            </div>
          </div>
          <div class="modal-footer">
            <button class="btn-secondary" @click="showBackupToModal = false">取消</button>
            <button class="btn-primary" @click="handleBackupTo" :disabled="backingUp">
              {{ backingUp ? "备份中..." : "确认备份" }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
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

.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal-content { background: var(--bg-card); border-radius: 12px; width: 420px; max-width: 90vw; box-shadow: 0 8px 32px rgba(0,0,0,0.2); }
.modal-header { display: flex; justify-content: space-between; align-items: center; padding: 1rem 1.25rem; border-bottom: 1px solid var(--border); }
.modal-header h3 { margin: 0; font-size: 1rem; }
.modal-body { padding: 1.25rem; }
.modal-footer { display: flex; justify-content: flex-end; gap: 0.5rem; padding: 1rem 1.25rem; border-top: 1px solid var(--border); }
.form-group { margin-top: 1rem; }
.form-group label { display: block; margin-bottom: 0.5rem; font-size: 0.875rem; color: var(--text-secondary); }
.backup-dir-select { width: 100%; padding: 0.5rem; border: 1px solid var(--border); border-radius: 6px; background-color: var(--bg-card); color: var(--text-primary); font-size: 0.875rem; outline: none; }
.backup-dir-select:focus { border-color: var(--accent); }
.btn-secondary { padding: 0.5rem 1rem; border: 1px solid var(--border); border-radius: 6px; background: transparent; color: var(--text-primary); cursor: pointer; font-size: 0.875rem; }
.btn-secondary:hover { background: var(--bg-secondary); }
.btn-primary { padding: 0.5rem 1rem; border: none; border-radius: 6px; background: var(--accent); color: white; cursor: pointer; font-size: 0.875rem; }
.btn-primary:hover { opacity: 0.9; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
