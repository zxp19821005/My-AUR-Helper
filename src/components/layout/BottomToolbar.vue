<!--
  BottomToolbar.vue - 底部工具栏组件（三段式）

  布局分为三部分：
  - 左侧（flex:1）：留白/次要状态占位
  - 中间：分页控件（仅 showPagination 时显示）
  - 右侧（约 1/3 宽）：信息显示区域，承载不重要的状态信息（infoText）与进度条

  说明：
  - 原右下角可折叠日志面板已移除，操作反馈统一走右下角 toast（ToastContainer）
  - 持久日志改由「日志」页面查看
-->
<script setup lang="ts">
import { inject } from "vue";
import { FOOTER_KEY } from "../../composables/footer";
import PaginationControls from "../common/PaginationControls.vue";
import ProgressBar from "../common/ProgressBar.vue";

const footer = inject(FOOTER_KEY)!;
</script>

<template>
  <div class="bottom-toolbar-wrapper">
    <div class="bottom-toolbar">
      <!-- 左侧：占位 -->
      <div class="btf-left"></div>

      <!-- 中间：分页 -->
      <div class="btf-center">
        <PaginationControls v-if="footer.showPagination" :footer="footer" />
      </div>

      <!-- 右侧：信息显示区域（约 1/3） -->
      <div class="btf-right">
        <span v-if="footer.infoText" class="info-text">{{ footer.infoText }}</span>
        <ProgressBar :footer="footer" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.bottom-toolbar-wrapper {
  border-top: 1px solid var(--border);
  background-color: var(--bg-secondary);
}

.bottom-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.375rem 1.25rem;
  min-height: 36px;
  font-size: 0.8125rem;
  gap: 1rem;
}

.btf-left {
  flex: 1;
  min-width: 0;
}

.btf-center {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

/* 右侧约 1/3 宽度，作为信息显示区域 */
.btf-right {
  flex: 0 0 33%;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  min-width: 0;
}

.info-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
}
</style>
