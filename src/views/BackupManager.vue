<!--
  BackupManager.vue - 备份管理页面

  功能：
  - 显示备份文件列表（含软件包名称、文件名、版本、架构等）
  - 支持搜索、分页、多选
  - 提供批量操作：清空备份表、扫描备份目录、软件去重
  - 支持单行操作：删除备份（同时删除磁盘文件）
-->
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useBackupList, fmtEpoch } from "../composables/useBackupList";
import PageToolbar from "../components/PageToolbar.vue";
import {
  Trash2,
  Scan,
  Copy,
} from "@lucide/vue";
import type { DeduplicateResult } from "../types";

const {
  searchQuery,
  selectedIds,
  loading,
  pageData,
  fetchEntries,
  toggleSelect,
  toggleSelectAll,
  syncToolbar,
} = useBackupList();

const backupPath = ref("");
const scanning = ref(false);

async function loadSettings() {
  try {
    const { invoke: inv } = await import("@tauri-apps/api/core");
    const setting = await inv<{ value: string } | null>("get_setting", { key: "backup_dir" });
    if (setting) backupPath.value = setting.value;
  } catch { /* ignore */ }
}

onMounted(async () => {
  await Promise.all([fetchEntries(), loadSettings()]);
  syncToolbar();
});

async function handleClearTable() {
  if (!confirm("确定要清空备份表吗？这只会删除数据库记录，不会删除磁盘文件。")) return;
  loading.value = true;
  try {
    const count = await invoke<number>("clear_backup_software");
    alert(`已清空备份表，删除 ${count} 条记录`);
    await fetchEntries();
  } catch (e) {
    alert(`清空失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function handleScanDirectory() {
  if (!backupPath.value) {
    alert("请先在设置中配置备份目录");
    return;
  }
  scanning.value = true;
  try {
    const count = await invoke<number>("scan_backup_directory", { backupPath: backupPath.value });
    alert(`扫描完成，新增 ${count} 条备份记录`);
    await fetchEntries();
  } catch (e) {
    alert(`扫描失败: ${e}`);
  } finally {
    scanning.value = false;
  }
}

async function handleDeduplicate() {
  if (!backupPath.value) {
    alert("请先在设置中配置备份目录");
    return;
  }
  if (!confirm("确定要执行软件去重吗？将删除每个包的旧版本文件和数据库记录。")) return;
  loading.value = true;
  try {
    const result = await invoke<DeduplicateResult>("deduplicate_backups", { backupPath: backupPath.value });
    const msg = `去重完成：删除 ${result.removed_files} 个文件，${result.removed_records} 条记录`;
    if (result.errors.length > 0) {
      alert(`${msg}\n\n错误:\n${result.errors.join("\n")}`);
    } else {
      alert(msg);
    }
    await fetchEntries();
  } catch (e) {
    alert(`去重失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function rowDelete(id: number, filename: string) {
  if (!confirm(`确定要删除备份文件 ${filename} 吗？`)) return;
  loading.value = true;
  try {
    await invoke("delete_backup", { id, backupPath: backupPath.value });
    await fetchEntries();
  } catch (e) {
    alert(`删除失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

function deleteSelected() {
  if (selectedIds.value.size === 0) return;
  if (!confirm(`确定要删除选中的 ${selectedIds.value.size} 个备份记录吗？`)) return;
  // TODO: batch delete
  alert("批量删除功能开发中");
}
</script>

<template>
  <div>
    <PageToolbar v-model="searchQuery" @refresh="fetchEntries">
      <button class="btn-icon btn-icon-danger" @click="handleClearTable" :disabled="loading" title="清空备份表">
        <Trash2 :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" @click="handleScanDirectory" :disabled="loading || scanning" title="扫描备份目录">
        <Scan :size="16" />
      </button>
      <button class="btn-icon btn-icon-warning" @click="handleDeduplicate" :disabled="loading" title="软件去重">
        <Copy :size="16" />
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
                :checked="pageData.length > 0 && pageData.every(p => selectedIds.has(p.id))"
                :indeterminate="pageData.some(p => selectedIds.has(p.id)) && !pageData.every(p => selectedIds.has(p.id))"
                @change="toggleSelectAll" />
            </th>
            <th>包名</th>
            <th>文件名</th>
            <th>版本</th>
            <th>Epoch</th>
            <th>PkgRel</th>
            <th>架构</th>
            <th>子目录</th>
            <th style="min-width: 80px">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="pkg in pageData" :key="pkg.id"
            :class="{ 'row-selected': selectedIds.has(pkg.id) }">
            <td @click.stop>
              <input type="checkbox" :checked="selectedIds.has(pkg.id)"
                @change="toggleSelect(pkg.id)" />
            </td>
            <td>
              <strong>{{ pkg.pkgname }}</strong>
            </td>
            <td class="cell-filename">{{ pkg.filename }}</td>
            <td>{{ pkg.pkgver }}</td>
            <td>{{ fmtEpoch(pkg.epoch) }}</td>
            <td>{{ pkg.pkgrel }}</td>
            <td>{{ pkg.arch }}</td>
            <td>{{ pkg.subdirectory || "-" }}</td>
            <td>
              <div class="row-actions">
                <button class="btn-icon btn-icon-danger" @click.stop="rowDelete(pkg.id, pkg.filename)" :disabled="loading" title="删除">
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
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
  font-size: 0.8125rem;
}
</style>
