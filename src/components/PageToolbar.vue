<script setup lang="ts">
import { Settings, List, ScrollText } from "@lucide/vue";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

async function openEnums() {
  const existing = await WebviewWindow.getByLabel("enums");
  if (existing) { existing.setFocus(); return; }
  new WebviewWindow("enums", {
    url: "/enums", title: "枚举值管理", width: 900, height: 600, resizable: true, center: true,
  });
}

async function openLogs() {
  const existing = await WebviewWindow.getByLabel("logs");
  if (existing) { existing.setFocus(); return; }
  new WebviewWindow("logs", {
    url: "/logs", title: "日志", width: 900, height: 600, resizable: true, center: true,
  });
}

async function openSettings() {
  const existing = await WebviewWindow.getByLabel("settings");
  if (existing) { existing.setFocus(); return; }
  new WebviewWindow("settings", {
    url: "/settings", title: "设置", width: 900, height: 600, resizable: true, center: true,
  });
}
</script>

<template>
  <div class="page-toolbar">
    <div class="toolbar-left">
      <slot />
    </div>
    <div class="toolbar-right">
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
</style>
