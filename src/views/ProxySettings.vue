<!--
  ProxySettings.vue - 代理管理页面

  功能：
  - 显示代理列表（含名称、URL、类型、状态、测试结果）
  - 支持搜索、分页、多选
  - 提供批量操作：获取代理文件、解析代理文件、代理测试
  - 支持单行操作：启用/禁用、测试、删除

  使用组件：
  - ProxyToolbar: 工具栏组件
  - ProxyRowActions: 行操作按钮组
  - StandardizedTable: 表格组件
  - StandardizedMessage: 消息提示
-->
<script setup lang="ts">
import { onMounted, ref, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProxyList } from "../composables/useProxyList";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import type { ProxyTestResult } from "../composables/useProxyList";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedMessage from "../components/base/StandardizedMessage.vue";
import StandardizedBadge from "../components/base/StandardizedBadge.vue";
import ProxyToolbar from "../components/ProxyToolbar.vue";
import ProxyRowActions from "../components/ProxyRowActions.vue";

const footer = inject(FOOTER_KEY)!;

const {
  searchQuery,
  selectedIds,
  loading,
  pageData,
  typeFilter,
  testingIds,
  fetchEntries,
  syncToolbar,
  toggleProxyActive,
  deleteSelectedProxies,
  setTestResult,
  getTestStatusText,
  getTestStatusClass,
} = useProxyList();

const downloading = ref(false);
const parsing = ref(false);

const messageText = ref("");
const messageType = ref<"success" | "error" | "warning" | "info">("info");

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

async function handleToggleProxy(proxy: any) {
  try {
    await toggleProxyActive(proxy);
  } catch (e) {
    addMessage(footer, "error", `操作失败: ${e}`);
  }
}

function handleDeleteSingleProxy(row: any) {
  if (row.proxy_id !== null) {
    selectedIds.value.delete(row.proxy_id);
    handleDeleteSelected();
  }
}

function getProxyTypeClass(type: string): string {
  return `type-${type}`;
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
  { key: "proxy_type", title: "类型" },
  { key: "is_active", title: "状态" },
  { key: "test_result", title: "测试结果" },
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
    <ProxyToolbar
      v-model:search-query="searchQuery"
      v-model:type-filter="typeFilter"
      :loading="loading"
      :downloading="downloading"
      :parsing="parsing"
      :selected-count="selectedIds.size"
      @download-proxy-file="handleDownloadProxyFile"
      @parse-proxy-file="handleParseProxyFile"
      @test-proxies="handleTestProxies"
      @delete-selected="handleDeleteSelected"
    />

    <!-- 代理表格 -->
    <StandardizedTable
      :columns="columns"
      :data="pageData"
      rowKey="proxy_id"
      showCheckbox
      showIndex
      striped
      hoverable
      emptyText="暂无代理数据"
      @selection-change="handleSelectionChange"
    >
      <template #cell-proxy_name="{ row }">
        <strong>{{ row.proxy_name }}</strong>
      </template>

      <template #cell-url="{ row }">
        <span class="cell-url">{{ row.url }}</span>
      </template>

      <template #cell-proxy_type="{ row }">
        <StandardizedBadge
          :text="row.proxy_type"
          :class="getProxyTypeClass(row.proxy_type)"
          size="sm"
          variant="soft"
        />
      </template>

      <template #cell-is_active="{ row }">
        <StandardizedBadge
          :type="row.is_active ? 'success' : 'neutral'"
          :text="row.is_active ? '启用' : '禁用'"
          size="sm"
        />
      </template>

      <template #cell-test_result="{ row }">
        <span
          v-if="row.proxy_id !== null"
          class="test-status"
          :class="getTestStatusClass(row.proxy_id)"
        >
          {{ getTestStatusText(row.proxy_id) }}
        </span>
        <span v-else class="test-not-tested">未测试</span>
      </template>

      <template #actions="{ row }">
        <ProxyRowActions
          :row="row"
          :testing-ids="testingIds"
          @toggle="handleToggleProxy"
          @test="handleTestSingleProxy"
          @delete="handleDeleteSingleProxy"
        />
      </template>
    </StandardizedTable>
  </div>
</template>

<style scoped>
/* 所有样式已移至全局 table-styles.css */
</style>