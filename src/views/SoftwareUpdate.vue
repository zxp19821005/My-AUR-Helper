<!--
  SoftwareUpdate.vue - 软件更新工作台（导航中位于「软件管理」之后）

  功能：
  - 只展示数据库中「过期」标记（is_outdated = 1）的软件包，不在浏览时重新比较版本
  - 汇总 AUR 现网版本 / 上游版本 / 本地 PKGBUILD 实际版本，展示写回状态
  - 状态筛选、搜索、多选、分页，与备份 / 缓存管理页共用 PageToolbar + StandardizedTable
  - 所有写回必须先经 dry-run diff 预览并确认；写回后自动 rescan 刷新磁盘状态

  设计说明：状态不落库。写回本地 PKGBUILD 后 AUR 现网版本并未改变，
  列表以磁盘实际 pkgver 判定「已写回 · 待推送 AUR」，因此不新增表、不改 schema。
-->
<script setup lang="ts">
import { inject, onMounted, ref } from "vue";
import { useListBase } from "../composables/useListBase";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import { Icon } from "../icons";
import type { Column } from "../composables/useTableState";
import type {
  SoftwareUpdateItem,
  SoftwareUpdatePreview,
  SoftwareUpdateStatus,
} from "../types";
import { UPDATE_STATUS_LABEL } from "../types";
import * as updateApi from "@/api/update";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedBadge from "../components/base/StandardizedBadge.vue";
import UpdateDiffDialog from "../components/update/UpdateDiffDialog.vue";

const footer = inject(FOOTER_KEY)!;

/** 状态筛选值，空串表示全部 */
const statusFilter = ref<SoftwareUpdateStatus | "">("");

/** 统计待写回数量（函数声明提升到 useListBase 之前，供 infoText 延迟调用） */
function pendingTotal(): number {
  return entries.value.filter((e) => e.status === "pending").length;
}

const {
  searchQuery, loading, entries, filteredEntries, pageSize, currentPage, syncToolbar,
} = useListBase<SoftwareUpdateItem>({
  pageSizeSetting: "list_page_size_update",
  filter: (all, q) =>
    all.filter((entry) => {
      if (statusFilter.value && entry.status !== statusFilter.value) return false;
      if (!q) return true;
      return (
        entry.pkgname.toLowerCase().includes(q) ||
        (entry.upstream_version ?? "").toLowerCase().includes(q)
      );
    }),
  getKey: (entry) => entry.software_id,
  infoText: (total) => `过期 ${total} 项 · 待写回 ${pendingTotal()} 项`,
  pageResetRefs: [statusFilter],
});

const columns: Column[] = [
  { key: "pkgname", title: "包名", width: "24%" },
  { key: "aur_version", title: "AUR 版本" },
  { key: "upstream_version", title: "上游版本" },
  { key: "local_version", title: "本地 PKGBUILD" },
  { key: "status", title: "状态" },
];

const selectedRows = ref<SoftwareUpdateItem[]>([]);
const showDiff = ref(false);
const previewLoading = ref(false);
const applying = ref(false);
const previews = ref<SoftwareUpdatePreview[]>([]);

/** 状态 → 中文标签（包一层避免模板里 any 直接索引 Record） */
function statusLabel(status: SoftwareUpdateStatus): string {
  return UPDATE_STATUS_LABEL[status];
}

/** 状态 → Badge 语义色调 */
function badgeType(
  status: SoftwareUpdateStatus
): "success" | "warning" | "danger" | "info" | "neutral" {
  switch (status) {
    case "pending":
      return "warning";
    case "local_updated":
      return "info";
    case "missing":
      return "danger";
    case "no_upstream":
      return "neutral";
    default:
      return "neutral";
  }
}

/** 本地版本列的展示文本（pkgver-pkgrel，缺失时显示占位符） */
function localVersion(row: SoftwareUpdateItem): string {
  if (!row.local_pkgver) return "—";
  return row.local_pkgrel ? `${row.local_pkgver}-${row.local_pkgrel}` : row.local_pkgver;
}

