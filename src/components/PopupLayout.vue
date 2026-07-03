<!--
  PopupLayout.vue - 通用弹出窗口布局组件

  功能：
  - 提供统一的弹出窗口布局结构
  - 左侧导航菜单 + 右侧内容区域
  - 用于设置、枚举值管理、日志等弹出窗口

  Props:
  - title: string - 窗口标题
  - icon: Component - 标题图标
  - menuItems: MenuItem[] - 左侧导航菜单项

  使用示例：
  <PopupLayout title="设置" :icon="SettingsIcon" :menuItems="menuItems" />
-->
<script setup lang="ts">
import { useRouter, useRoute } from "vue-router";      // 路由 API
import type { Component } from "vue";                  // Vue 组件类型

/** 菜单项配置接口 - 定义导航菜单项的属性 */
interface MenuItem {
  /** 路由路径 - 点击菜单跳转的目标 */
  path: string;
  /** 显示标签 - 菜单项的文字说明 */
  label: string;
  /** 图标组件 - Lucide 图标实例 */
  icon: Component;
}

defineProps<{
  /** 窗口标题 - 显示在侧边栏顶部 */
  title: string;
  /** 标题图标 - 显示在标题左侧 */
  icon: Component;
  /** 导航菜单项 - 左侧菜单列表 */
  menuItems: MenuItem[];
}>();

const router = useRouter();
const route = useRoute();

/**
 * 导航到指定路径
 * @param path - 目标路由路径
 */
function navigate(path: string) {
  router.push(path);
}
</script>

<template>
  <div class="popup-layout">
    <!-- 左侧导航栏 -->
    <aside class="popup-sidebar">
      <!-- 标题区域 - 图标 + 标题文字 -->
      <div class="popup-sidebar-header">
        <component :is="icon" :size="20" />
        <span>{{ title }}</span>
      </div>

      <!-- 导航菜单 - 遍历 menuItems 渲染菜单项 -->
      <nav class="popup-nav">
        <div
          v-for="item in menuItems"
          :key="item.path"
          class="popup-nav-item"
          :class="{ active: route.path === item.path }"
          @click="navigate(item.path)"
        >
          <!-- 菜单图标 -->
          <component :is="item.icon" :size="18" />
          <!-- 菜单标签 -->
          <span>{{ item.label }}</span>
        </div>
      </nav>
    </aside>

    <!-- 右侧内容区域 - 显示当前路由对应的子页面 -->
    <main class="popup-content">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
/* 弹出窗口布局容器 - 左右分栏 */
.popup-layout {
  display: flex;
  height: 100vh;
  background-color: var(--bg-primary);
}

/* 左侧导航栏 - 固定宽度 */
.popup-sidebar {
  width: 200px;
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

/* 标题区域 - 图标和文字水平排列 */
.popup-sidebar-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 1rem;
  border-bottom: 1px solid var(--border);
  font-weight: 700;
  color: var(--accent);
}

/* 导航菜单容器 */
.popup-nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0.5rem;
}

/* 导航项 - 图标和标签水平排列 */
.popup-nav-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  padding: 0.625rem 0.75rem;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.15s;
  font-size: 0.875rem;
}

.popup-nav-item:hover {
  background-color: var(--bg-card);
  color: var(--text-primary);
}

/* 选中状态 - 强调色高亮 */
.popup-nav-item.active {
  background-color: var(--bg-card);
  color: var(--accent);
}

/* 右侧内容区域 - 可滚动 */
.popup-content {
  flex: 1;
  padding: 1.5rem;
  overflow-y: auto;
}
</style>
