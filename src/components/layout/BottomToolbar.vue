<!--
  BottomToolbar.vue - 底部工具栏组件

  功能：
  - 左侧：信息文本（向后兼容 infoText）
  - 中间：分页控件（首页/上一页/页码输入/下一页/末页/总条数）
  - 右侧：进度条（用于长时间运行的操作）
  - 日志面板：点击展开/收起，显示历史操作日志

  使用组件：
  - LogPanel: 日志面板组件
  - PaginationControls: 分页控件组件
  - ProgressBar: 进度条组件
-->
<script setup lang="ts">
import { ref, inject, watch, nextTick } from "vue";
import { FOOTER_KEY, clearMessages } from "../../composables/footer";
import { ChevronUp, ChevronDown } from "@lucide/vue";
import LogPanel from "./LogPanel.vue";
import PaginationControls from "../common/PaginationControls.vue";
import ProgressBar from "../common/ProgressBar.vue";

const footer = inject(FOOTER_KEY)!;

const logContainer = ref<HTMLDivElement | null>(null);

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
    <LogPanel
      v-if="footer.logPanelExpanded"
      :footer="footer"
      @clear="handleClearMessages"
    />

    <!-- 底部工具栏主体 -->
    <div class="bottom-toolbar">
      <div class="btf-left">
        <span v-if="footer.infoText" class="info-text">{{ footer.infoText }}</span>
      </div>
      <div class="btf-center">
        <PaginationControls v-if="footer.showPagination" :footer="footer" />
      </div>
      <div class="btf-right">
        <button class="log-toggle-btn" @click="toggleLogPanel" :title="footer.logPanelExpanded ? '收起日志' : '展开日志'">
          <ChevronUp v-if="footer.logPanelExpanded" :size="14" />
          <ChevronDown v-else :size="14" />
          <span v-if="footer.messages.length > 0" class="log-count-badge">{{ footer.messages.length }}</span>
        </button>
        <ProgressBar :footer="footer" />
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

.btf-right {
  flex: 1;
  display: flex;
  justify-content: flex-end;
}
</style>