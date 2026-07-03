<!--
  LogViewer.vue - 日志查看页面

  功能：
  - 显示应用日志列表
  - 支持清空日志
  - 按日志级别着色显示（ERROR/WARN/INFO/DEBUG）

  数据来源：
  - get_logs: 获取日志列表
  - clear_logs: 清空日志
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";                 // Vue 核心 API
import { invoke } from "@tauri-apps/api/core";         // Tauri IPC 调用
import type { LogEntry } from "../types";              // 日志条目类型定义

/** 日志列表 - 从后端获取的日志条目集合 */
const logs = ref<LogEntry[]>([]);

/** 加载中状态 - 标识是否正在加载日志 */
const loading = ref(false);

/** 组件挂载时加载日志 */
onMounted(() => fetchLogs());

/** 加载日志列表 - 调用后端获取最近 200 条日志 */
async function fetchLogs() {
  loading.value = true;
  try {
    logs.value = await invoke<LogEntry[]>("get_logs", { limit: 200 });
  } catch { /* 忽略获取日志错误 */ }
  finally { loading.value = false; }
}

/** 清空日志 - 调用后端清除所有日志并清空本地列表 */
async function clearLogs() {
  await invoke("clear_logs");
  logs.value = [];
}

/** 日志级别颜色映射 - 不同级别显示不同颜色 */
const levelColors: Record<string, string> = {
  ERROR: "var(--error)",          // 错误 - 红色
  WARN: "var(--warning)",         // 警告 - 橙色
  INFO: "var(--accent)",          // 信息 - 紫色
  DEBUG: "var(--text-secondary)", // 调试 - 灰色
};
</script>

<template>
  <div>
    <!-- 操作按钮区域 -->
    <div style="margin-bottom: 1rem">
      <!-- 清空日志按钮 -->
      <button class="btn btn-outline" @click="clearLogs">清空日志</button>
    </div>

    <!-- 日志列表卡片 -->
    <div class="card">
      <!-- 加载中提示 -->
      <div v-if="loading" style="color: var(--text-secondary)">加载中...</div>
      <!-- 无数据提示 -->
      <div v-else-if="logs.length === 0" style="color: var(--text-secondary)">暂无日志</div>
      <!-- 日志条目列表 -->
      <div v-else class="log-list">
        <div v-for="(log, idx) in logs" :key="log.id ?? idx" class="log-entry">
          <!-- 日志级别 - 按级别着色 -->
          <span class="log-level" :style="{ color: levelColors[log.level] || 'inherit' }">
            [{{ log.level }}]
          </span>
          <!-- 日志时间 -->
          <span class="log-time">{{ new Date(log.created_at).toLocaleString() }}</span>
          <!-- 日志模块 -->
          <span class="log-module" v-if="log.module">[{{ log.module }}]</span>
          <!-- 日志消息内容 -->
          <span class="log-msg">{{ log.message }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 日志列表容器 - 固定高度可滚动，等宽字体 */
.log-list {
  max-height: 60vh;
  overflow-y: auto;
  font-family: "Cascadia Code", "Fira Code", monospace;
  font-size: 0.8125rem;
}

/* 单条日志条目 - 水平排列各字段 */
.log-entry {
  padding: 0.375rem 0;
  border-bottom: 1px solid rgba(53, 56, 87, 0.5);
  display: flex;
  gap: 0.5rem;
}

/* 日志级别 - 加粗固定宽度 */
.log-level {
  font-weight: 600;
  min-width: 3.5rem;
}

/* 日志时间 - 灰色固定宽度 */
.log-time {
  color: var(--text-secondary);
  min-width: 10rem;
}

/* 日志模块 - 灰色 */
.log-module {
  color: var(--text-secondary);
}

/* 日志消息 - 占据剩余空间 */
.log-msg {
  flex: 1;
}
</style>
