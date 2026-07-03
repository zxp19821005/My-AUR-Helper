<!--
  EnumLayout.vue - 枚举值管理弹出窗口组件

  功能：
  - 提供枚举值管理窗口的布局和菜单配置
  - 使用 PopupLayout 通用组件

  菜单项：
  - License 管理：管理软件许可证信息
  - 编程语言管理：管理编程语言信息
-->
<script setup lang="ts">
import { List, FileText, Code } from "@lucide/vue";      // Lucide 图标
import PopupLayout from "./PopupLayout.vue";              // 通用弹出窗口布局组件

/** 枚举值管理菜单项配置 - 定义左侧导航菜单 */
const menuItems = [
  { path: "/enums/licenses", label: "License 管理", icon: FileText },     // License 管理
  { path: "/enums/languages", label: "编程语言管理", icon: Code },        // 编程语言管理
];
</script>

<template>
  <!-- 使用 PopupLayout 通用布局，传入标题、图标和菜单配置 -->
  <PopupLayout title="枚举值管理" :icon="List" :menuItems="menuItems" />
</template>
