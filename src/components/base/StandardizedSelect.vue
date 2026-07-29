<!--
  StandardizedSelect.vue - 通用下拉选择框组件

  功能：
  - 统一选择框样式
  - 支持多种尺寸
  - 支持前置图标
  - 支持禁用状态
  - 支持选项配置

  Props:
  - modelValue: string | number - 选中值（v-model绑定）
  - options: Array<{value: string | number, label: string}> - 选项列表
  - placeholder?: string - 占位符
  - size?: 'sm' | 'md' | 'lg' - 选择框尺寸
  - disabled?: boolean - 是否禁用
  - prefix?: Component - 前置图标

  Events:
  - update:modelValue - 值变化
  - change - 变化事件

  使用示例：
  <StandardizedSelect
    v-model="theme"
    :options="[
      { value: 'dark', label: '深色' },
      { value: 'light', label: '浅色' },
    ]"
    :prefix="Palette"
  />
-->
<script setup lang="ts">
import type { Component } from "vue";

const props = withDefaults(defineProps<{
  /** 选中值 */
  modelValue?: string | number | null;
  /** 选项列表 */
  options?: Array<{ value: string | number; label: string }>;
  /** 占位符 */
  placeholder?: string;
  /** 选择框尺寸 */
  size?: "sm" | "md" | "lg";
  /** 是否禁用 */
  disabled?: boolean;
  /** 前置图标 */
  prefix?: Component;
}>(), {
  modelValue: null,
  options: () => [],
  placeholder: "请选择",
  size: "md",
  disabled: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: string | number];
  change: [value: string | number];
}>();

function handleChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  emit("update:modelValue", value);
  emit("change", value);
}
</script>

<template>
  <div
    class="select-wrapper"
    :class="[
      `select-wrapper-${size}`,
      { 'select-wrapper-disabled': disabled, 'select-wrapper-has-prefix': prefix },
    ]"
  >
    <div v-if="prefix" class="select-prefix">
      <component :is="prefix" :size="size === 'sm' ? 14 : size === 'lg' ? 18 : 16" />
    </div>

    <select
      :value="modelValue"
      :disabled="disabled"
      class="select-field"
      @change="handleChange"
    >
      <option value="" disabled>{{ placeholder }}</option>
      <option v-for="opt in options" :key="opt.value" :value="opt.value">
        {{ opt.label }}
      </option>
    </select>
  </div>
</template>

<style scoped>
.select-wrapper {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 2rem 0.5rem 0.75rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background-color: var(--bg-primary);
  transition: all 0.2s;
  position: relative;
  width: 100%;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%239a9cb8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 0.75rem center;
}

.select-wrapper:hover:not(.select-wrapper-disabled) {
  border-color: var(--text-muted);
}

.select-wrapper:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgba(108, 99, 255, 0.15);
}

.select-wrapper-disabled {
  opacity: 0.6;
  cursor: not-allowed;
  background-color: var(--bg-secondary);
}

.select-wrapper-sm {
  padding: 0.375rem 1.75rem 0.375rem 0.625rem;
  border-radius: 6px;
  font-size: 0.8125rem;
  background-position: right 0.5rem center;
}

.select-wrapper-md {
  padding: 0.5rem 2rem 0.5rem 0.75rem;
  border-radius: 8px;
  font-size: 0.875rem;
}

.select-wrapper-lg {
  padding: 0.625rem 2.25rem 0.625rem 1rem;
  border-radius: 10px;
  font-size: 1rem;
  background-position: right 1rem center;
}

.select-wrapper-has-prefix {
  padding-left: 0.5rem;
}

.select-field {
  flex: 1;
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: inherit;
  outline: none;
  appearance: none;
  -webkit-appearance: none;
  cursor: pointer;
}

.select-field:disabled {
  cursor: not-allowed;
}

.select-field option {
  background-color: var(--bg-primary);
  color: var(--text-primary);
}

.select-prefix {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  flex-shrink: 0;
}
</style>