<!--
  ProxySettings.vue - 代理管理页面

  功能：
  - 显示代理列表（名称、URL、类型、成功次数、失败次数、平均延迟、操作）
  - 支持搜索、分页、多选
  - 提供批量操作：获取代理文件、解析代理文件、代理测试
  - 支持单行操作：详情、编辑、测试、删除

  使用组件：
  - StandardizedTable: 表格组件
  - StandardizedMessage: 消息提示
  - StandardizedModal: 模态框（编辑/详情）
-->
<script setup lang="ts">
import { onMounted, ref, inject, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProxyList, PROXY_TYPE_OPTIONS, getProxyDisplayName } from "../composables/useProxyList";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import type { ProxyTestResult } from "../composables/useProxyList";
import type { ProxyInfo } from "../types";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedMessage from "../components/base/StandardizedMessage.vue";
import StandardizedBadge from "../components/base/StandardizedBadge.vue";
import StandardizedModal from "../components/common/StandardizedModal.vue";
import PageToolbar from "../components/common/PageToolbar.vue";
import { Trash2, Download, FileCode, Zap, Filter, X, Info, Edit, TestTube } from "@lucide/vue";

const footer = inject(FOOTER_KEY)!;

const showFilterBar = ref(false);
const activeFilterCount = computed(() => typeFilter.value ? 1 : 0);

const {
  searchQuery,
  selectedIds,
  loading,
  filteredEntries,
  pageSize,
  currentPage,
  typeFilter,
  testingIds,
  testResults,
  fetchEntries,
  syncToolbar,
  deleteSelectedProxies,
  setTestResult,
  updateProxy,
} = useProxyList();

const downloading = ref(false);
const parsing = ref(false);

const clearing = ref(false);
const showClearConfirm = ref(false);

const messageText = ref("");
const messageType = ref<"success" | "error" | "warning" | "info">("info");

// 编辑/详情模态框状态
const showEditModal = ref(false);
const showDetailModal = ref(false);
const editingProxy = ref<ProxyInfo | null>(null);
const detailProxy = ref<ProxyInfo | null>(null);
const editProxyName = ref("");

onMounted(async () => {
  await fetchEntries();
  syncToolbar();
});

async function handleDownloadProxyFile() {
  downloading.value = true;
  try {
    const count = await invoke<number>("download_proxy_file");
    addMessage(footer, "success", `成功下载代理文件，获取到 ${count} 个代理`);
    await fetchEntries();
  } catch (e) {
    addMessage(footer, "error", `下载失败: ${e}`);
  } finally {
    downloading.value = false;
  }
}

async function handleParseProxyFile() {
  parsing.value = true;
  try {
    const count = await invoke<number>("parse_proxy_file");
    addMessage(footer, "success", `成功解析代理文件，新增 ${count} 个代理`);
    await fetchEntries();
  } catch (e) {
    addMessage(footer, "error", `解析失败: ${e}`);
  } finally {
    parsing.value = false;
  }
}

async function handleClearProxyTables() {
  showClearConfirm.value = false;
  clearing.value = true;
  try {
    const count = await invoke<number>("clear_proxy_tables");
    addMessage(footer, "success", `已清空 ${count} 个代理记录，proxy_id 已重置`);
    selectedIds.value = new Set();
    await fetchEntries();
  } catch (e) {
    addMessage(footer, "error", `清空失败: ${e}`);
  } finally {
    clearing.value = false;
  }
}

async function handleTestProxies() {
  const proxyIds = selectedIds.value.size > 0
    ? Array.from(selectedIds.value)
    : [];

  if (proxyIds.length === 0 && selectedIds.value.size === 0) {
    addMessage(footer, "info", "开始测试所有代理...");
  } else {
    addMessage(footer, "info", `开始测试 ${proxyIds.length} 个选中代理...`);
  }

  try {
    const results = await invoke<ProxyTestResult[]>("test_proxies_batch", {
      proxyIds: proxyIds.length > 0 ? proxyIds : null,
    });

    for (const result of results) {
      setTestResult(result.proxy_id, result);
    }

    const successCount = results.filter(r => r.success).length;
    const failCount = results.filter(r => !r.success).length;
    addMessage(footer, "success", `测试完成: ${successCount} 个成功, ${failCount} 个失败`);
  } catch (e) {
    addMessage(footer, "error", `测试失败: ${e}`);
  }
}

