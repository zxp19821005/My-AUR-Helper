<!--
  LogViewer.vue - 日志查看器页面

  功能：
  - 从日志文件读取并显示应用日志（时间、级别、模块、消息）
  - 支持搜索、按级别筛选
  - 支持清空当天日志
  - 自动刷新（每 5 秒）
-->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import { Trash2, RefreshCw, Pause, Play } from "@lucide/vue";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedSelect from "../components/base/StandardizedSelect.vue";
import StandardizedBadge from "../components/base/StandardizedBadge.vue";

const footer = inject(FOOTER_KEY)!;

interface LogEntry {
  timestamp: string;
  level: string;
  module: string;
  message: string;
}

const logs = ref<LogEntry[]>([]);
const loading = ref(false);
const searchQuery = ref("");
const levelFilter = ref("");
const autoRefresh = ref(true);
const refreshInterval = ref<number | null>(null);

onMounted(async () => {
  await loadLogs();
  startAutoRefresh();
});

onUnmounted(() => {
  stopAutoRefresh();
});

function startAutoRefresh() {
  stopAutoRefresh();
  if (autoRefresh.value) {
    refreshInterval.value = window.setInterval(() => {
      loadLogs();
    }, 5000);
  }
}

function stopAutoRefresh() {
  if (refreshInterval.value !== null) {
    clearInterval(refreshInterval.value);
    refreshInterval.value = null;
  }
}

function toggleAutoRefresh() {
  autoRefresh.value = !autoRefresh.value;
  if (autoRefresh.value) {
    startAutoRefresh();
  } else {
    stopAutoRefresh();
  }
}

async function loadLogs() {
  loading.value = true;
  try {
    logs.value = await invoke<LogEntry[]>("get_logs", { limit: 500 });
  } catch (e) {
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
    await loadLogs();
  } catch (e) {
    addMessage(footer, "error", `清空日志失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

/** 过滤后的日志 */
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

/** 获取级别对应的徽章类型 */
function getLevelType(level: string): "info" | "success" | "warning" | "danger" {
  switch (level.toLowerCase()) {
    case "错误":
    case "error":
      return "danger";
    case "警告":
    case "warn":
    case "warning":
      return "warning";
    case "信息":
    case "info":
      return "info";
    case "调试":
    case "debug":
      return "info";
    default:
      return "info";
  }
}

/** 表格列配置 */
const columns = [
  { key: "timestamp", title: "时间" },
  { key: "level", title: "级别" },
  { key: "module", title: "模块" },
  { key: "message", title: "消息" },
];
</script>

<template>
  <div>
    <PageToolbar v-model="searchQuery" @refresh="loadLogs">
      <template #right>
        <StandardizedSelect v-model="levelFilter" size="md">
          <option value="">全部级别</option>
          <option value="信息">INFO</option>
          <option value="警告">WARNING</option>
          <option value="错误">ERROR</option>
          <option value="调试">DEBUG</option>
        </StandardizedSelect>
        <button
          class="btn-icon btn-icon-secondary"
          :disabled="loading"
          @click="toggleAutoRefresh"
          :title="autoRefresh ? '暂停自动刷新' : '启用自动刷新'"
        >
          <Pause v-if="autoRefresh" :size="16" />
          <Play v-else :size="16" />
        </button>
        <button
          class="btn-icon btn-icon-secondary"
          :disabled="loading"
          @click="loadLogs"
          title="手动刷新"
        >
          <RefreshCw :size="16" />
        </button>
        <button
          class="btn-icon btn-icon-danger"
          :disabled="loading"
          @click="clearLogs"
          title="清空当天日志"
        >
          <Trash2 :size="16" />
        </button>
      </template>
    </PageToolbar>

    <!-- 状态提示 -->
    <div class="log-status">
      <span class="status-item">
        <span class="status-dot" :class="{ active: autoRefresh }"></span>
        {{ autoRefresh ? '自动刷新中（每5秒）' : '已暂停自动刷新' }}
      </span>
      <span class="status-item">
        共 {{ filteredLogs.length }} 条日志
      </span>
    </div>

    <!-- 日志表格 -->
    <StandardizedTable
      :columns="columns"
      :data="filteredLogs"
      rowKey="timestamp"
      showIndex
      striped
      hoverable
      emptyText="暂无日志数据"
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