/** 重新扫描：数据库版本差 + 本地磁盘 PKGBUILD，写回后调用即自动刷新状态 */
async function fetchEntries() {
  loading.value = true;
  try {
    entries.value = await updateApi.listSoftwareUpdates();
    selectedRows.value = [];
    syncToolbar();
  } catch (e) {
    addMessage(footer, "error", `加载软件更新列表失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

onMounted(fetchEntries);

function handleSelectionChange(rows: SoftwareUpdateItem[]) {
  selectedRows.value = rows;
}

/** 确定预览范围：优先选中行，否则取当前筛选下全部待写回项 */
function targetPkgnames(): string[] {
  const selected = selectedRows.value
    .filter((r) => r.status === "pending" || r.status === "missing")
    .map((r) => r.pkgname);
  if (selected.length > 0) return selected;
  return filteredEntries.value
    .filter((r) => r.status === "pending")
    .map((r) => r.pkgname);
}

/** 生成 dry-run diff 预览（此步骤绝不写文件） */
async function openPreview(pkgnames: string[]) {
  if (pkgnames.length === 0) {
    addMessage(footer, "warning", "没有待写回的软件包，可切换筛选条件或先执行上游检查");
    return;
  }
  showDiff.value = true;
  previewLoading.value = true;
  previews.value = [];
  try {
    previews.value = await updateApi.previewSoftwareUpdates(pkgnames);
  } catch (e) {
    addMessage(footer, "error", `生成变更预览失败: ${e}`);
    showDiff.value = false;
  } finally {
    previewLoading.value = false;
  }
}

/** 弹窗确认后才真正写回，并自动 rescan 同步 UI 与磁盘状态 */
async function handleConfirm() {
  const pkgnames = previews.value.filter((p) => !p.error).map((p) => p.pkgname);
  if (pkgnames.length === 0) return;
  applying.value = true;
  try {
    const results = await updateApi.applySoftwareUpdates(pkgnames);
    const ok = results.filter((r) => r.success).length;
    const failed = results.filter((r) => !r.success);
    if (ok > 0) {
      addMessage(footer, "success", `已写回 ${ok} 个 PKGBUILD，原文件备份为 PKGBUILD.bak`);
    }
    if (failed.length > 0) {
      const detail = failed
        .slice(0, 3)
        .map((f) => `${f.pkgname}: ${f.message}`)
        .join("；");
      addMessage(footer, "error", `${failed.length} 个写回失败：${detail}`);
    }
    showDiff.value = false;
    await fetchEntries();
  } catch (e) {
    addMessage(footer, "error", `执行更新失败: ${e}`);
  } finally {
    applying.value = false;
  }
}

/** 行操作是否可用：无上游数据时无法写回 */
function canUpdate(row: SoftwareUpdateItem): boolean {
  return !loading.value && row.status !== "no_upstream";
}
</script>

<template>
  <div>
    <PageToolbar
      v-model="searchQuery"
      :show-filter-button="false"
      @refresh="fetchEntries"
    >
      <template #filters>
        <select
          class="toolbar-filter-select"
          :value="statusFilter"
          @change="statusFilter = ($event.target as HTMLSelectElement).value as SoftwareUpdateStatus | ''"
        >
          <option value="">全部状态</option>
          <option v-for="(label, key) in UPDATE_STATUS_LABEL" :key="key" :value="key">
            {{ label }}
          </option>
        </select>
      </template>
      <button class="btn-icon btn-icon-info" :disabled="loading" title="重新扫描本地 PKGBUILD" @click="fetchEntries">
        <component :is="Icon.actionRefresh" :size="16" />
      </button>
      <button
        class="btn-icon btn-icon-success"
        :disabled="loading"
        title="预览并写回（选中项，或全部待写回项）"
        @click="openPreview(targetPkgnames())"
      >
        <component :is="Icon.updatePreview" :size="16" />
      </button>
    </PageToolbar>

    <div class="update-hint">
      <component :is="Icon.statusInfo" :size="14" />
      仅显示已标记「需更新」的软件包，浏览时不再重新比较版本；状态以本地 PKGBUILD 实际 pkgver 为准，写回后显示「已写回 · 待推送 AUR」。
    </div>

    <StandardizedTable
      key="software-update-table"
      :columns="columns"
      :data="filteredEntries"
      :pageSize="pageSize"
      :currentPage="currentPage"
      rowKey="software_id"
      showCheckbox
      showIndex
      striped
      hoverable
      :showPagination="false"
      emptyText="没有已标记需更新的软件包，请先在软件管理执行上游检查"
      @selection-change="handleSelectionChange"
    >
      <template #cell-pkgname="{ row }">
        <strong>{{ row.pkgname }}</strong>
      </template>
      <template #cell-aur_version="{ row }">
        {{ row.aur_version || "—" }}
      </template>
      <template #cell-upstream_version="{ row }">
        {{ row.upstream_version || "—" }}
      </template>
      <template #cell-local_version="{ row }">
        {{ localVersion(row) }}
      </template>
      <template #cell-status="{ row }">
        <StandardizedBadge
          :type="badgeType(row.status)"
          :text="statusLabel(row.status)"
          size="sm"
        />
      </template>
      <template #actions="{ row }">
        <button
          class="btn-icon btn-icon-success"
          :disabled="!canUpdate(row)"
          title="预览并写回该包"
          @click="openPreview([row.pkgname])"
        >
          <component :is="Icon.updateApply" :size="14" />
        </button>
      </template>
    </StandardizedTable>

    <UpdateDiffDialog
      :show="showDiff"
      :loading="previewLoading"
      :previews="previews"
      :applying="applying"
      @close="showDiff = false"
      @confirm="handleConfirm"
    />
  </div>
</template>

<style scoped>
.update-hint {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.45rem 1rem;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  color: var(--text-secondary);
  font-size: 0.8125rem;
}
</style>