async function handleTestSingleProxy(proxyId: number) {
  testingIds.value.add(proxyId);
  testingIds.value = new Set(testingIds.value);

  try {
    const result = await invoke<ProxyTestResult>("test_proxy_single", { proxyId });
    setTestResult(proxyId, result);
    if (result.success) {
      addMessage(footer, "success", `代理测试成功: ${result.latency}ms`);
    } else {
      addMessage(footer, "warning", `代理测试失败: ${result.error}`);
    }
  } catch (e) {
    setTestResult(proxyId, {
      proxy_id: proxyId,
      success: false,
      latency: null,
      error: String(e),
      test_url: "",
    });
    addMessage(footer, "error", `测试失败: ${e}`);
  } finally {
    testingIds.value.delete(proxyId);
    testingIds.value = new Set(testingIds.value);
  }
}

async function handleDeleteSelected() {
  if (selectedIds.value.size === 0) return;
  if (!confirm(`确定要删除选中的 ${selectedIds.value.size} 个代理吗？`)) return;
  loading.value = true;
  try {
    await deleteSelectedProxies();
    addMessage(footer, "success", `已删除 ${selectedIds.value.size} 个代理`);
    await fetchEntries();
  } catch (e) {
    addMessage(footer, "error", `删除失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

function handleDeleteSingleProxy(row: any) {
  if (row.proxy_id !== null) {
    selectedIds.value = new Set([row.proxy_id]);
    handleDeleteSelected();
  }
}

function openEditModal(proxy: ProxyInfo) {
  editingProxy.value = proxy;
  editProxyName.value = proxy.proxy_name;
  showEditModal.value = true;
}

function closeEditModal() {
  showEditModal.value = false;
  editingProxy.value = null;
  editProxyName.value = "";
}

async function handleSaveEdit() {
  if (!editingProxy.value || !editingProxy.value.proxy_id) return;
  try {
    await updateProxy(editingProxy.value.proxy_id, {
      proxy_name: editProxyName.value.trim(),
    });
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

function getProxyTypeClass(type: string): string {
  return `type-${type}`;
}

/**
 * 平均延迟列的展示文本
 * 优先级：本地最新测试结果 > 数据库持久化的最后测试状态 > 平均延迟 > 未测试。
 * 这样刷新页面后，失败的代理依然显示红色「失败」。
 */
function latencyText(row: any): string {
  const r = testResults.value.get(row.proxy_id);
  if (r) {
    return r.success ? `${r.latency}ms` : "失败";
  }
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
    <!-- 消息提示 -->
    <StandardizedMessage
      v-if="messageText"
      :type="messageType"
      :message="messageText"
      :duration="3000"
      @close="messageText = ''"
    />

    <!-- 工具栏 -->
    <PageToolbar 
      v-model="searchQuery" 
      @refresh="fetchEntries"
      :filter-active="activeFilterCount > 0"
      @toggle-filter="showFilterBar = !showFilterBar"
    >
      <template #filter-icon>
        <Filter :size="16" />
        <span v-if="activeFilterCount > 0" class="filter-count-badge">{{ activeFilterCount }}</span>
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

    <!-- 筛选面板 -->
    <Teleport to="body">
      <div v-if="showFilterBar" class="filter-overlay" @click.self="showFilterBar = false">
        <div class="filter-panel">
          <div class="filter-header">
            <div class="filter-title">
              <Filter :size="16" />
              <span>筛选条件</span>
              <span v-if="activeFilterCount > 0" class="filter-badge">{{ activeFilterCount }}</span>
            </div>
            <button class="btn-icon btn-icon-default" @click="showFilterBar = false">
              <X :size="16" />
            </button>
          </div>
          <div class="filter-body">
            <div class="filter-section">
              <div class="filter-row">
                <div class="filter-field">
                  <label class="filter-field-label">代理类型</label>
                  <select 
                    class="filter-select"
                    :value="typeFilter"
                    @change="typeFilter = ($event.target as HTMLSelectElement).value as any"
                  >
                    <option v-for="opt in PROXY_TYPE_OPTIONS" :key="opt.value" :value="opt.value">
                      {{ opt.label }}
                    </option>
                  </select>
                </div>
              </div>
            </div>
          </div>
          <div class="filter-footer">
            <button class="btn btn-secondary" @click="typeFilter = ''">
              清空筛选
            </button>
            <button class="btn btn-primary" @click="showFilterBar = false">
              应用筛选
            </button>
          </div>
        </div>
      </div>
    </Teleport>

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

    <!-- 编辑代理弹窗 -->
    <StandardizedModal
      :show="showEditModal"
      title="编辑代理"
      width="sm"
      @close="closeEditModal"
    >
      <div class="modal-form">
        <div class="form-group">
          <label class="form-label">代理名称</label>
          <input
            v-model="editProxyName"
            type="text"
            class="form-input"
            placeholder="输入代理名称"
          />
        </div>
      </div>
      <template #footer>
        <div class="modal-footer-actions">
          <button class="btn btn-secondary" @click="closeEditModal">取消</button>
          <button class="btn btn-primary" @click="handleSaveEdit">保存</button>
        </div>
      </template>
    </StandardizedModal>

    <!-- 代理详情弹窗 -->
    <StandardizedModal
      :show="showDetailModal"
      title="代理详情"
      width="md"
      @close="closeDetailModal"
    >
      <div v-if="detailProxy" class="detail-content">
        <div class="detail-row">
          <span class="detail-label">名称</span>
          <span class="detail-value">{{ getProxyDisplayName(detailProxy) }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">URL</span>
          <span class="detail-value url-value">{{ detailProxy.url }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">代理类型</span>
          <StandardizedBadge
            :text="detailProxy.proxy_type"
            :class="getProxyTypeClass(detailProxy.proxy_type)"
            size="sm"
            variant="soft"
          />
        </div>
        <div class="detail-row">
          <span class="detail-label">状态</span>
          <StandardizedBadge
            :type="detailProxy.is_active ? 'success' : 'neutral'"
            :text="detailProxy.is_active ? '启用' : '禁用'"
            size="sm"
          />
        </div>
        <div class="detail-row">
          <span class="detail-label">成功次数</span>
          <span class="detail-value">{{ detailProxy.success_count ?? 0 }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">失败次数</span>
          <span class="detail-value">{{ detailProxy.fail_count ?? 0 }}</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">平均延迟</span>
          <span class="detail-value">{{ detailProxy.avg_latency !== null ? `${detailProxy.avg_latency}ms` : '未测试' }}</span>
        </div>
      </div>
    </StandardizedModal>

    <!-- 清空代理表确认弹窗 -->
    <StandardizedModal
      :show="showClearConfirm"
      title="确认清空"
      width="sm"
      @close="showClearConfirm = false"
    >
      <div class="modal-form">
        <p class="confirm-text">确定要清空所有代理数据吗？此操作不可恢复。</p>
        <p class="confirm-sub">将清空 <code>proxies_info</code> 和 <code>proxies_test</code> 表，并重置 proxy_id。</p>
      </div>
      <template #footer>
        <div class="modal-footer-actions">
          <button class="btn btn-secondary" @click="showClearConfirm = false">取消</button>
          <button class="btn btn-danger" @click="handleClearProxyTables" :disabled="clearing">确定清空</button>
        </div>
      </template>
    </StandardizedModal>
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

.confirm-text { font-size: 14px; color: var(--text-primary, #374151); margin-bottom: 8px; }
.confirm-sub { font-size: 12px; color: var(--text-muted, #6b7280); }
.confirm-sub code { background: var(--bg-secondary, #f3f4f6); padding: 1px 4px; border-radius: 3px; font-size: 11px; }

/* ── 编辑弹窗 ── */
.modal-form { padding: 4px 0; }
.form-group { display: flex; flex-direction: column; gap: 6px; }
.form-label { font-size: 13px; font-weight: 500; color: var(--text-primary, #374151); }
.form-input {
  padding: 7px 10px;
  border: 1px solid var(--border-color, #d1d5db);
  border-radius: 6px;
  font-size: 13px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #374151);
  outline: none;
}
.form-input:focus { border-color: var(--color-primary, #7c3aed); box-shadow: 0 0 0 2px rgba(124,58,237,.15); }

.modal-footer-actions { display: flex; justify-content: flex-end; gap: 8px; }

/* ── 详情弹窗 ── */
.detail-content { display: flex; flex-direction: column; gap: 12px; padding: 4px 0; }
.detail-row { display: flex; align-items: center; gap: 12px; }
.detail-label {
  min-width: 80px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted, #6b7280);
  flex-shrink: 0;
}
.detail-value { font-size: 13px; color: var(--text-primary, #374151); }
.url-value { word-break: break-all; }
</style>