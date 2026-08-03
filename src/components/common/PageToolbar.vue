<script setup lang="ts">
import { ref, watch } from "vue";
import { Settings, List, ScrollText, Search, RefreshCw } from "@lucide/vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

const props = defineProps<{
  modelValue?: string;
  filterActive?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "refresh"): void;
  (e: "toggle-filter"): void;
}>();

const searchText = ref(props.modelValue || "");
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(searchText, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    emit("update:modelValue", val);
  }, 500);
});

async function openWindow(label: string, url: string, title: string) {
  const existing = await WebviewWindow.getByLabel(label);
  if (existing) {
    try {
      // 检查窗口是否仍然有效且可见
      if (await existing.isVisible()) {
        await existing.setFocus();
        return;
      }
      // 窗口存在但不可见（最小化），恢复它
      await existing.show();
      await existing.setFocus();
      return;
    } catch {
      // 窗口已关闭/销毁，忽略错误，继续创建新窗口
    }
  }
  // 创建新窗口
  new WebviewWindow(label, {
    url,
    title,
    width: 900,
    height: 600,
    resizable: true,
    center: true,
  });
}

async function openEnums() {
  await openWindow("enums", "/enums", "枚举值管理");
}

async function openLogs() {
  await openWindow("logs", "/logs", "日志");
}

async function openSettings() {
  await openWindow("settings", "/settings", "设置");
}
</script>

<template>
  <div class="page-toolbar">
    <div class="toolbar-left">
      <slot />
    </div>
    <div class="toolbar-right">
      <slot name="right" />
      <button
        class="toolbar-icon-btn"
        :class="{ 'btn-icon-warning': filterActive }"
        @click="emit('toggle-filter')"
        title="筛选"
      >
        <slot name="filter-icon" />
      </button>
      <div class="toolbar-divider"></div>
      <div class="search-box">
        <Search :size="14" class="search-icon" />
        <input
          v-model="searchText"
          type="text"
          class="search-input"
          placeholder="搜索..."
        />
      </div>
      <button class="toolbar-icon-btn" @click="openEnums" title="枚举值管理">
        <List :size="18" />
      </button>
      <button class="toolbar-icon-btn" @click="openLogs" title="日志">
        <ScrollText :size="18" />
      </button>
      <div class="toolbar-divider"></div>
      <button class="toolbar-icon-btn" @click="openSettings" title="设置">
        <Settings :size="18" />
      </button>
      <button class="toolbar-icon-btn" @click="emit('refresh')" title="刷新数据">
        <RefreshCw :size="18" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.page-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 1.25rem;
  border-bottom: 1px solid var(--border);
  background-color: var(--bg-primary);
  min-height: 44px;
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}
.toolbar-divider {
  width: 1px;
  height: 20px;
  background-color: var(--border);
  margin: 0 0.375rem;
}
.toolbar-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.375rem;
  border-radius: 6px;
  transition: all 0.15s;
}
.toolbar-icon-btn:hover {
  color: var(--text-primary);
  background-color: var(--bg-card);
}
.btn-icon-warning {
  color: var(--color-warning) !important;
}
.search-box {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.25rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--bg-card);
  transition: border-color 0.15s;
}
.search-box:focus-within {
  border-color: var(--accent);
}
.search-icon {
  color: var(--text-secondary);
  flex-shrink: 0;
}
.search-input {
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: 0.8125rem;
  outline: none;
  width: 140px;
}
.search-input::placeholder {
  color: var(--text-muted);
}
</style>