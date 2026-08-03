<!--
  LogViewer.vue - 日志查看器页面

  功能：
  - 从日志文件读取并显示应用日志（时间、级别、模块、消息）
  - 支持搜索、按级别筛选
  - 支持清空当天日志
  - 实时推送（Tauri 事件系统）

  布局：顶部工具栏（单行）+ 内容显示区域 + 底部分页工具栏
-->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, inject, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import { Trash2, X } from "@lucide/vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedBadge from "../components/base/StandardizedBadge.vue";
import type { Column } from "../composables/useTableState";
import { useSettingsStore } from "../stores/settings";

const footer = inject(FOOTER_KEY)!;
const settingsStore = useSettingsStore();

interface LogEntry {
  timestamp: string;
  level: string;
  module: string;
  message: string;
  _id?: number; // 唯一行ID，用于表格渲染
}

const logs = ref<LogEntry[]>([]);
const loading = ref(false);
const searchQuery = ref("");
const levelFilter = ref("");

// 从设置中读取每页显示行数
const pageSize = ref(12);
const currentPage = ref(1);

// 事件监听器
let unlisten: UnlistenFn | null = null;
let nextId = 0;

onMounted(async () => {
  // 从设置中读取每页显示行数
  pageSize.value = await settingsStore.getSettingNumber("list_page_size_log", 12);
  await loadInitialLogs();
  await startLogListener();
});

onUnmounted(() => {
  stopLogListener();
});

/** 监听 Tauri 日志事件 */
async function startLogListener() {
  unlisten = await listen("log-entry", (event) => {
    const payload = event.payload as LogEntry;
    if (payload) {
      // 新日志插入到开头（倒序：最新的在前）
      logs.value.unshift({
        ...payload,
        _id: nextId++,
      });
    }
  });
}

function stopLogListener() {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
}

