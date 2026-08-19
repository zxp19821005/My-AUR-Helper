<!--
  BackupManager.vue - 备份管理页面

  功能：
  - 显示备份文件列表（含软件包名称、版本、架构等）
  - 支持搜索、分页、多选
  - 子目录筛选下拉框
  - 批量操作：清空备份表、扫描备份目录、软件去重、批量安装备份包
  - 单行操作：查看包信息、安装备份包、删除备份
  - sudoers 配置检测与提示弹窗

  使用组件：
  - PageToolbar: 工具栏组件
  - BackupRowActions: 行操作按钮组
  - StandardizedTable: 表格组件
  - BackupInfoDialog / BackupSudoersDialog: 弹窗
  - useBackupInfoNav: 详情弹窗导航与选择逻辑（见 composables/useBackupInfoNav.ts）
-->
<script setup lang="ts">
import { onMounted, ref, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useBackupList, fmtEpoch } from "../composables/useBackupList";
import { useBackupInstall } from "../composables/useBackupInstall";
import { useBackupInfoNav } from "../composables/useBackupInfoNav";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import { openConfirm as confirm } from "../composables/useConfirm";
import type { DeduplicateResult, BackupSoftwareEntry } from "../types";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import BackupRowActions from "../components/backup/BackupRowActions.vue";
import BackupInfoDialog from "../components/backup/BackupInfoDialog.vue";
import BackupSudoersDialog from "../components/backup/BackupSudoersDialog.vue";
import { Icon } from "../icons";

const footer = inject(FOOTER_KEY)!;

const {
  searchQuery, selectedIds, loading,
  filteredEntries, pageSize, currentPage,
  subdirectoryFilter, subdirectories,
  archFilter, architectures,
  fetchEntries, syncToolbar,
} = useBackupList();

const {
  installing, sudoersCommand, showSudoersPrompt,
  pendingInstallPath, pendingInstallPkgname,
  infoDialogVisible, infoDialogLoading, infoDialogContent, infoDialogPkgname, infoDialogEntry,
  checkSudoers, viewPackageInfo, closeInfoDialog,
  handleInstall, doInstall, closeSudoersPrompt, batchInstall,
} = useBackupInstall();

const {
  prevEntry, nextEntry, onDialogNavigate, handleSelectionChange,
} = useBackupInfoNav({ filteredEntries, selectedIds, infoDialogEntry, viewPackageInfo });

const backupPath = ref("");
const scanning = ref(false);

async function loadSettings() {
  try {
    const setting = await invoke<{ value: string } | null>("get_setting", { key: "backup_dir" });
    if (setting) backupPath.value = setting.value;
  } catch (e) {
    console.error("加载备份目录设置失败:", e);
  }
}

async function loadSubdirectories() {
  try {
    subdirectories.value = await invoke<string[]>("list_backup_subdirectories");
  } catch (e) {
    console.error("加载备份子目录失败:", e);
  }
}

onMounted(async () => {
  await Promise.all([fetchEntries(), loadSettings(), loadSubdirectories(), checkSudoers()]);
  syncToolbar();
});

