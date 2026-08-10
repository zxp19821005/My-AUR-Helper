<!--
  ProxySettings.vue - 代理管理页面

  功能：
  - 显示代理列表（名称、URL、类型、成功/失败次数、平均延迟、操作）
  - 支持搜索、分页、多选、类型筛选
  - 批量操作：获取/解析代理文件、代理测试、清空、删除
  - 单行操作：详情、编辑、测试、删除

  拆分说明：批量/单行操作逻辑抽到 useProxyActions，
  弹窗 UI 抽到 components/proxy/ 下的三个子组件，本文件只负责编排与表格渲染。
-->
<script setup lang="ts">
import { onMounted, ref, inject } from "vue";
import { useProxyList, getProxyDisplayName, PROXY_TYPE_OPTIONS } from "../composables/useProxyList";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import { useProxyActions } from "../composables/useProxyActions";
import type { ProxyInfo } from "../types";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedBadge from "../components/base/StandardizedBadge.vue";
import PageToolbar from "../components/common/PageToolbar.vue";
import ProxyEditModal from "../components/proxy/ProxyEditModal.vue";
import ProxyDetailModal from "../components/proxy/ProxyDetailModal.vue";
import ProxyClearConfirmModal from "../components/proxy/ProxyClearConfirmModal.vue";
import { Trash2, Download, FileCode, Zap, Database, Info, Edit, TestTube } from "@lucide/vue";

const footer = inject(FOOTER_KEY)!;
const list = useProxyList();
const {
  searchQuery, selectedIds, loading, filteredEntries, pageSize, currentPage,
  typeFilter, testingIds, testResults, fetchEntries, syncToolbar,
} = list;
const {
  downloading, parsing, clearing, showClearConfirm,
  handleDownloadProxyFile, handleParseProxyFile, handleClearProxyTables,
  handleTestProxies, handleTestSingleProxy, handleDeleteSelected, handleDeleteSingleProxy,
} = useProxyActions(list);

const editingProxy = ref<ProxyInfo | null>(null);
const detailProxy = ref<ProxyInfo | null>(null);
const showEditModal = ref(false);
const showDetailModal = ref(false);

onMounted(async () => {
  await fetchEntries();
  syncToolbar();
});

function openEditModal(proxy: ProxyInfo) {
  editingProxy.value = proxy;
  showEditModal.value = true;
}
function closeEditModal() {
  showEditModal.value = false;
  editingProxy.value = null;
}
async function handleSaveEdit(name: string) {
  if (!editingProxy.value?.proxy_id) return;
  try {
    await list.updateProxy(editingProxy.value.proxy_id, { proxy_name: name });
    addMessage(footer, "success", "代理名称已更新");
    closeEditModal();
  } catch (e) {
    addMessage(footer, "error", `更新失败: ${e}`);
  }
}
function openDetailModal(proxy: ProxyInfo) {
  detailProxy.value = proxy;
  showDetailModal.value = true;
}
function closeDetailModal() {
  showDetailModal.value = false;
  detailProxy.value = null;
}

/** 代理类型对应的样式类后缀 */
function getProxyTypeClass(type: string): string {
  return `type-${type}`;
}

/**
 * 平均延迟列的展示文本
 * 优先级：本地最新测试结果 > 数据库持久化的最后测试状态 > 平均延迟 > 未测试。
 */
function latencyText(row: any): string {
  const r = testResults.value.get(row.proxy_id);
  if (r) return r.success ? `${r.latency}ms` : "失败";
  if (row.last_test_status === "fail") return "失败";
  if (row.last_test_status === "success") {
    return row.avg_latency != null ? `${row.avg_latency}ms` : "已测";
  }
  return row.avg_latency != null ? `${row.avg_latency}ms` : "未测试";
}

/** 平均延迟列的样式类（成功绿 / 失败红 / 未测试灰） */
function latencyClass(row: any): string {
  const r = testResults.value.get(row.proxy_id);
  if (r) return r.success ? "latency-ok" : "latency-fail";
  if (row.last_test_status === "fail") return "latency-fail";
  if (row.last_test_status === "success") return "latency-ok";
  return row.avg_latency != null ? "latency-ok" : "latency-untested";
}

function handleSelectionChange(selectedRows: any[]) {
  const newSelected = new Set<number>();
  selectedRows.forEach((row) => {
    if (row.proxy_id !== null) newSelected.add(row.proxy_id);
  });
  selectedIds.value = newSelected;
}

