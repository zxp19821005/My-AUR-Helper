<!--
  Sidebar.vue - 可收缩侧边栏组件

  功能：
  - 显示应用主导航菜单
  - 支持展开/收起状态切换
  - 收起时仅显示图标，展开时显示图标+文字
  - 点击导航项时同步更新标签页状态

  Props:
  - collapsed: boolean - 是否处于收起状态

  Events:
  - toggle: 点击切换按钮时触发
-->
<script setup lang="ts">
import { useRouter } from "vue-router";
import { PanelLeftClose, PanelLeftOpen } from "@lucide/vue";
import { useTabStore } from "../stores/tabs";
import { iconMap } from "../utils/icons";

defineProps<{ collapsed: boolean }>();
const emit = defineEmits<{ toggle: [] }>();

const router = useRouter();
const tabStore = useTabStore();

/** 导航菜单项配置 - 定义侧边栏中各导航项的路径、标签和图标 */
const navItems = [
  { path: "/", label: "仪表盘", icon: "LayoutDashboard" },
  { path: "/packages", label: "软件管理", icon: "Package" },
  { path: "/backup", label: "备份管理", icon: "HardDrive" },
  { path: "/cache", label: "缓存管理", icon: "Database" },
  { path: "/proxy", label: "代理管理", icon: "Globe" },
];

/**
 * 导航到指定页面
 * 同时更新标签页状态（打开对应标签）和路由导航
 * @param item - 导航项配置对象（路径、标签、图标）
 */
function navigate(item: (typeof navItems)[number]) {
  tabStore.openTab({ path: item.path, label: item.label, icon: item.icon });
  router.push(item.path);
}
</script>

<template>
  <aside class="sidebar" :class="{ collapsed }">
    <!-- 侧边栏头部：显示标题和展开/收起切换按钮 -->
    <div class="sidebar-header">
      <span v-if="!collapsed" class="sidebar-title">My AUR Helper</span>
      <!-- 展开/收起切换按钮 -->
      <button class="sidebar-toggle" @click="emit('toggle')" :title="collapsed ? '展开' : '收起'">
        <PanelLeftOpen v-if="collapsed" :size="18" />
        <PanelLeftClose v-else :size="18" />
      </button>
    </div>

    <!-- 导航菜单 - 遍历 navItems 渲染各导航项 -->
    <nav class="sidebar-nav">
      <div
        v-for="item in navItems"
        :key="item.path"
        class="nav-item"
        :class="{ active: tabStore.activeTab === item.path }"
        @click="navigate(item)"
        :title="collapsed ? item.label : ''"
      >
        <!-- 导航图标 -->
        <component :is="iconMap[item.icon]" :size="20" class="nav-icon" />
        <!-- 导航文字标签 - 收起时隐藏 -->
        <span v-if="!collapsed" class="nav-label">{{ item.label }}</span>
      </div>
    </nav>
  </aside>
</template>

<style scoped>
/* 侧边栏容器 - 固定宽度，深色背景 */
.sidebar {
  width: 200px;
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  transition: width 0.2s ease;
  overflow: hidden;
  flex-shrink: 0;
}

/* 收起状态 - 缩小宽度仅显示图标 */
.sidebar.collapsed {
  width: 56px;
}

/* 头部区域 - 标题和切换按钮 */
.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem;
  border-bottom: 1px solid var(--border);
  min-height: 48px;
}

/* 标题文字 - 强调色加粗 */
.sidebar-title {
  font-size: 0.875rem;
  font-weight: 700;
  color: var(--accent);
  white-space: nowrap;
  overflow: hidden;
}

/* 展开/收起切换按钮 - 无样式按钮 */
.sidebar-toggle {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.25rem;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.sidebar-toggle:hover {
  color: var(--text-primary);
  background-color: var(--bg-card);
}

/* 导航菜单容器 */
.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0.5rem;
  flex: 1;
}

/* 导航项 - 图标和标签水平排列 */
.nav-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0.625rem;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.15s;
  white-space: nowrap;
}

.nav-item:hover {
  background-color: var(--bg-card);
  color: var(--text-primary);
}

/* 选中状态 - 强调色高亮 */
.nav-item.active {
  background-color: var(--bg-card);
  color: var(--accent);
}

/* 图标 - 禁止收缩 */
.nav-icon {
  flex-shrink: 0;
}

/* 标签文字 - 溢出省略 */
.nav-label {
  font-size: 0.875rem;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
