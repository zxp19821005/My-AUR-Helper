<!--
  QuickFilterDropdown.vue - 工具栏单选快速筛选下拉

  仅展示「单选快速筛选」菜单（AUR更新失败/上游更新失败/...），与多重快速筛选
  （FilterBar 复选框）为 OR 关系。选中态由父组件通过 activeKey / activeLabel 传入，
  点击选项 emit "select"、清除按钮 emit "clear"，开合状态自管。
-->
<script setup lang="ts">
import { ref } from "vue";
import { Icon } from "@/icons";
import type { QuickFilterKey } from "@/composables/filterConfig";

const props = defineProps<{
  options: { key: QuickFilterKey; label: string }[];
  activeKey: QuickFilterKey | null;
  activeLabel: string;
}>();

const emit = defineEmits<{
  (e: "select", key: QuickFilterKey): void;
  (e: "clear"): void;
}>();

const open = ref(false);

function toggle() {
  open.value = !open.value;
}
function close() {
  open.value = false;
}
function selectAndClose(key: QuickFilterKey) {
  emit("select", key);
  open.value = false;
}
</script>

<template>
  <div class="toolbar-quickfilter">
    <button
      class="btn-icon qf-trigger"
      :class="{ 'qf-active': !!activeLabel }"
      @click="toggle"
      :title="activeLabel ? `快速筛选：${activeLabel}` : '快速筛选'"
    >
      <component :is="Icon.actionFilter" :size="16" />
      <component :is="Icon.actionDropdown" :size="12" class="qf-caret" />
    </button>
    <button
      v-if="activeLabel"
      class="btn-icon qf-clear"
      @click="emit('clear')"
      :title="`清除快速筛选：${activeLabel}`"
    >
      <component :is="Icon.actionClear" :size="14" />
    </button>
    <div v-if="open" class="qf-menu">
      <button
        v-for="opt in options"
        :key="opt.key"
        class="qf-menu-item"
        :class="{ active: activeKey === opt.key }"
        @click="selectAndClose(opt.key)"
      >
        {{ opt.label }}
      </button>
    </div>
  </div>
  <div v-if="open" class="qf-backdrop" @click="close"></div>
</template>

<style scoped>
.toolbar-quickfilter {
  position: relative;
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

/* 触发器与清除按钮均自包含基础按钮样式（不依赖父组件 .btn-icon，避免 scoped 跨组件失效） */
.qf-trigger,
.qf-clear {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.375rem;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.qf-trigger:hover:not(:disabled),
.qf-clear:hover:not(:disabled) {
  background-color: var(--bg-card);
  color: var(--text-primary);
}

/* 漏斗与箭头属同一视觉整体，负外边距抵消 .qf-trigger 的 gap，使两者贴合为一个组合图标 */
.qf-caret {
  margin-left: -0.125rem;
}

/* 激活态：图标整体转橙色（--warning），与「已启用快速筛选」语义一致 */
.qf-trigger.qf-active,
.qf-trigger.qf-active:hover:not(:disabled) {
  color: var(--warning);
  background-color: var(--warning-bg);
}

/* 点击下拉外部关闭菜单的透明遮罩 */
.qf-backdrop {
  position: fixed;
  inset: 0;
  z-index: 50;
}

.qf-menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 168px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
  padding: 0.25rem;
  z-index: 60;
  display: flex;
  flex-direction: column;
}

.qf-menu-item {
  text-align: left;
  padding: 0.4rem 0.6rem;
  border: none;
  background: none;
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 0.8125rem;
  cursor: pointer;
  white-space: nowrap;
}

.qf-menu-item:hover {
  background: var(--bg-card);
}

/* 选中项与触发器统一使用橙色（--warning），保持视觉语义一致 */
.qf-menu-item.active {
  background: var(--warning-bg);
  color: var(--warning);
  font-weight: 600;
}
</style>
