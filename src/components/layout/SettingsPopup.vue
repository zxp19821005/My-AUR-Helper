<!--
  SettingsPopup.vue - 设置弹出窗口组件

  功能：
  - 提供设置窗口的布局和菜单配置
  - 使用 PopupLayout 通用组件

  菜单项：
  - 通用设置：外观、字体等
  - 列表设置：各页面列表显示行数
  - AUR 设置：AUR 相关配置
  - 上游检查器设置：版本检查器配置
  - 备份管理设置：备份相关配置
  - 缓存软件设置：缓存管理配置
  - 代理管理设置：代理相关配置
-->
<script setup lang="ts">
import { Settings, List, Globe, Search, HardDrive, Database, Wifi, ScrollText } from "@lucide/vue";  // Lucide 图标
import PopupLayout from "./PopupLayout.vue";                                        // 通用弹出窗口布局组件

/** 设置菜单项配置 - 定义设置窗口左侧导航菜单的各个分类 */
const menuItems = [
  { path: "/settings", label: "通用设置", icon: Settings },           // 通用设置
  { path: "/settings/list", label: "列表设置", icon: List },          // 列表设置
  { path: "/settings/aur", label: "AUR 设置", icon: Globe },          // AUR 设置
  { path: "/settings/checker", label: "上游检查器设置", icon: Search }, // 检查器设置
  { path: "/settings/backup", label: "备份管理设置", icon: HardDrive },  // 备份设置
  { path: "/settings/cache", label: "缓存软件设置", icon: Database },    // 缓存设置
  { path: "/settings/proxy", label: "代理管理设置", icon: Wifi },       // 代理设置
  { path: "/settings/log", label: "日志管理设置", icon: ScrollText },   // 日志管理设置
];
</script>

<template>
  <!-- 使用 PopupLayout 通用布局，传入标题、图标和菜单配置 -->
  <PopupLayout title="设置" :icon="Settings" :menuItems="menuItems" />
</template>