const columns = [
  { key: "proxy_name", title: "名称" },
  { key: "url", title: "URL" },
  { key: "proxy_type", title: "代理类型" },
  { key: "success_count", title: "成功次数" },
  { key: "fail_count", title: "失败次数" },
  { key: "avg_latency", title: "平均延迟" },
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
          :value="typeFilter"
          @change="typeFilter = ($event.target as HTMLSelectElement).value as any"
        >
          <option v-for="opt in PROXY_TYPE_OPTIONS" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </template>
      <button class="btn-icon btn-icon-danger" :disabled="clearing" @click="showClearConfirm = true" title="清空代理表">
        <Database :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" :disabled="downloading" @click="handleDownloadProxyFile" title="获取代理文件">
        <Download :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" :disabled="parsing" @click="handleParseProxyFile" title="解析代理文件">
        <FileCode :size="16" />
      </button>
      <button class="btn-icon btn-icon-success" :disabled="loading" @click="handleTestProxies" title="代理测试">
        <Zap :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" :disabled="selectedIds.size === 0" @click="handleDeleteSelected" title="删除选中">
        <Trash2 :size="16" />
      </button>
    </PageToolbar>

    <!-- 代理表格 -->
    <StandardizedTable
      :key="`table-${filteredEntries.length}`"
      :columns="columns"
      :data="filteredEntries"
      :pageSize="pageSize"
      :currentPage="currentPage"
      rowKey="proxy_id"
      showCheckbox
      showIndex
      striped
      hoverable
      :showPagination="false"
      emptyText="暂无代理数据"
      @selection-change="handleSelectionChange"
    >
      <template #cell-proxy_name="{ row }">
        <span class="proxy-name-text">{{ getProxyDisplayName(row) }}</span>
      </template>

      <template #cell-url="{ row }">
        <span class="cell-url-text">{{ row.url }}</span>
      </template>

      <template #cell-proxy_type="{ row }">
        <StandardizedBadge
          :text="row.proxy_type"
          :class="getProxyTypeClass(row.proxy_type)"
          size="sm"
          variant="soft"
        />
      </template>

      <template #cell-success_count="{ row }">
        <span class="stat-cell">{{ row.success_count ?? 0 }}</span>
      </template>

      <template #cell-fail_count="{ row }">
        <span class="stat-cell">{{ row.fail_count ?? 0 }}</span>
      </template>

      <template #cell-avg_latency="{ row }">
        <span class="latency-cell" :class="latencyClass(row)">
          {{ latencyText(row) }}
        </span>
      </template>

      <template #actions="{ row }">
        <div class="action-buttons">
          <button class="btn-icon btn-icon-default" @click="openDetailModal(row)" title="详情">
            <Info :size="14" />
          </button>
          <button class="btn-icon btn-icon-accent" @click="openEditModal(row)" title="编辑">
            <Edit :size="14" />
          </button>
          <button
            class="btn-icon btn-icon-success"
            :disabled="row.proxy_id !== null && testingIds.has(row.proxy_id)"
            @click="row.proxy_id !== null && handleTestSingleProxy(row.proxy_id)"
            title="测试"
          >
            <TestTube :size="14" />
          </button>
          <button class="btn-icon btn-icon-danger" @click="handleDeleteSingleProxy(row)" title="删除">
            <Trash2 :size="14" />
          </button>
        </div>
      </template>
    </StandardizedTable>

    <!-- 弹窗子组件 -->
    <ProxyEditModal :show="showEditModal" :proxy="editingProxy" @close="closeEditModal" @save="handleSaveEdit" />
    <ProxyDetailModal :show="showDetailModal" :proxy="detailProxy" @close="closeDetailModal" />
    <ProxyClearConfirmModal :show="showClearConfirm" :clearing="clearing" @close="showClearConfirm = false" @confirm="handleClearProxyTables" />
  </div>
</template>

<style scoped>
/* ── 操作按钮组 ── */
.action-buttons {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  white-space: nowrap;
}

/* ── 表格单元 ── */
.proxy-name-text { font-weight: 600; }
.cell-url-text {
  word-break: break-all;
  max-width: 360px;
  display: inline-block;
}
.stat-cell { font-variant-numeric: tabular-nums; }
.latency-cell { font-variant-numeric: tabular-nums; }
.latency-ok { color: var(--color-success, #16a34a); }
.latency-untested { color: var(--text-muted, #9ca3af); }
.latency-fail { color: var(--color-danger, #dc2626); font-weight: 600; }
</style>
