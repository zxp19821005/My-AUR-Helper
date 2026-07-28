<!--
  BackupManager.vue - 备份管理页面

  功能：
  - 显示备份文件列表（含软件包名称、文件名、版本、架构等）
  - 支持搜索、分页、多选
  - 子目录筛选下拉框
  - 批量操作：清空备份表、扫描备份目录、软件去重、批量安装备份包
  - 单行操作：查看包信息、安装备份包、删除备份
  - sudoers 配置检测与提示弹窗
-->
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useBackupList, fmtEpoch } from "../composables/useBackupList";
import { useBackupInstall } from "../composables/useBackupInstall";
import PageToolbar from "../components/PageToolbar.vue";
import { Trash2, Scan, Copy, Info, Download, X } from "@lucide/vue";
import type { DeduplicateResult } from "../types";

const {
  searchQuery, selectedIds, loading, pageData,
  subdirectoryFilter, subdirectories,
  fetchEntries, toggleSelect, toggleSelectAll, syncToolbar,
} = useBackupList();

const {
  installing, sudoersCommand, showSudoersPrompt,
  pendingInstallPath, pendingInstallPkgname,
  sudoersAvailable: _sudoersAvailable, infoDialogVisible, infoDialogLoading, infoDialogContent, infoDialogPkgname,
  checkSudoers, viewPackageInfo, closeInfoDialog,
  handleInstall, doInstall, closeSudoersPrompt, batchInstall,
} = useBackupInstall();

const backupPath = ref("");
const scanning = ref(false);

async function loadSettings() {
  try {
    const { invoke: inv } = await import("@tauri-apps/api/core");
    const setting = await inv<{ value: string } | null>("get_setting", { key: "backup_dir" });
    if (setting) backupPath.value = setting.value;
  } catch { /* ignore */ }
}

async function loadSubdirectories() {
  try {
    subdirectories.value = await invoke<string[]>("list_backup_subdirectories");
  } catch { /* ignore */ }
}

onMounted(async () => {
  await Promise.all([fetchEntries(), loadSettings(), loadSubdirectories(), checkSudoers()]);
  syncToolbar();
});

