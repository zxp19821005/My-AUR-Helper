<!--
  SettingsActionBar.vue - 设置页统一操作栏

  功能：
  - 固定在设置页右下角，提供「保存设置」与「重置设置」两个图标按钮
  - 仅在存在未保存修改（dirty）且非保存中时可用
  - 点击后通过事件冒泡给各设置页自行实现持久化

  Props:
  - dirty: boolean - 是否存在未保存修改
  - saving: boolean - 是否正在保存

  Events:
  - save - 点击保存设置
  - reset - 点击重置设置
-->
<script setup lang="ts">
import { Icon } from "../../icons";

defineProps<{
  /** 是否存在未保存修改 */
  dirty: boolean;
  /** 是否正在保存 */
  saving: boolean;
}>();

const emit = defineEmits<{
  save: [];
  reset: [];
}>();
</script>

<template>
  <div class="settings-action-bar">
    <button
      class="action-btn action-reset"
      type="button"
      :disabled="!dirty || saving"
      title="重置设置（撤销未保存的修改）"
      @click="emit('reset')"
    >
      <component :is="Icon.reset" :size="18" />
      <span>重置设置</span>
    </button>
    <button
      class="action-btn action-save"
      type="button"
      :disabled="!dirty || saving"
      :title="saving ? '保存中...' : '保存设置'"
      @click="emit('save')"
    >
      <component :is="Icon.save" :size="18" />
      <span>{{ saving ? "保存中..." : "保存设置" }}</span>
    </button>
  </div>
</template>

<style scoped>
.settings-action-bar {
  position: fixed;
  right: 1.5rem;
  bottom: 4rem;
  z-index: 100;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem;
  background-color: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.5rem 0.875rem;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: none;
  color: var(--text-primary);
  font-size: 0.8125rem;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.action-btn:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--accent);
}

.action-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.action-save {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}

.action-save:hover:not(:disabled) {
  opacity: 0.9;
  color: #fff;
}

/* 小屏幕：仅显示图标，隐藏文字以节省空间 */
@media (max-width: 480px) {
  .settings-action-bar {
    right: 0.75rem;
    bottom: 3.5rem;
  }

  .action-btn span {
    display: none;
  }

  .action-btn {
    padding: 0.5rem;
  }
}
</style>
