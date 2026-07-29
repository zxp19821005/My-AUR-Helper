<!--
  ProxySettings.vue - 代理管理页面

  功能：
  - 显示代理列表（含名称、URL、类型、状态、测试结果）
  - 支持搜索、分页、多选
  - 提供批量操作：获取代理文件、解析代理文件、代理测试
  - 支持单行操作：启用/禁用、测试、删除
  - 支持类型筛选
-->
<script setup lang="ts">
import { onMounted, ref, inject, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useProxyList, PROXY_TYPE_OPTIONS } from "../composables/useProxyList";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import PageToolbar from "../components/PageToolbar.vue";
import { Trash2, Download, FileCode, Zap, Globe } from "@lucide/vue";
import type { ProxyTestResult } from "../composables/useProxyList";

const footer = inject(FOOTER_KEY)!;

const {
  searchQuery,
  selectedIds,
  loading,
  pageData,
  typeFilter,
  testingIds,
  fetchEntries,
  toggleSelect,
  toggleSelectAll,
  syncToolbar,
  toggleProxyActive,
  deleteSelectedProxies,
  setTestResult,
  getTestStatusText,
  getTestStatusClass,
} = useProxyList();

/** 下载中状态 */
const downloading = ref(false);
/** 解析中状态 */
const parsing = ref(false);

/** 全选状态计算 */
const isAllPageSelected = computed(() => {
  return pageData.value.length > 0 && pageData.value.every(p => p.proxy_id !== null && selectedIds.value.has(p.proxy_id));
});

const isPartialPageSelected = computed(() => {
  return pageData.value.some(p => p.proxy_id !== null && selectedIds.value.has(p.proxy_id)) && !isAllPageSelected.value;
});

onMounted(async () => {
  await fetchEntries();
  syncToolbar();
});

/**
 * 获取代理文件
 * 从设置中配置的 URL 下载 JS 文件到本地
 */
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

/**
 * 解析代理文件
 * 读取已下载的 JS 文件，解析代理规则并写入数据库
 */
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

/**
 * 代理测试
 * 测试选中的代理或所有代理
 */
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

    // 更新测试结果
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

/**
 * 单行测试代理
 */
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

/**
 * 删除选中的代理
 */
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

/**
 * 切换代理启用状态
 */
async function handleToggleProxy(proxy: any) {
  try {
    await toggleProxyActive(proxy);
  } catch (e) {
    addMessage(footer, "error", `操作失败: ${e}`);
  }
}
</script>

<template>
  <div>
    <PageToolbar v-model="searchQuery" @refresh="fetchEntries">
      <template #right>
        <select v-model="typeFilter" class="type-filter-select">
          <option v-for="opt in PROXY_TYPE_OPTIONS" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </template>
      <button class="btn-icon btn-icon-accent" @click="handleDownloadProxyFile" :disabled="downloading" title="获取代理文件">
        <Download :size="16" />
      </button>
      <button class="btn-icon btn-icon-accent" @click="handleParseProxyFile" :disabled="parsing" title="解析代理文件">
        <FileCode :size="16" />
      </button>
      <button class="btn-icon btn-icon-info" @click="handleTestProxies" :disabled="loading" title="代理测试">
        <Zap :size="16" />
      </button>
      <button class="btn-icon btn-icon-danger" @click="handleDeleteSelected" :disabled="selectedIds.size === 0" title="删除选中">
        <Trash2 :size="16" />
      </button>
    </PageToolbar>

    <div class="card" style="overflow-x: auto; padding: 0">
      <table class="pkg-table">
        <thead>
          <tr>
            <th style="width: 2rem">
              <input type="checkbox"
                :checked="isAllPageSelected"
                :indeterminate="isPartialPageSelected"
                @change="toggleSelectAll" />
            </th>
            <th>名称</th>
            <th>URL</th>
            <th>类型</th>
            <th>状态</th>
            <th>测试结果</th>
            <th style="min-width: 180px">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="proxy in pageData" :key="proxy.proxy_id ?? 0"
            :class="{ 'row-selected': proxy.proxy_id !== null && selectedIds.has(proxy.proxy_id) }">
            <td @click.stop>
              <input type="checkbox" :checked="proxy.proxy_id !== null && selectedIds.has(proxy.proxy_id)"
                @change="proxy.proxy_id !== null && toggleSelect(proxy.proxy_id)" />
            </td>
            <td>
              <strong>{{ proxy.proxy_name }}</strong>
            </td>
            <td class="cell-url">{{ proxy.url }}</td>
            <td>
              <span class="proxy-type-badge" :class="`type-${proxy.proxy_type}`">
                {{ proxy.proxy_type }}
              </span>
            </td>
            <td>
              <span class="status-badge" :class="proxy.is_active ? 'status-active' : 'status-inactive'">
                {{ proxy.is_active ? "启用" : "禁用" }}
              </span>
            </td>
            <td>
              <span class="test-status" :class="proxy.proxy_id !== null ? getTestStatusClass(proxy.proxy_id) : ''">
                {{ proxy.proxy_id !== null ? getTestStatusText(proxy.proxy_id) : '未测试' }}
              </span>
            </td>
            <td>
              <div class="row-actions">
                <button class="btn-icon btn-icon-default" @click.stop="handleToggleProxy(proxy)" :title="proxy.is_active ? '禁用' : '启用'">
                  <Globe :size="14" />
                </button>
                <button class="btn-icon btn-icon-info" @click.stop="proxy.proxy_id !== null && handleTestSingleProxy(proxy.proxy_id)"
                  :disabled="proxy.proxy_id !== null && testingIds.has(proxy.proxy_id)" title="测试代理">
                  <Zap :size="14" />
                </button>
                <button class="btn-icon btn-icon-danger" @click.stop="proxy.proxy_id !== null && selectedIds.delete(proxy.proxy_id); handleDeleteSelected()" title="删除">
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
.type-filter-select {
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--bg-card);
  color: var(--text-primary);
  font-size: 0.8125rem;
  outline: none;
  cursor: pointer;
  min-width: 100px;
}

.type-filter-select:focus {
  border-color: var(--accent);
}

.cell-url {
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
  font-size: 0.8125rem;
}

.proxy-type-badge {
  display: inline-block;
  padding: 0.125rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: uppercase;
}

.type-download {
  background-color: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.type-clone {
  background-color: rgba(16, 185, 129, 0.1);
  color: #10b981;
}

.type-raw {
  background-color: rgba(139, 92, 246, 0.1);
  color: #8b5cf6;
}

.type-ssh {
  background-color: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.status-badge {
  display: inline-block;
  padding: 0.125rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 500;
}

.status-active {
  background-color: rgba(16, 185, 129, 0.1);
  color: #10b981;
}

.status-inactive {
  background-color: rgba(107, 114, 128, 0.1);
  color: #6b7280;
}

.test-status {
  font-size: 0.8125rem;
}

.status-success {
  color: #10b981;
}

.status-error {
  color: #ef4444;
}

.btn-icon-info {
  color: var(--text-secondary);
}

.btn-icon-info:hover {
  color: var(--accent);
}
</style>
