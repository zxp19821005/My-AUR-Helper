<!--
  SettingsCard.vue - 通用设置卡片组件

  功能：
  - 提供统一的设置卡片容器
  - 支持标题、描述、内容插槽
  - 宽度100%占满父容器可用宽度

  Props:
  - title: string - 卡片标题
  - description: string - 卡片描述（可选）

  Slots:
  - default: 主要内容区域
  - actions: 底部操作按钮区域
-->
<script setup lang="ts">
defineProps<{
  /** 卡片标题 */
  title: string;
  /** 卡片描述 */
  description?: string;
}>();
</script>

<template>
  <div class="settings-card">
    <div class="settings-card-header">
      <h3 class="settings-card-title">{{ title }}</h3>
      <p v-if="description" class="settings-card-desc">{{ description }}</p>
    </div>
    <div class="settings-card-body">
      <slot />
    </div>
    <div v-if="$slots.actions" class="settings-card-actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<style scoped>
.settings-card {
  width: 100%;
  background-color: var(--bg-card);
  border-radius: 12px;
  border: 1px solid var(--border);
  box-sizing: border-box;
}

.settings-card-header {
  padding: 1.5rem 1.5rem 0 1.5rem;
}

.settings-card-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 0.25rem 0;
}

.settings-card-desc {
  font-size: 0.8125rem;
  color: var(--text-secondary);
  margin: 0;
  line-height: 1.5;
}

.settings-card-body {
  padding: 1rem 1.5rem 1.5rem 1.5rem;
}

.settings-card-actions {
  display: flex;
  gap: 0.5rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border);
}

/* 响应式设计 - 平板及以下 */
@media (max-width: 768px) {
  .settings-card-header {
    padding: 1rem 1rem 0 1rem;
  }

  .settings-card-body {
    padding: 0.75rem 1rem 1rem 1rem;
  }

  .settings-card-actions {
    padding: 0.75rem 1rem;
    flex-direction: column;
  }
}

/* 响应式设计 - 小屏幕手机 */
@media (max-width: 480px) {
  .settings-card-header {
    padding: 0.75rem 0.75rem 0 0.75rem;
  }

  .settings-card-body {
    padding: 0.5rem 0.75rem 0.75rem 0.75rem;
  }

  .settings-card-actions {
    padding: 0.5rem 0.75rem;
  }
}
</style>