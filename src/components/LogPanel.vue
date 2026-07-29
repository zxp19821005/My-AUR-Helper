<!--
  LogPanel.vue - 日志面板组件

  功能：
  - 显示操作日志列表
  - 支持按级别高亮显示（info/success/warning/error）
  - 支持清空日志
  - 自动滚动到底部
-->
<script setup lang="ts">
import { ref, nextTick, watch } from "vue";
import { Trash2 } from "@lucide/vue";
import type { FooterState } from "../composables/footer";

const props = defineProps<{
  footer: FooterState;
}>();

const logContainer = ref<HTMLDivElement | null>(null);

function scrollToBottom() {
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight;
  }
}

function formatTime(ts: number): string {
  const d = new Date(ts);
  const h = String(d.getHours()).padStart(2, "0");
  const m = String(d.getMinutes()).padStart(2, "0");
  const s = String(d.getSeconds()).padStart(2, "0");
  return `${h}:${m}:${s}`;
}

function getLevelText(level: string): string {
  return level === "info" ? "信息" : level === "success" ? "成功" : level === "warning" ? "警告" : "错误";
}

watch(
  () => props.footer.messages.length,
  () => {
    if (props.footer.logPanelExpanded) {
      nextTick(() => scrollToBottom());
    }
  }
);

const emit = defineEmits<{
  clear: [];
}>();
</script>

<template>
  <div class="log-panel">
    <div class="log-panel-header">
      <span class="log-panel-title">操作日志</span>
      <div class="log-panel-actions">
        <button class="btn-icon btn-icon-info" @click="emit('clear')" title="清空日志">
          <Trash2 :size="14" />
        </button>
      </div>
    </div>
    <div class="log-panel-body" ref="logContainer">
      <div v-if="footer.messages.length === 0" class="log-empty">暂无日志记录</div>
      <div
        v-for="msg in footer.messages"
        :key="msg.id"
        class="log-item"
        :class="`log-${msg.level}`"
      >
        <span class="log-time">{{ formatTime(msg.timestamp) }}</span>
        <span class="log-level-badge" :class="`badge-${msg.level}`">
          {{ getLevelText(msg.level) }}
        </span>
        <span class="log-text">{{ msg.text }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-panel {
  display: flex;
  flex-direction: column;
  max-height: 200px;
  width: 420px;
  max-width: 100vw;
  border-bottom: 1px solid var(--border);
  border-left: 1px solid var(--border);
  border-radius: 8px 0 0 0;
}

.log-panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.25rem 0.75rem;
  background-color: var(--bg-card);
  border-bottom: 1px solid var(--border);
}

.log-panel-title {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.log-panel-actions {
  display: flex;
  gap: 0.25rem;
}

.log-panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 0.25rem 0;
  scrollbar-width: thin;
}

.log-panel-body::-webkit-scrollbar {
  display: block;
  width: 4px;
}

.log-panel-body::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}

.log-panel-body::-webkit-scrollbar-track {
  background: transparent;
}

.log-empty {
  text-align: center;
  color: var(--text-muted);
  font-size: 0.75rem;
  padding: 0.75rem;
}

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 0.2rem 0.75rem;
  font-size: 0.75rem;
  line-height: 1.4;
}

.log-item:hover {
  background-color: var(--bg-hover);
}

.log-time {
  color: var(--text-muted);
  font-family: monospace;
  white-space: nowrap;
  flex-shrink: 0;
}

.log-level-badge {
  display: inline-block;
  padding: 0.05rem 0.375rem;
  border-radius: 3px;
  font-size: 0.625rem;
  font-weight: 600;
  white-space: nowrap;
  flex-shrink: 0;
}

.badge-info {
  background-color: rgba(66, 165, 245, 0.15);
  color: #42a5f5;
}

.badge-success {
  background-color: var(--success-bg);
  color: var(--success);
}

.badge-warning {
  background-color: var(--warning-bg);
  color: var(--warning);
}

.badge-error {
  background-color: var(--error-bg);
  color: var(--error);
}

.log-text {
  color: var(--text-primary);
  word-break: break-all;
}

.log-error .log-text {
  color: var(--error);
}

.log-warning .log-text {
  color: var(--warning);
}

.log-success .log-text {
  color: var(--success);
}
</style>