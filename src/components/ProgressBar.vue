<!--
  ProgressBar.vue - 进度条组件

  功能：
  - 显示当前操作进度
  - 显示进度消息和百分比
-->
<script setup lang="ts">
import type { FooterState } from "../composables/footer";

defineProps<{
  footer: FooterState;
}>();
</script>

<template>
  <div v-if="footer.progress" class="btf-progress">
    <span v-if="footer.progress.message" class="btf-progress-msg">{{ footer.progress.message }}</span>
    <div class="btf-progress-track">
      <div class="btf-progress-fill" :style="{ width: (footer.progress.current / footer.progress.total * 100) + '%' }"></div>
    </div>
    <span class="btf-text">{{ footer.progress.current }} / {{ footer.progress.total }}</span>
  </div>
</template>

<style scoped>
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

.btf-text {
  color: var(--text-secondary);
  white-space: nowrap;
}
</style>