/** 初始加载所有日志 */
async function loadInitialLogs() {
  loading.value = true;
  try {
    const result = await invoke<LogEntry[]>("get_logs", { limit: 500 });
    // 使用 JSON 序列化/反序列化来确保数据是普通对象
    const serialized = JSON.stringify(result);
    const parsed = JSON.parse(serialized);
    // 倒序排列：最新的日志在前
    logs.value = parsed.map((entry: any, index: number) => ({
      timestamp: String(entry.timestamp || ""),
      level: String(entry.level || ""),
      module: String(entry.module || ""),
      message: String(entry.message || ""),
      _id: index,
    })).reverse();
    nextId = logs.value.length;
  } catch (e) {
    console.error("加载日志失败:", e);
    addMessage(footer, "error", `加载日志失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function clearLogs() {
  if (!confirm("确定要清空当天日志吗？")) return;
  loading.value = true;
  try {
    await invoke("clear_logs");
    addMessage(footer, "success", "已清空当天日志");
    await loadInitialLogs();
  } catch (e) {
    addMessage(footer, "error", `清空日志失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

/** 同步底部工具栏状态 */
function syncFooter() {
  footer.infoText = `共 ${filteredLogs.value.length} 条日志`;
  footer.showPagination = filteredLogs.value.length > pageSize.value;
  footer.totalRecords = filteredLogs.value.length;
  footer.pageSize = pageSize.value;
  footer.currentPage = currentPage.value;
  footer.onPageChange = (page: number) => {
    currentPage.value = page;
  };
}

/** 过滤后的日志（保持时间正序） */
const filteredLogs = computed(() => {
  let result = logs.value;
  if (levelFilter.value) {
    result = result.filter((log) => log.level === levelFilter.value);
  }
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter(
      (log) =>
        log.message.toLowerCase().includes(query) ||
        log.timestamp.toLowerCase().includes(query) ||
        log.module.toLowerCase().includes(query)
    );
  }
  return result;
});

/** 监听筛选变化，重置页码 */
watch([levelFilter, searchQuery], () => {
  currentPage.value = 1;
});

/** 监听 pageSize 变化 */
watch(pageSize, () => {
  currentPage.value = 1;
});

/** 监听 filteredLogs 变化，同步底部工具栏 */
watch(filteredLogs, () => {
  syncFooter();
}, { deep: true });

/** 监听 currentPage 变化，同步底部工具栏 */
watch(currentPage, () => {
  syncFooter();
});

/** 清除级别筛选 */
function clearLevelFilter() {
  levelFilter.value = "";
}

/** 获取级别对应的徽章类型 */
function getLevelType(level: string): "info" | "success" | "warning" | "danger" {
  switch (level.toUpperCase()) {
    case "ERROR":
      return "danger";
    case "WARN":
    case "WARNING":
      return "warning";
    case "INFO":
      return "info";
    case "DEBUG":
    case "TRACE":
      return "info";
    default:
      return "info";
  }
}

/** 表格列配置（禁用排序，保持时间正序） */
const columns: Column[] = [
  { key: "timestamp", title: "时间", sortable: false },
  { key: "level", title: "级别", sortable: false },
  { key: "module", title: "模块", sortable: false },
  { key: "message", title: "消息", sortable: false },
];
</script>

<template>
  <div>
    <!-- 顶部工具栏（单行） -->
    <div class="log-toolbar">
      <div class="log-toolbar-left">
        <div class="level-filter-wrapper">
          <select
            v-model="levelFilter"
            class="level-select"
          >
            <option value="">全部级别</option>
            <option value="INFO">INFO</option>
            <option value="WARN">WARN</option>
            <option value="ERROR">ERROR</option>
            <option value="DEBUG">DEBUG</option>
          </select>
          <button
            v-if="levelFilter"
            class="btn-icon btn-icon-sm btn-icon-secondary"
            @click="clearLevelFilter"
            title="清除筛选"
          >
            <X :size="14" />
          </button>
        </div>
      </div>
      <div class="log-toolbar-right">
        <div class="search-box">
          <input
            v-model="searchQuery"
            type="text"
            class="search-input"
            placeholder="搜索日志..."
          />
        </div>
        <button
          class="btn-icon btn-icon-danger"
          :disabled="loading"
          @click="clearLogs"
          title="清空当天日志"
        >
          <Trash2 :size="16" />
        </button>
      </div>
    </div>

    <!-- 内容显示区域（禁用内置分页，使用全局 BottomToolbar） -->
    <StandardizedTable
      :key="`table-${filteredLogs.length}`"
      :columns="columns"
      :data="filteredLogs"
      :pageSize="pageSize"
      :currentPage="currentPage"
      rowKey="_id"
      showIndex
      striped
      hoverable
      emptyText="暂无日志数据"
      :showPagination="false"
      @page-change="(page: number) => { currentPage = page; }"
    >
      <template #cell-timestamp="{ row }">
        <span class="log-timestamp">{{ row.timestamp }}</span>
      </template>
      <template #cell-level="{ row }">
        <StandardizedBadge :text="row.level" :type="getLevelType(row.level)" size="sm" />
      </template>
      <template #cell-module="{ row }">
        <span class="log-module">{{ row.module || '-' }}</span>
      </template>
      <template #cell-message="{ row }">
        <span class="log-message">{{ row.message }}</span>
      </template>
    </StandardizedTable>
  </div>
</template>

<style scoped>
.log-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 1.25rem;
  border-bottom: 1px solid var(--border);
  background-color: var(--bg-primary);
  min-height: 44px;
}
.log-toolbar-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.log-toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}
.level-filter-wrapper {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}
.level-select {
  padding: 0.375rem 0.75rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--bg-card);
  color: var(--text-primary);
  font-size: 0.8125rem;
  cursor: pointer;
  transition: border-color 0.15s;
}
.level-select:hover {
  border-color: var(--accent);
}
.level-select:focus {
  outline: none;
  border-color: var(--accent);
}
.search-box {
  display: flex;
  align-items: center;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--bg-card);
  transition: border-color 0.15s;
}
.search-box:focus-within {
  border-color: var(--accent);
}
.search-input {
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: 0.8125rem;
  outline: none;
  width: 140px;
}
.search-input::placeholder {
  color: var(--text-muted);
}
</style>