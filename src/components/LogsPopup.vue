<!--
  LogsPopup.vue - 日志弹出窗口组件

  功能：
  - 提供日志查看窗口的布局和菜单配置
  - 使用 PopupLayout 通用组件

  菜单项：
  - 日志查看：查看应用运行日志
-->
<script setup lang="ts">
import { ScrollText } from "@lucide/vue";      // Lucide 图标
import PopupLayout from "./PopupLayout.vue";    // 通用弹出窗口布局组件

/** 日志菜单项配置 - 定义左侧导航菜单 */
const menuItems = [
  { path: "/logs", label: "日志查看", icon: ScrollText },  // 日志查看入口
];
</script>

<template>
  <!-- 使用 PopupLayout 通用布局，传入标题、图标和菜单配置 -->
  <PopupLayout title="日志" :icon="ScrollText" :menuItems="menuItems" />
</template>