async function handleClearTable() {
  if (!(await confirm({ message: "确定要清空备份表吗？这只会删除数据库记录，不会删除磁盘文件。", variant: "danger" }))) return;
  loading.value = true;
  try {
    const count = await invoke<number>("clear_backup_software");
    addMessage(footer, "success", `已清空备份表，删除 ${count} 条记录`);
    await fetchEntries();
    await loadSubdirectories();
  } catch (e) {
    addMessage(footer, "error", `清空失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function handleScanDirectory() {
  if (!backupPath.value) {
    addMessage(footer, "warning", "请先在设置中配置备份目录");
    return;
  }
  scanning.value = true;
  try {
    const count = await invoke<number>("scan_backup_directory", { backupPath: backupPath.value });
    addMessage(footer, "success", `扫描完成，新增 ${count} 条备份记录`);
    await fetchEntries();
    await loadSubdirectories();
  } catch (e) {
    addMessage(footer, "error", `扫描失败: ${e}`);
  } finally {
    scanning.value = false;
  }
}

async function handleDeduplicate() {
  if (!backupPath.value) {
    addMessage(footer, "warning", "请先在设置中配置备份目录");
    return;
  }
  if (!(await confirm({ message: "确定要执行软件去重吗？将删除每个包的旧版本文件和数据库记录。", variant: "danger" }))) return;
  loading.value = true;
  try {
    const result = await invoke<DeduplicateResult>("deduplicate_backups", { backupPath: backupPath.value });
    const msg = `去重完成：删除 ${result.removed_files} 个文件，${result.removed_records} 条记录`;
    if (result.errors.length > 0) {
      addMessage(footer, "warning", `${msg}，错误: ${result.errors.join("; ")}`);
    } else {
      addMessage(footer, "success", msg);
    }
    await fetchEntries();
  } catch (e) {
    addMessage(footer, "error", `去重失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function rowDelete(id: number, filename: string) {
  if (!(await confirm({ message: `确定要删除备份文件 ${filename} 吗？`, variant: "danger" }))) return;
  loading.value = true;
  try {
    await invoke("delete_backup", { id, backupPath: backupPath.value });
    addMessage(footer, "success", `已删除备份文件 ${filename}`);
    await fetchEntries();
    await loadSubdirectories();
  } catch (e) {
    addMessage(footer, "error", `删除失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function deleteSelected() {
  if (selectedIds.value.size === 0) return;
  if (!(await confirm({ message: `确定要删除选中的 ${selectedIds.value.size} 个备份记录吗？`, variant: "danger" }))) return;
  addMessage(footer, "info", "批量删除功能开发中");
}

function handleBatchInstall() {
  batchInstall(selectedIds.value, filteredEntries.value);
}

/** 详情弹窗：安装备份包 */
function onDialogInstall(entry: BackupSoftwareEntry) {
  handleInstall(entry.full_path, entry.pkgname);
}

/** 详情弹窗：删除备份（删除后关闭弹窗） */
function onDialogDelete(entry: BackupSoftwareEntry) {
  rowDelete(entry.id, entry.filename);
  closeInfoDialog();
}

const columns = [
  { key: "pkgname", title: "包名" },
  { key: "epoch", title: "Epoch" },
  { key: "pkgver", title: "版本" },
  { key: "pkgrel", title: "PkgRel" },
  { key: "arch", title: "架构" },
  { key: "subdirectory", title: "子目录" },
];
</script>

<template>
  <div>
    <!-- 工具栏 -->
    <PageToolbar 
      v-model="searchQuery" 
      @refresh="fetchEntries"
      :show-filter-button="false"
    >
      <template #filters>
        <select
          class="toolbar-filter-select"
          :value="subdirectoryFilter"
          @change="subdirectoryFilter = ($event.target as HTMLSelectElement).value"
        >
          <option value="">全部子目录</option>
          <option v-for="dir in subdirectories" :key="dir" :value="dir">{{ dir }}</option>
        </select>
        <select
          class="toolbar-filter-select"
          :value="archFilter"
          @change="archFilter = ($event.target as HTMLSelectElement).value"
        >
          <option value="">全部架构</option>
          <option v-for="a in architectures" :key="a" :value="a">{{ a }}</option>
        </select>
      </template>
      <button class="btn-icon btn-icon-danger" :disabled="loading" @click="handleClearTable" title="清空备份表">
        <component :is="Icon.clearTable" :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" :disabled="loading || scanning" @click="handleScanDirectory" title="扫描备份目录">
        <component :is="Icon.scan" :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" :disabled="loading" @click="handleDeduplicate" title="软件去重">
        <component :is="Icon.dedup" :size="16" />
      </button>
      <button class="btn-icon btn-icon-success" :disabled="selectedIds.size === 0 || installing" @click="handleBatchInstall" title="批量安装备份包">
        <component :is="Icon.install" :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" :disabled="selectedIds.size === 0" @click="deleteSelected" title="删除选中">
        <component :is="Icon.deleteSelected" :size="16" />
      </button>
    </PageToolbar>

    <!-- 备份表格（key 固定，避免条数变化时整表重建） -->
    <StandardizedTable
      key="backup-table"
      :columns="columns"
      :data="filteredEntries"
      :pageSize="pageSize"
      :currentPage="currentPage"
      rowKey="id"
      showCheckbox
      showIndex
      striped
      hoverable
      :showPagination="false"
      emptyText="暂无备份数据"
      @selection-change="handleSelectionChange"
    >
      <template #cell-pkgname="{ row }">
        <strong>{{ row.pkgname }}</strong>
      </template>

      <template #cell-epoch="{ row }">
        {{ fmtEpoch(row.epoch) }}
      </template>

      <template #cell-subdirectory="{ row }">
        {{ row.subdirectory || "-" }}
      </template>

      <template #actions="{ row }">
        <BackupRowActions
          :row="row"
          :loading="loading"
          :installing="installing"
          @view-info="(r) => viewPackageInfo(r)"
          @install="(r) => handleInstall(r.full_path, r.pkgname)"
          @delete="(r) => rowDelete(r.id, r.filename)"
        />
      </template>
    </StandardizedTable>

    <!-- 包详情弹窗 -->
    <BackupInfoDialog
      :show="infoDialogVisible"
      :loading="infoDialogLoading"
      :pkgname="infoDialogPkgname"
      :content="infoDialogContent"
      :entry="infoDialogEntry"
      :prev-entry="prevEntry"
      :next-entry="nextEntry"
      v-model:installing="installing"
      v-model:deleting="loading"
      @close="closeInfoDialog"
      @install="onDialogInstall"
      @delete="onDialogDelete"
      @navigate="onDialogNavigate"
    />

    <!-- sudoers 配置提示弹窗 -->
    <BackupSudoersDialog
      :show="showSudoersPrompt"
      :sudoers-command="sudoersCommand"
      :pending-install-path="pendingInstallPath"
      :pending-install-pkgname="pendingInstallPkgname"
      :installing="installing"
      @close="closeSudoersPrompt"
      @install="doInstall"
    />
  </div>
</template>
