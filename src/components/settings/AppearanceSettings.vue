<!--
  AppearanceSettings.vue - 外观设置组件

  功能：
  - 主题选择（深色/浅色）
  - 字体大小选择
-->
<script setup lang="ts">
import StandardizedCard from "../components/base/StandardizedCard.vue";
import StandardizedSelect from "../components/base/StandardizedSelect.vue";

defineProps<{
  theme: string;
  fontSize: string;
}>();

const emit = defineEmits<{
  "update:theme": [value: string];
  "update:fontSize": [value: string];
}>();

function applySettings(theme: string, fontSize: string) {
  document.documentElement.setAttribute("data-theme", theme);
  document.documentElement.style.fontSize = fontSize + "px";
}

function handleThemeChange(value: string | number) {
  const theme = String(value);
  localStorage.setItem("app-theme", theme);
  emit("update:theme", theme);
  applySettings(theme, props.fontSize);
}

function handleFontSizeChange(value: string | number) {
  const fontSize = String(value);
  localStorage.setItem("app-font-size", fontSize);
  emit("update:fontSize", fontSize);
  applySettings(props.theme, fontSize);
}

const props = defineProps<{
  theme: string;
  fontSize: string;
}>();
</script>

<template>
  <StandardizedCard
    title="外观设置"
    subtitle="选择应用主题和字体大小"
  >
    <div class="setting-row">
      <div class="setting-info">
        <h4>主题</h4>
        <p>选择应用主题</p>
      </div>
      <StandardizedSelect
        :modelValue="theme"
        size="md"
        @update:modelValue="handleThemeChange"
      >
        <option value="dark">深色</option>
        <option value="light">浅色</option>
      </StandardizedSelect>
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <h4>字体大小</h4>
        <p>调整界面文字大小</p>
      </div>
      <StandardizedSelect
        :modelValue="fontSize"
        size="md"
        @update:modelValue="handleFontSizeChange"
      >
        <option value="12">小 (12px)</option>
        <option value="14">默认 (14px)</option>
        <option value="16">大 (16px)</option>
        <option value="18">特大 (18px)</option>
      </StandardizedSelect>
    </div>
  </StandardizedCard>
</template>

<style scoped>
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--border);
  gap: 1rem;
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-info h4 {
  margin: 0 0 0.25rem 0;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
}

.setting-info p {
  margin: 0;
  font-size: 0.75rem;
  color: var(--text-secondary);
}
</style>