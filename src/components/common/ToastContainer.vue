<!--
  ToastContainer.vue - 全局 toast 堆叠容器

  功能：
  - 固定在右下角，渲染全局 toast 队列（来自 useToast）
  - 多条消息按时间顺序从底向上堆叠，互不覆盖
  - 每条 toast 复用 StandardizedMessage 样式，支持手动关闭与自动消失
  - 覆盖在底部工具栏之上，不阻挡页面其余交互

  使用：在 App.vue 根节点挂载一次即可，主窗口与弹出窗口共用。
-->
<script setup lang="ts">
import StandardizedMessage from "../base/StandardizedMessage.vue";
import { useToast } from "../../composables/useToast";

const { toasts, removeToast } = useToast();
</script>

<template>
  <div class="toast-container" aria-live="polite">
    <TransitionGroup name="toast">
      <div
        v-for="item in toasts"
        :key="item.id"
        class="toast-item"
      >
        <StandardizedMessage
          :type="item.level"
          :message="item.text"
          :duration="item.duration"
          closable
          @close="removeToast(item.id)"
        />
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  right: 1rem;
  bottom: 3.25rem;
  z-index: 2000;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.5rem;
  width: min(360px, calc(100vw - 2rem));
  pointer-events: none;
}

.toast-item {
  width: 100%;
  pointer-events: auto;
}

/* 堆叠进出场动画（从底部向上滑入/滑出） */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(12px);
}

.toast-leave-active {
  position: absolute;
  right: 0;
  width: 100%;
}
</style>
