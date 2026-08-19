<!--
  CacheManager.vue - 缓存管理页面

  功能：
  - 扫描所有启用的缓存目录中的 .pkg.tar.zst 包文件
  - 显示扫描结果（表格形式，支持分页、搜索、选择，不含文件名列）
  - 按缓存目录筛选
  - 批量操作：清空缓存表、去重、备份新版（自动比较版本）、备份到、删除缓存、缓存清理
  - 单行操作：删除缓存

  使用组件：
  - PageToolbar: 工具栏组件
  - CacheRowActions: 行操作按钮组
  - StandardizedTable: 表格组件
  - BackupToModal / BackupInfoDialog / BackupSudoersDialog / CacheSudoersModal: 弹窗
  - useCacheInfoNav: 详情弹窗导航与选择逻辑（见 composables/useCacheInfoNav.ts）
-->
<script setup lang="ts">
import { ref, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useCacheList } from "../composables/useCacheList";
import { fmtEpoch } from "../composables/useBackupList";
import { useCacheBackupActions } from "../composables/useCacheBackupActions";
import { useCacheManagerInit } from "../composables/useCacheManagerInit";
import { useCacheCleanup } from "../composables/useCacheCleanup";
import { useCacheInstall } from "../composables/useCacheInstall";
import { useCacheInfoNav, cacheColumns } from "../composables/useCacheInfoNav";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import { openConfirm as confirm } from "../composables/useConfirm";
import BackupToModal from "../components/backup/BackupToModal.vue";
import BackupInfoDialog from "../components/backup/BackupInfoDialog.vue";
import BackupSudoersDialog from "../components/backup/BackupSudoersDialog.vue";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import CacheRowActions from "../components/cache/CacheRowActions.vue";
import CacheSudoersModal from "../components/cache/CacheSudoersModal.vue";
import { Icon } from "../icons";

const footer = inject(FOOTER_KEY)!;

const {
  searchQuery,
  selectedIds,
  loading,
  filteredEntries,
  pageSize,
  currentPage,
  sourceDirFilter,
  sourceDirs,
  archFilter,
  architectures,
  loadEntries,
  fetchEntries,
} = useCacheList();

const scanning = ref(false);

const {
  loading: cleanupLoading,
  sudoersCommand,
  showSudoersPrompt,
  handleFullCleanup,
  closeSudoersPrompt,
} = useCacheCleanup();

const {
  installing,
  sudoersCommand: installSudoersCommand,
  showSudoersPrompt: showInstallSudoersPrompt,
  pendingInstallPath,
  pendingInstallPkgname,
  infoDialogVisible,
  infoDialogLoading,
  infoDialogContent,
  infoDialogPkgname,
  infoDialogEntry,
  checkSudoers,
  viewPackageInfo,
  resolveFullPath,
  closeInfoDialog,
  handleInstall,
  doInstall,
  closeSudoersPrompt: closeInstallSudoersPrompt,
} = useCacheInstall();

// 初始化逻辑（加载缓存表/缓存目录/backup_dir/子目录/sudoers）抽离到 composable，
// 需在 useCacheInstall 之后调用以复用其 checkSudoers
const { backupPath, backupSubdirectories } = useCacheManagerInit(footer, {
  loadEntries,
  setSourceDirs: (dirs) => (sourceDirs.value = dirs),
  checkSudoers,
});

// useCacheInfoNav 必须在 useCacheBackupActions 之前调用，
// 因为后者依赖前者导出的 selectedFilenames
const { openCacheInfo, prevEntry, nextEntry, onCacheInfoNavigate, handleSelectionChange, selectedFilenames } = useCacheInfoNav({ filteredEntries, selectedIds, viewPackageInfo });

const {
  showBackupToModal,
  handleDedup,
  handleBackupNewVersion,
  openBackupToModal,
  handleBackupSuccess,
} = useCacheBackupActions(footer, backupPath, selectedIds, selectedFilenames, loading);

// onMounted 初始化已抽离到 useCacheManagerInit composable

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
  if (!(await confirm({ message: "确定要清空缓存表吗？", variant: "danger" }))) return;
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

