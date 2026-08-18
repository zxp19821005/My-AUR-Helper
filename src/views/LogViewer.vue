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
import { openConfirm as confirm } from "../composables/useConfirm";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedBadge from "../components/base/StandardizedBadge.vue";
import LogToolbar from "../components/common/LogToolbar.vue";
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

/** 日志内存上限：超过后丢弃最旧条目，防止长会话内存无限增长 */
const MAX_LOG_ENTRIES = 1000;

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
      // 限制内存占用：只保留最近 MAX_LOG_ENTRIES 条
      if (logs.value.length > MAX_LOG_ENTRIES) {
        logs.value.length = MAX_LOG_ENTRIES;
      }
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
  if (!(await confirm({ message: "确定要清空当天日志吗？", variant: "danger" }))) return;
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

/** 监听筛选变化或日志条数变化，同步底部工具栏。
 *  注意：不要 deep 监听 filteredLogs 整数组——实时日志流下每条新日志都会
 *  触发全量深比较，叠加表格重建造成 WebKitWebProcess 持续高 CPU。 */
watch([levelFilter, searchQuery, () => logs.value.length], () => {
  syncFooter();
});

/** 监听 currentPage 变化，同步底部工具栏 */
watch(currentPage, () => {
  syncFooter();
});

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
    <LogToolbar
      v-model:level-filter="levelFilter"
      v-model:search-query="searchQuery"
      :loading="loading"
      @clear-logs="clearLogs"
    />

    <!-- 内容显示区域（禁用内置分页，使用全局 BottomToolbar）。
         注意：key 必须固定——若绑定 filteredLogs.length，每条新日志都会销毁重建
         整个表格（含全部日志行），实时日志流下 CPU 持续飙升。 -->
    <StandardizedTable
      key="log-table"
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
