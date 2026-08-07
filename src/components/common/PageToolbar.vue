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

/**
 * 打开（或激活）一个独立 Tauri 子窗口。
 *
 * 变更逻辑（修复重复点击失效）：
 * 1. 若窗口已存在（getByLabel 命中），先 unminimize 再 show + setFocus 重新激活，直接 return。
 *    这一步依赖 capabilities 中授予的 core:window:allow-show / allow-unminimize / allow-set-focus。
 *    早期版本缺少 allow-show 权限，show() 会抛错并误入“重新创建”分支；而重复 label 的
 *    new WebviewWindow 错误是异步事件（tauri://error），外部 try/catch 捕获不到，导致第二次
 *    点击静默失效。补上权限后才真正生效。
 * 2. 若窗口不存在才创建新窗口；创建后监听 tauri://error，极少数竞态下（label 已存在）
 *    补一次聚焦，避免窗口“创建但不可见”。
 */
async function openWindow(label: string, url: string, title: string) {
  // 1) 窗口已存在：取消最小化 -> 显示 -> 聚焦（再次点击切换/激活窗口）
  try {
    const existing = await WebviewWindow.getByLabel(label);
    if (existing) {
      try {
        await existing.unminimize();
      } catch {
        // 部分平台不支持 unminimize，忽略
      }
      try {
        await existing.show();
        await existing.setFocus();
      } catch {
        // show/setFocus 失败，窗口可能已销毁，继续创建新窗口
      }
      return;
    }
  } catch {
    // getByLabel 失败，继续创建新窗口
  }

  // 2) 创建新窗口；duplicate-label 错误通过异步事件发出，不会被下方 try/catch 捕获
  try {
    const win = new WebviewWindow(label, {
      url,
      title,
      width: 900,
      height: 600,
      resizable: true,
      center: true,
    });
    // 极少数竞态下 label 已存在，create 会异步报错，这里补一次聚焦
    win.once("tauri://error", async () => {
      try {
        const existing = await WebviewWindow.getByLabel(label);
        if (existing) {
          try { await existing.unminimize(); } catch { /* ignore */ }
          await existing.show();
          await existing.setFocus();
        }
      } catch {
        // 忽略
      }
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