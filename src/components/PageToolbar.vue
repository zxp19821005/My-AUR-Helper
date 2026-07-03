<!--
  PageToolbar.vue - 页面工具栏组件

  功能：
  - 显示当前页面标题
  - 提供操作按钮区域（通过 slot）
  - 右侧固定按钮：枚举值管理、日志、设置
  - 点击按钮打开独立的 Tauri 弹出窗口

  Props:
  - title: string - 页面标题

  弹出窗口：
  - 枚举值管理：管理 License 和编程语言
  - 日志：查看应用日志
  - 设置：应用设置
-->
<script setup lang="ts">
import { Settings, List, ScrollText } from "@lucide/vue";              // Lucide 图标
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";         // Tauri 多窗口管理 API

/** 页面标题 - 父组件传入 */
defineProps<{ title: string }>();

/**
 * 打开枚举值管理窗口
 * 如果窗口已存在则聚焦，否则创建新窗口
 * 窗口标签为 "enums"，用于后续的窗口查找复用
 */
async function openEnums() {
  const existing = await WebviewWindow.getByLabel("enums");
  if (existing) {
    existing.setFocus();  // 窗口已存在，聚焦
    return;
  }
  // 创建新的弹出窗口
  new WebviewWindow("enums", {
    url: "/enums",
    title: "枚举值管理",
    width: 900,
    height: 600,
    resizable: true,
    center: true,
  });
}

/**
 * 打开日志窗口
 * 如果窗口已存在则聚焦，否则创建新窗口
 */
async function openLogs() {
  const existing = await WebviewWindow.getByLabel("logs");
  if (existing) {
    existing.setFocus();
    return;
  }
  new WebviewWindow("logs", {
    url: "/logs",
    title: "日志",
    width: 900,
    height: 600,
    resizable: true,
    center: true,
  });
}

/**
 * 打开设置窗口
 * 如果窗口已存在则聚焦，否则创建新窗口
 */
async function openSettings() {
  const existing = await WebviewWindow.getByLabel("settings");
  if (existing) {
    existing.setFocus();
    return;
  }
  new WebviewWindow("settings", {
    url: "/settings",
    title: "设置",
    width: 900,
    height: 600,
    resizable: true,
    center: true,
  });
}
</script>

<template>
  <div class="page-toolbar">
    <!-- 页面标题 -->
    <h2 class="toolbar-title">{{ title }}</h2>

    <!-- 右侧操作按钮区域 -->
    <div class="toolbar-actions">
      <!-- 自定义操作按钮插槽 - 可由父组件注入额外按钮 -->
      <slot />
      <div class="toolbar-divider"></div>

      <!-- 枚举值管理按钮 -->
      <button class="toolbar-icon-btn" @click="openEnums" title="枚举值管理">
        <List :size="18" />
      </button>

      <!-- 日志按钮 -->
      <button class="toolbar-icon-btn" @click="openLogs" title="日志">
        <ScrollText :size="18" />
      </button>

      <div class="toolbar-divider"></div>

      <!-- 设置按钮 -->
      <button class="toolbar-icon-btn" @click="openSettings" title="设置">
        <Settings :size="18" />
      </button>
    </div>
  </div>
</template>

<style scoped>
/* 工具栏容器 - 左右两端对齐 */
.page-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.625rem 1.25rem;
  border-bottom: 1px solid var(--border);
  background-color: var(--bg-primary);
  min-height: 44px;
}

/* 页面标题 */
.toolbar-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}

/* 操作按钮容器 */
.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

/* 分隔线 - 垂直细线 */
.toolbar-divider {
  width: 1px;
  height: 20px;
  background-color: var(--border);
  margin: 0 0.375rem;
}

/* 图标按钮 - 无背景，悬停时显示背景色 */
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
