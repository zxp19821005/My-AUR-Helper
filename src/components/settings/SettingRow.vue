<!--
  SettingRow.vue - 通用设置行组件

  功能：
  - 提供统一的设置行布局
  - 支持标签、描述、输入内容

  Props:
  - label: string - 设置项标签
  - description: string - 设置项描述（可选）
-->
<script setup lang="ts">
defineProps<{
  /** 设置项标签 */
  label: string;
  /** 设置项描述 */
  description?: string;
}>();
</script>

<template>
  <div class="setting-row">
    <div class="setting-label">
      <strong>{{ label }}</strong>
      <span v-if="description" class="setting-desc">{{ description }}</span>
    </div>
    <div class="setting-input">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.setting-row {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--border);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-label {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  flex: 1;
  min-width: 200px;
}

.setting-label strong {
  font-size: 0.875rem;
  color: var(--text-primary);
}

.setting-desc {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.setting-input {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 2;
  min-width: 0;
}

.setting-input :deep(.input-wrapper) {
  min-width: 0;
  flex: 1;
}

.setting-input :deep(.input-field) {
  min-width: 0;
  flex: 1;
}

/* 响应式设计 - 平板及以下 */
@media (max-width: 768px) {
  .setting-row {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
  }

  .setting-input {
    width: 100%;
  }
}
</style>