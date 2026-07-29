<!--
  BackupInfoDialog.vue - 包信息查看弹窗

  功能：
  - 显示软件包的详细信息（通过 pacinfo 命令获取）
  - 加载状态提示
-->
<script setup lang="ts">
import StandardizedModal from "../components/common/StandardizedModal.vue";

defineProps<{
  show: boolean;
  loading: boolean;
  pkgname: string;
  content: string;
}>();

const emit = defineEmits<{
  close: [];
}>();
</script>

<template>
  <StandardizedModal
    :show="show"
    title="包信息"
    @close="emit('close')"
    width="700px"
  >
    <template #header>
      <h3>{{ pkgname }} - 包信息</h3>
    </template>
    <div v-if="loading" class="loading-spinner">加载中...</div>
    <pre v-else class="info-content">{{ content }}</pre>
  </StandardizedModal>
</template>

<style scoped>
.loading-spinner {
  text-align: center;
  color: var(--text-secondary);
  padding: 2rem;
}

.info-content {
  font-family: monospace;
  font-size: 0.8125rem;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-primary);
  background: var(--bg-secondary);
  padding: 1rem;
  border-radius: 8px;
  margin: 0;
  max-height: 400px;
  overflow-y: auto;
}
</style>