async function handleClearTable() {
  if (!confirm("确定要清空备份表吗？这只会删除数据库记录，不会删除磁盘文件。")) return;
  loading.value = true;
  try {
    const count = await invoke<number>("clear_backup_software");
    alert(`已清空备份表，删除 ${count} 条记录`);
    await fetchEntries();
    await loadSubdirectories();
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
    await loadSubdirectories();
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
    await loadSubdirectories();
  } catch (e) {
    alert(`删除失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

function deleteSelected() {
  if (selectedIds.value.size === 0) return;
  if (!confirm(`确定要删除选中的 ${selectedIds.value.size} 个备份记录吗？`)) return;
  alert("批量删除功能开发中");
}

function handleBatchInstall() {
  batchInstall(selectedIds.value, pageData.value);
}
</script>

<template>
  <div>
    <PageToolbar v-model="searchQuery" @refresh="fetchEntries">
      <template #right>
        <select v-model="subdirectoryFilter" class="subdirectory-select">
          <option value="">全部子目录</option>
          <option v-for="dir in subdirectories" :key="dir" :value="dir">{{ dir }}</option>
        </select>
      </template>
      <button class="btn-icon btn-icon-danger" @click="handleClearTable" :disabled="loading" title="清空备份表">
        <Trash2 :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" @click="handleScanDirectory" :disabled="loading || scanning" title="扫描备份目录">
        <Scan :size="16" />
      </button>
      <button class="btn-icon btn-icon-warning" @click="handleDeduplicate" :disabled="loading" title="软件去重">
        <Copy :size="16" />
      </button>
      <button class="btn-icon btn-icon-primary" @click="handleBatchInstall" :disabled="selectedIds.size === 0 || installing" title="批量安装备份包">
        <Download :size="16" />
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
            <th>Epoch</th>
            <th>版本</th>
            <th>PkgRel</th>
            <th>架构</th>
            <th>子目录</th>
            <th style="min-width: 120px">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="pkg in pageData" :key="pkg.id"
            :class="{ 'row-selected': selectedIds.has(pkg.id) }">
            <td @click.stop>
              <input type="checkbox" :checked="selectedIds.has(pkg.id)"
                @change="toggleSelect(pkg.id)" />
            </td>
            <td><strong>{{ pkg.pkgname }}</strong></td>
            <td class="cell-filename">{{ pkg.filename }}</td>
            <td>{{ fmtEpoch(pkg.epoch) }}</td>
            <td>{{ pkg.pkgver }}</td>
            <td>{{ pkg.pkgrel }}</td>
            <td>{{ pkg.arch }}</td>
            <td>{{ pkg.subdirectory || "-" }}</td>
            <td>
              <div class="row-actions">
                <button class="btn-icon btn-icon-info" @click.stop="viewPackageInfo(pkg.full_path, pkg.pkgname)" title="查看信息">
                  <Info :size="14" />
                </button>
                <button class="btn-icon btn-icon-primary" @click.stop="handleInstall(pkg.full_path, pkg.pkgname)" :disabled="installing" title="安装">
                  <Download :size="14" />
                </button>
                <button class="btn-icon btn-icon-danger" @click.stop="rowDelete(pkg.id, pkg.filename)" :disabled="loading" title="删除">
                  <Trash2 :size="14" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 包信息弹窗 -->
    <Teleport to="body">
      <div v-if="infoDialogVisible" class="modal-overlay" @click.self="closeInfoDialog">
        <div class="modal-dialog">
          <div class="modal-header">
            <h3>{{ infoDialogPkgname }} - 包信息</h3>
            <button class="btn-icon btn-icon-danger" @click="closeInfoDialog"><X :size="16" /></button>
          </div>
          <div class="modal-body">
            <div v-if="infoDialogLoading" class="loading-spinner">加载中...</div>
            <pre v-else class="info-content">{{ infoDialogContent }}</pre>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- sudoers 配置提示弹窗 -->
    <Teleport to="body">
      <div v-if="showSudoersPrompt" class="modal-overlay" @click.self="closeSudoersPrompt">
        <div class="modal-dialog">
          <div class="modal-header">
            <h3>需要配置 sudoers 免密</h3>
            <button class="btn-icon btn-icon-danger" @click="closeSudoersPrompt"><X :size="16" /></button>
          </div>
          <div class="modal-body">
            <p>安装备份包需要 root 权限。请在终端中执行以下命令配置 sudoers 免密：</p>
            <pre class="sudoers-command">{{ sudoersCommand }}</pre>
            <p class="hint">配置完成后，点击"重试"按钮继续安装。</p>
          </div>
          <div class="modal-footer">
            <button class="btn btn-secondary" @click="closeSudoersPrompt">取消</button>
            <button class="btn btn-primary" @click="doInstall(pendingInstallPath, pendingInstallPkgname)">重试</button>
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
.cell-filename { max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-secondary); font-size: 0.8125rem; }
.subdirectory-select { padding: 0.25rem 0.5rem; border: 1px solid var(--border); border-radius: 6px; background-color: var(--bg-card); color: var(--text-primary); font-size: 0.8125rem; outline: none; cursor: pointer; min-width: 120px; }
.subdirectory-select:focus { border-color: var(--accent); }
.btn-icon-info { color: var(--text-secondary); }
.btn-icon-info:hover { color: var(--accent); }
.btn-icon-primary { color: var(--text-secondary); }
.btn-icon-primary:hover { color: var(--primary); }
.modal-overlay { position: fixed; inset: 0; background-color: rgba(0, 0, 0, 0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal-dialog { background: var(--bg-primary); border-radius: 12px; border: 1px solid var(--border); box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3); width: 90%; max-width: 700px; max-height: 80vh; display: flex; flex-direction: column; }
.modal-header { display: flex; align-items: center; justify-content: space-between; padding: 1rem 1.5rem; border-bottom: 1px solid var(--border); }
.modal-header h3 { margin: 0; font-size: 1rem; color: var(--text-primary); }
.modal-body { padding: 1.5rem; overflow-y: auto; flex: 1; }
.modal-footer { display: flex; justify-content: flex-end; gap: 0.5rem; padding: 1rem 1.5rem; border-top: 1px solid var(--border); }
.loading-spinner { text-align: center; color: var(--text-secondary); padding: 2rem; }
.info-content { font-family: monospace; font-size: 0.8125rem; line-height: 1.5; white-space: pre-wrap; word-break: break-all; color: var(--text-primary); background: var(--bg-secondary); padding: 1rem; border-radius: 8px; margin: 0; max-height: 400px; overflow-y: auto; }
.sudoers-command { font-family: monospace; font-size: 0.8125rem; background: var(--bg-secondary); padding: 1rem; border-radius: 8px; margin: 0.75rem 0; white-space: pre-wrap; word-break: break-all; color: var(--accent); }
.hint { color: var(--text-secondary); font-size: 0.8125rem; margin-top: 0.5rem; }
.btn { padding: 0.5rem 1rem; border-radius: 6px; border: 1px solid var(--border); cursor: pointer; font-size: 0.8125rem; transition: all 0.15s; }
.btn-primary { background: var(--primary); color: white; border-color: var(--primary); }
.btn-primary:hover { opacity: 0.9; }
.btn-secondary { background: var(--bg-secondary); color: var(--text-primary); }
.btn-secondary:hover { background: var(--bg-card); }
</style>
