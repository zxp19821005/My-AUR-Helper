<!--
  StandardizedButton.vue - 通用按钮组件

  功能：
  - 统一按钮样式和交互
  - 支持多种变体（primary/secondary/outline/danger/ghost）
  - 支持多种尺寸（sm/md/lg）
  - 支持加载状态
  - 支持禁用状态
  - 支持图标插槽

  Props:
  - variant?: 'primary' | 'secondary' | 'outline' | 'danger' | 'ghost' - 按钮变体
  - size?: 'sm' | 'md' | 'lg' - 按钮尺寸
  - disabled?: boolean - 是否禁用
  - loading?: boolean - 是否加载中
  - type?: 'button' | 'submit' | 'reset' - 按钮类型

  Slots:
  - default - 按钮内容
  - icon - 图标（前置）

  使用示例：
  <StandardizedButton variant="primary" size="md" @click="handleClick">
    点击我
  </StandardizedButton>

  <StandardizedButton variant="outline" :loading="saving">
    保存
  </StandardizedButton>
-->
<script setup lang="ts">
import { Loader2 } from "@lucide/vue";

withDefaults(defineProps<{
  /** 按钮变体 */
  variant?: "primary" | "secondary" | "outline" | "danger" | "ghost";
  /** 按钮尺寸 */
  size?: "sm" | "md" | "lg";
  /** 是否禁用 */
  disabled?: boolean;
  /** 是否加载中 */
  loading?: boolean;
  /** 按钮类型 */
  type?: "button" | "submit" | "reset";
}>(), {
  variant: "primary",
  size: "md",
  disabled: false,
  loading: false,
  type: "button",
});
</script>

<template>
  <button
    class="standardized-button"
    :class="[
      `variant-${variant}`,
      `size-${size}`,
      { loading },
    ]"
    :disabled="disabled || loading"
    :type="type"
  >
    <Loader2 v-if="loading" :size="size === 'sm' ? 14 : size === 'lg' ? 18 : 16" class="icon" />
    <slot name="icon" />
    <slot />
  </button>
</template>

<style scoped>
.standardized-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  border-radius: 6px;
  font-weight: 500;
  transition: all 0.2s;
  outline: none;
  cursor: pointer;
  white-space: nowrap;
  user-select: none;
  border: 1px solid transparent;
}

/* Variants */
.variant-primary {
  background-color: var(--primary);
  color: white;
  border-color: var(--primary);
}

.variant-primary:hover:not(:disabled):not(.loading) {
  opacity: 0.9;
}

.variant-primary:active:not(:disabled):not(.loading) {
  transform: translateY(1px);
}

.variant-secondary {
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  border-color: var(--border);
}

.variant-secondary:hover:not(:disabled):not(.loading) {
  background-color: var(--bg-card);
}

.variant-outline {
  background-color: transparent;
  color: var(--primary);
  border-color: var(--primary);
}

.variant-outline:hover:not(:disabled):not(.loading) {
  background-color: rgba(108, 99, 255, 0.05);
}

.variant-danger {
  background-color: var(--error);
  color: white;
  border-color: var(--error);
}

.variant-danger:hover:not(:disabled):not(.loading) {
  opacity: 0.9;
}

.variant-ghost {
  background-color: transparent;
  color: var(--text-secondary);
  border-color: transparent;
}

.variant-ghost:hover:not(:disabled):not(.loading) {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

/* Sizes */
.size-sm {
  padding: 0.375rem 0.75rem;
  font-size: 0.8125rem;
  min-height: 2rem;
}

.size-sm .icon {
  width: 14px;
  height: 14px;
}

.size-md {
  padding: 0.5rem 1rem;
  font-size: 0.875rem;
  min-height: 2.5rem;
}

.size-md .icon {
  width: 16px;
  height: 16px;
}

.size-lg {
  padding: 0.625rem 1.25rem;
  font-size: 0.875rem;
  min-height: 3rem;
}

.size-lg .icon {
  width: 18px;
  height: 18px;
}

/* Disabled state */
.standardized-button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  pointer-events: none;
}

/* Icon spacing */
.standardized-button :deep(.icon) + :deep(span),
.standardized-button :deep(span) + :deep(.icon) {
  margin-left: 0.375rem;
}

/* Loading state */
.loading {
  position: relative;
  pointer-events: none;
  opacity: 0.7;
}

.loading .icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>