async function deleteSelected() {
  if (selectedIds.value.size === 0) return;
  if (!(await confirm({ message: `确定要删除选中的 ${selectedIds.value.size} 个缓存文件吗？`, variant: "danger" }))) return;
  addMessage(footer, "info", "批量删除功能开发中");
}

async function rowDelete(filename: string) {
  if (!(await confirm({ message: `确定要删除缓存文件 ${filename} 吗？`, variant: "danger" }))) return;
  addMessage(footer, "info", "删除功能开发中");
}
</script>

<template>
  <div>
    <!-- 工具栏 -->
    <PageToolbar 
      v-model="searchQuery" 
      @refresh="handleScan"
      :show-filter-button="false"
    >
      <template #filters>
        <select
          class="toolbar-filter-select"
          :value="sourceDirFilter"
          @change="sourceDirFilter = ($event.target as HTMLSelectElement).value"
        >
          <option value="">全部缓存目录</option>
          <option v-for="dir in sourceDirs" :key="dir.name" :value="dir.name">
            {{ dir.name }}
          </option>
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
      <button class="btn-icon btn-icon-accent" :disabled="loading || scanning" @click="handleScan" title="扫描所有缓存目录">
        <component :is="Icon.scan" :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" :disabled="loading" @click="handleClearTable" title="清空缓存表">
        <component :is="Icon.clearTable" :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" :disabled="selectedIds.size === 0" @click="deleteSelected" title="删除选中">
        <component :is="Icon.deleteSelected" :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" :disabled="loading" @click="handleDedup" title="去重（保留最新版本）">
        <component :is="Icon.dedup" :size="16" />
      </button>
      <button class="btn-icon btn-icon-success" :disabled="loading" @click="handleBackupNewVersion" title="备份新版（自动比较版本，将更新的包备份到已有位置）">
        <component :is="Icon.backupNewVersion" :size="16" />
      </button>
      <button class="btn-icon btn-icon-success" :disabled="loading || selectedIds.size === 0" @click="openBackupToModal" title="备份到（选择子目录）">
        <component :is="Icon.backupTo" :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" :disabled="loading || cleanupLoading || scanning" @click="() => handleFullCleanup(fetchEntries)" title="缓存清理（清理系统缓存和自定义缓存目录）">
        <component :is="Icon.fullCleanup" :size="16" />
      </button>
    </PageToolbar>

    <!-- 缓存表格（key 固定，避免条数变化时整表重建） -->
    <StandardizedTable
      key="cache-table"
      :columns="cacheColumns"
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

      <template #cell-epoch="{ row }">
        {{ fmtEpoch(row.epoch) }}
      </template>

      <template #cell-cache_directory="{ row }">
        {{ row.cache_directory || "-" }}
      </template>

      <template #actions="{ row }">
        <CacheRowActions
          :row="row"
          :loading="loading"
          :installing="installing"
          @view-info="(r) => openCacheInfo(r)"
          @install="(r) => handleInstall(resolveFullPath(r), r.pkgname)"
          @delete="(r) => rowDelete(r.filename)"
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

    <!-- sudoers 配置提示弹窗 -->
    <CacheSudoersModal
      :show="showSudoersPrompt"
      :sudoers-command="sudoersCommand"
      @close="closeSudoersPrompt"
    />

    <!-- 缓存包详情弹窗 -->
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
      @install="(e) => handleInstall(resolveFullPath(e), e.pkgname)"
      @delete="(e) => { rowDelete(e.filename); closeInfoDialog(); }"
      @navigate="(e) => onCacheInfoNavigate(e)"
    />

    <!-- 安装 sudoers 配置提示弹窗 -->
    <BackupSudoersDialog
      :show="showInstallSudoersPrompt"
      :sudoers-command="installSudoersCommand"
      :pending-install-path="pendingInstallPath"
      :pending-install-pkgname="pendingInstallPkgname"
      :installing="installing"
      @close="closeInstallSudoersPrompt"
      @install="doInstall"
    />
  </div>
</template>
