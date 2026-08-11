<!--
  StandardizedInput.vue - 通用输入框组件

  功能：
  - 统一输入框样式
  - 支持多种尺寸
  - 支持前置/后置图标或内容
  - 支持清空按钮
  - 支持密码显示/隐藏
  - 支持验证状态显示

  Props:
  - modelValue: string - 输入值（v-model绑定）
  - type?: 'text' | 'password' | 'number' | 'email' | 'url' - 输入类型
  - placeholder?: string - 占位符
  - size?: 'sm' | 'md' | 'lg' - 输入框尺寸
  - disabled?: boolean - 是否禁用
  - clearable?: boolean - 是否显示清空按钮
  - prefix?: Component - 前置图标
  - suffix?: Component - 后置图标
  - error?: boolean - 是否显示错误状态
  - success?: boolean - 是否显示成功状态

  Events:
  - update:modelValue - 值变化
  - input - 输入事件
  - change - 变化事件
  - clear - 清空事件

  使用示例：
  <StandardizedInput
    v-model="username"
    :prefix="User"
    placeholder="请输入用户名"
    clearable
  />
-->
<script setup lang="ts">
import type { Component } from "vue";
import { ref, computed } from "vue";
import { Icon } from "../../icons";

const props = withDefaults(defineProps<{
  /** 输入值 */
  modelValue?: string;
  /** 输入类型 */
  type?: "text" | "password" | "number" | "email" | "url";
  /** 占位符 */
  placeholder?: string;
  /** 输入框尺寸 */
  size?: "sm" | "md" | "lg";
  /** 是否禁用 */
  disabled?: boolean;
  /** 是否显示清空按钮 */
  clearable?: boolean;
  /** 前置图标 */
  prefix?: Component;
  /** 后置图标 */
  suffix?: Component;
  /** 是否显示错误状态 */
  error?: boolean;
  /** 是否显示成功状态 */
  success?: boolean;
}>(), {
  modelValue: "",
  type: "text",
  placeholder: "",
  size: "md",
  disabled: false,
  clearable: false,
  error: false,
  success: false,
});

const emit = defineEmits<{
  "update:modelValue": [value: string];
  input: [value: string];
  change: [value: string];
  clear: [];
}>();

const showPassword = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);

/** 实际输入类型（处理密码显示/隐藏） */
const actualType = computed(() => {
  if (props.type === "password" && showPassword.value) {
    return "text";
  }
  return props.type;
});

/** 是否有密码切换按钮 */
const hasPasswordToggle = computed(() => props.type === "password");

/** 处理输入 */
function handleInput(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  emit("update:modelValue", value);
  emit("input", value);
}

/** 处理变化 */
function handleChange(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  emit("change", value);
}

/** 清空输入 */
function handleClear() {
  emit("update:modelValue", "");
  emit("clear");
  inputRef.value?.focus();
}

/** 切换密码显示 */
function togglePassword() {
  showPassword.value = !showPassword.value;
}

/** 暴露方法 */
defineExpose({
  focus: () => inputRef.value?.focus(),
  blur: () => inputRef.value?.blur(),
  select: () => inputRef.value?.select(),
});
</script>

<template>
  <div
    class="input-wrapper"
    :class="[
      `input-wrapper-${size}`,
      {
        'input-wrapper-disabled': disabled,
        'input-wrapper-error': error,
        'input-wrapper-success': success,
        'input-wrapper-has-prefix': prefix,
        'input-wrapper-has-suffix': suffix || clearable || hasPasswordToggle,
      },
    ]"
  >
    <div v-if="prefix" class="input-prefix">
      <component :is="prefix" :size="size === 'sm' ? 14 : size === 'lg' ? 18 : 16" />
    </div>

    <input
      ref="inputRef"
      :type="actualType"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      class="input-field"
      @input="handleInput"
      @change="handleChange"
    />

    <button
      v-if="clearable && modelValue && !disabled"
      class="input-clear"
      type="button"
      @click="handleClear"
      title="清空"
    >
      <component :is="Icon.actionClear" :size="14" />
    </button>

    <button
      v-if="hasPasswordToggle"
      class="input-password-toggle"
      type="button"
      @click="togglePassword"
      :title="showPassword ? '隐藏密码' : '显示密码'"
    >
      <component :is="showPassword ? Icon.hide : Icon.show" :size="16" />
    </button>

    <div v-if="suffix && !clearable && !hasPasswordToggle" class="input-suffix">
      <component :is="suffix" :size="size === 'sm' ? 14 : size === 'lg' ? 18 : 16" />
    </div>
  </div>
</template>

<style scoped>
.input-wrapper {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background-color: var(--bg-primary);
  transition: all 0.2s;
  width: 100%;
}

.input-wrapper:hover:not(.input-wrapper-disabled) {
  border-color: var(--text-muted);
}

.input-wrapper:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgba(108, 99, 255, 0.15);
}

.input-wrapper-error {
  border-color: var(--error);
}

.input-wrapper-error:focus-within {
  box-shadow: 0 0 0 2px rgba(239, 83, 80, 0.15);
}

.input-wrapper-success {
  border-color: var(--success);
}

.input-wrapper-success:focus-within {
  box-shadow: 0 0 0 2px rgba(76, 175, 125, 0.15);
}

.input-wrapper-disabled {
  opacity: 0.6;
  cursor: not-allowed;
  background-color: var(--bg-secondary);
}

.input-wrapper-sm {
  padding: 0.375rem 0.625rem;
  border-radius: 6px;
  font-size: 0.8125rem;
}

.input-wrapper-md {
  padding: 0.5rem 0.75rem;
  border-radius: 8px;
  font-size: 0.875rem;
}

.input-wrapper-lg {
  padding: 0.625rem 1rem;
  border-radius: 10px;
  font-size: 1rem;
}

.input-field {
  flex: 1;
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: inherit;
  outline: none;
  min-width: 0;
}

.input-field::placeholder {
  color: var(--text-muted);
}

.input-field:disabled {
  cursor: not-allowed;
}

.input-prefix,
.input-suffix {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.input-clear,
.input-password-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0.125rem;
  border-radius: 4px;
  transition: all 0.15s;
  flex-shrink: 0;
}

.input-clear:hover,
.input-password-toggle:hover {
  color: var(--text-primary);
  background-color: var(--bg-hover);
}
</style>