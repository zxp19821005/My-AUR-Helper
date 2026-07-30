<!--
  LogViewer.vue - 日志查看器页面

  功能：
  - 显示应用日志列表（时间、级别、消息）
  - 支持搜索、分页
  - 支持按级别筛选
  - 支持清空日志

  使用组件：
  - StandardizedTable: 表格组件
  - PageToolbar: 页面工具栏
  - StandardizedSelect: 级别筛选下拉框
  - StandardizedBadge: 状态徽章
-->
<script setup lang="ts">
import { ref, onMounted, computed, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FOOTER_KEY, addMessage } from "../composables/footer";
import { Trash2 } from "@lucide/vue";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedSelect from "../components/base/StandardizedSelect.vue";
import StandardizedBadge from "../components/base/StandardizedBadge.vue";

const footer = inject(FOOTER_KEY)!;

interface LogEntry {
  id: number;
  timestamp: string;
  level: string;
  message: string;
}

const logs = ref<LogEntry[]>([]);
const loading = ref(false);
const searchQuery = ref("");
const levelFilter = ref("");
const pageSize = ref(50);

onMounted(async () => {
  await loadLogs();
});

async function loadLogs() {
  loading.value = true;
  try {
    logs.value = await invoke<LogEntry[]>("get_logs");
  } catch (e) {
    addMessage(footer, "error", `加载日志失败: ${e}`);
  } finally {
    loading.value = false;
  }
}

async function clearLogs() {
  if (!confirm("确定要清空所有日志吗？")) return;
  loading.value = true;
  try {
    await invoke("clear_logs");
    addMessage(footer, "success", "已清空日志");
    logs.value = [];
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
        log.timestamp.toLowerCase().includes(query)
    );
  }
  return result;
});

/** 获取级别对应的徽章类型 */
function getLevelType(level: string): "info" | "success" | "warning" | "danger" {
  switch (level.toLowerCase()) {
    case "info":
      return "info";
    case "warn":
    case "warning":
      return "warning";
    case "error":
      return "danger";
    default:
      return "info";
  }
}

/** 表格列配置 */
const columns = [
  { key: "timestamp", title: "时间" },
  { key: "level", title: "级别" },
  { key: "message", title: "消息" },
];
</script>

<template>
  <div>
    <PageToolbar v-model="searchQuery" @refresh="loadLogs">
      <template #right>
        <StandardizedSelect
          v-model="levelFilter"
          size="md"
        >
          <option value="">全部级别</option>
          <option value="info">INFO</option>
          <option value="warning">WARNING</option>
          <option value="error">ERROR</option>
        </StandardizedSelect>
      </template>
      <button
        class="btn-icon btn-icon-danger"
        :disabled="loading"
        @click="clearLogs"
        title="清空日志"
      >
        <Trash2 :size="16" />
      </button>
    </PageToolbar>

    <!-- 日志表格 -->
    <StandardizedTable
      :columns="columns"
      :data="filteredLogs"
      :pageSize="pageSize"
      rowKey="id"
      showIndex
      striped
      hoverable
      emptyText="暂无日志数据"
    >
      <!-- 时间列 -->
      <template #cell-timestamp="{ row }">
        <span class="timestamp">{{ row.timestamp }}</span>
      </template>

      <!-- 级别列 -->
      <template #cell-level="{ row }">
        <StandardizedBadge
          :text="row.level"
          :type="getLevelType(row.level)"
          size="sm"
        />
      </template>

      <!-- 消息列 -->
      <template #cell-message="{ row }">
        <span class="message">{{ row.message }}</span>
      </template>
    </StandardizedTable>
  </div>
</template>

<style scoped>
.timestamp {
  color: var(--text-secondary);
  font-size: 0.8125rem;
  font-family: monospace;
}

.message {
  color: var(--text-primary);
  font-size: 0.875rem;
  word-break: break-word;
}
</style>