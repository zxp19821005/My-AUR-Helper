<!--
  BottomToolbar.vue - 底部工具栏组件

  功能：
  - 左侧：信息文本（向后兼容 infoText）
  - 中间：分页控件（首页/上一页/页码输入/下一页/末页/总条数）
  - 右侧：进度条（用于长时间运行的操作）
  - 日志面板：点击展开/收起，显示历史操作日志
    - 支持滚动查看、自动滚到底部
    - 按级别高亮显示（info/success/warning/error）
    - 支持清空日志
-->
<script setup lang="ts">
import { ref, computed, inject, watch, nextTick } from "vue";
import { FOOTER_KEY, clearMessages } from "../composables/footer";
import { Home, ChevronLeft, ChevronRight, SkipForward, ChevronUp, ChevronDown, Trash2 } from "@lucide/vue";

const footer = inject(FOOTER_KEY)!;

const totalPages = computed(() => Math.ceil(footer.totalRecords / footer.pageSize) || 1);

const jumpInput = ref(String(footer.currentPage));
let jumpTimer: ReturnType<typeof setTimeout> | null = null;

const logContainer = ref<HTMLDivElement | null>(null);

function goTo(page: number) {
  if (page < 1 || page > totalPages.value) return;
  footer.currentPage = page;
  if (footer.onPageChange) footer.onPageChange(page);
}

function onJumpInput() {
  if (jumpTimer) clearTimeout(jumpTimer);
  jumpTimer = setTimeout(() => {
    const p = parseInt(jumpInput.value, 10);
    if (!isNaN(p)) goTo(p);
  }, 500);
}

function toggleLogPanel() {
  footer.logPanelExpanded = !footer.logPanelExpanded;
  if (footer.logPanelExpanded) {
    nextTick(() => scrollToBottom());
  }
}

function scrollToBottom() {
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight;
  }
}

function handleClearMessages() {
  clearMessages(footer);
}

function formatTime(ts: number): string {
  const d = new Date(ts);
  const h = String(d.getHours()).padStart(2, "0");
  const m = String(d.getMinutes()).padStart(2, "0");
  const s = String(d.getSeconds()).padStart(2, "0");
  return `${h}:${m}:${s}`;
}

watch(() => footer.currentPage, (p) => {
  jumpInput.value = String(p);
});

watch(
  () => footer.messages.length,
  () => {
    if (footer.logPanelExpanded) {
      nextTick(() => scrollToBottom());
    }
  }
);
</script>

<template>
  <div class="bottom-toolbar-wrapper">
    <!-- 日志面板（展开时显示） -->
    <div v-if="footer.logPanelExpanded" class="log-panel">
      <div class="log-panel-header">
        <span class="log-panel-title">操作日志</span>
        <div class="log-panel-actions">
          <button class="btn-icon btn-icon-info" @click="handleClearMessages" title="清空日志">
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
            {{ msg.level === "info" ? "信息" : msg.level === "success" ? "成功" : msg.level === "warning" ? "警告" : "错误" }}
          </span>
          <span class="log-text">{{ msg.text }}</span>
        </div>
      </div>
    </div>

    <!-- 底部工具栏主体 -->
    <div class="bottom-toolbar">
      <div class="btf-left">
        <span v-if="footer.infoText" class="info-text">{{ footer.infoText }}</span>
      </div>
      <div class="btf-center">
        <template v-if="footer.showPagination">
          <button class="btn-icon btn-icon-info" :disabled="footer.currentPage <= 1" @click="goTo(1)" title="首页">
            <Home :size="16" />
          </button>
          <button class="btn-icon btn-icon-info" :disabled="footer.currentPage <= 1" @click="goTo(footer.currentPage - 1)" title="上一页">
            <ChevronLeft :size="16" />
          </button>

          <span class="btf-page-info">
            <input v-model="jumpInput" class="btf-input" @input="onJumpInput" />
            <span class="btf-text">/ {{ totalPages }} 页</span>
          </span>

          <button class="btn-icon btn-icon-info" :disabled="footer.currentPage >= totalPages" @click="goTo(footer.currentPage + 1)" title="下一页">
            <ChevronRight :size="16" />
          </button>
          <button class="btn-icon btn-icon-info" :disabled="footer.currentPage >= totalPages" @click="goTo(totalPages)" title="末页">
            <SkipForward :size="16" />
          </button>

          <span class="btf-text">共 {{ footer.totalRecords }} 条</span>
        </template>
      </div>
      <div class="btf-right">
        <button class="log-toggle-btn" @click="toggleLogPanel" :title="footer.logPanelExpanded ? '收起日志' : '展开日志'">
          <ChevronUp v-if="footer.logPanelExpanded" :size="14" />
          <ChevronDown v-else :size="14" />
          <span v-if="footer.messages.length > 0" class="log-count-badge">{{ footer.messages.length }}</span>
        </button>
        <div v-if="footer.progress" class="btf-progress">
          <span v-if="footer.progress.message" class="btf-progress-msg">{{ footer.progress.message }}</span>
          <div class="btf-progress-track">
            <div class="btf-progress-fill" :style="{ width: (footer.progress.current / footer.progress.total * 100) + '%' }"></div>
          </div>
          <span class="btf-text">{{ footer.progress.current }} / {{ footer.progress.total }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bottom-toolbar-wrapper {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  border-top: 1px solid var(--border);
  background-color: var(--bg-secondary);
}

/* 日志面板 */
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

/* 底部工具栏主体 */
.bottom-toolbar {
  display: flex;
  align-items: center;
  align-self: stretch;
  justify-content: space-between;
  padding: 0.375rem 1.25rem;
  min-height: 36px;
  font-size: 0.8125rem;
  gap: 1rem;
}

.btf-left {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  text-align: left;
  color: var(--text-secondary);
}

.info-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-toggle-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
  width: 24px;
  height: 24px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
  flex-shrink: 0;
}

.log-toggle-btn:hover {
  background-color: var(--bg-hover);
  color: var(--accent);
}

.log-count-badge {
  position: absolute;
  top: -4px;
  right: -6px;
  min-width: 14px;
  height: 14px;
  padding: 0 3px;
  border-radius: 7px;
  background-color: var(--accent);
  color: white;
  font-size: 0.5625rem;
  font-weight: 700;
  line-height: 14px;
  text-align: center;
}

.btf-center {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.btf-right {
  flex: 1;
  display: flex;
  justify-content: flex-end;
}

.btf-page-info {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.btf-input {
  width: 48px;
  padding: 0.125rem 0.25rem;
  border-radius: 4px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.8125rem;
  text-align: center;
}

.btf-text {
  color: var(--text-secondary);
  white-space: nowrap;
}

.btf-progress {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.btf-progress-msg {
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
  font-size: 0.75rem;
}

.btf-progress-track {
  width: 120px;
  height: 6px;
  border-radius: 3px;
  background-color: var(--border);
  overflow: hidden;
}

.btf-progress-fill {
  height: 100%;
  border-radius: 3px;
  background-color: var(--accent);
  transition: width 0.2s;
}
</style>
