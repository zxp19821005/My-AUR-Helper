<!--
  LogsPopup.vue - 日志查看弹出窗口组件

  功能：
  - 提供日志查看窗口的布局（无左侧导航栏）
  - 直接显示日志查看内容
  - 包含底部工具栏（分页、进度条等）
-->
<script setup lang="ts">
import BottomToolbar from "./BottomToolbar.vue";
</script>

<template>
  <div class="logs-popup-layout">
    <!-- 内容区域 -->
    <main class="logs-popup-content">
      <RouterView />
    </main>

    <!-- 底部工具栏 - 分页、进度条等 -->
    <BottomToolbar />
  </div>
</template>

<style scoped>
.logs-popup-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100%;
  background-color: var(--bg-primary);
}

.logs-popup-content {
  flex: 1;
  overflow-y: auto;
  min-width: 0;
}
</style>