<script setup lang="ts">
import { inject, ref, watch } from "vue";
import { Settings, List, ScrollText, Search, RefreshCw, X } from "@lucide/vue";
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

const isPopupWindow = inject<boolean>("isPopupWindow", false);

const searchText = ref(props.modelValue || "");
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(searchText, (val) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    emit("update:modelValue", val);
  }, 500);
});

watch(
  () => props.modelValue,
  (val) => {
    if (val !== searchText.value) {
      searchText.value = val || "";
    }
  }
);

function clearSearch() {
  searchText.value = "";
  if (debounceTimer) clearTimeout(debounceTimer);
  emit("update:modelValue", "");
}

async function openWindow(label: string, url: string, title: string) {
  try {
    const existing = await WebviewWindow.getByLabel(label);
    if (existing) {
      try {
        await existing.show();
        existing.setFocus().catch(() => {});
        return;
      } catch {
        // show 失败，窗口可能已销毁，继续创建新窗口
      }
    }
  } catch {
    // getByLabel 失败，继续创建新窗口
  }

  try {
    new WebviewWindow(label, {
      url,
      title,
      width: 900,
      height: 600,
      resizable: true,
      center: true,
    });
  } catch (error) {
    console.error(`打开窗口 "${label}" 失败:`, error);
  }
}

function openEnums() {
  openWindow("enums", "/enums", "枚举值管理");
}

function openLogs() {
  openWindow("logs", "/logs", "日志");
}

function openSettings() {
  openWindow("settings", "/settings", "设置");
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
        <button
          v-if="searchText"
          class="search-clear-btn"
          @click="clearSearch"
          title="清除搜索"
        >
          <X :size="12" />
        </button>
      </div>
      <button
        v-if="!isPopupWindow"
        class="toolbar-icon-btn"
        @click="openEnums"
        title="枚举值管理"
      >
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
  position: relative;
  z-index: 1001;
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
.search-clear-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  flex-shrink: 0;
  transition: all 0.15s;
}
.search-clear-btn:hover {
  color: var(--text-primary);
  background-color: var(--bg-hover);
}
</style>