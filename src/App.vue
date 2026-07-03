<!--
  App.vue - 应用根组件

  功能：
  - 根据窗口类型（主窗口/弹出窗口）显示不同布局
  - 主窗口布局：侧边栏 + 标签栏 + 工具栏 + 内容区
  - 弹出窗口布局：直接显示路由内容

  窗口检测：
  - 通过 WebviewWindow.label 判断窗口类型
  - settings: 设置窗口
  - enums: 枚举值管理窗口
  - logs: 日志窗口
-->
<script setup lang="ts">
import { ref, watch, onMounted } from "vue";                              // Vue 核心 API：响应式数据、侦听器、生命周期
import { useRoute } from "vue-router";                                    // 路由 API：获取当前路由信息
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";  // Tauri Webview 窗口 API：获取当前窗口信息
import Sidebar from "./components/Sidebar.vue";                           // 侧边栏导航组件
import TabBar from "./components/TabBar.vue";                             // 标签栏组件（类 VS Code 风格）
import { useTabStore } from "./stores/tabs";                              // 标签页状态管理 Store

const route = useRoute();           // 当前路由对象
const tabStore = useTabStore();     // 标签页 Store 实例

/** 侧边栏是否收起 - 控制侧边栏展开/折叠状态 */
const sidebarCollapsed = ref(true);

/** 是否为弹出窗口 - 标记当前窗口类型，影响布局渲染 */
const isPopupWindow = ref(false);

/**
 * 组件挂载时检测窗口类型
 * 通过 Tauri 的 WebviewWindow API 获取当前窗口标签
 * 如果是弹出窗口（settings/enums/logs），则不显示主窗口布局
 */
onMounted(async () => {
  const win = getCurrentWebviewWindow();
  isPopupWindow.value = win.label === "settings" || win.label === "enums" || win.label === "logs";
});

/**
 * 监听路由变化，自动打开对应标签页
 */
watch(
  () => route.path,
  (path) => {
    const routeLabels: Record<string, string> = {
      "/": "仪表盘",
      "/packages": "软件管理",
      "/backup": "备份管理",
      "/cache": "缓存管理",
      "/proxy": "代理管理",
    };
    if (path.startsWith("/packages/")) {
      const pkgName = path.split("/packages/")[1];
      tabStore.openTab({ path, label: pkgName || "软件详情", icon: "Package" });
    } else if (routeLabels[path]) {
      tabStore.openTab({ path, label: routeLabels[path], icon: "Package" });
    }
  },
  { immediate: true }
);
</script>

<template>
  <div id="app-container">
    <!-- 弹出窗口布局：直接显示路由内容，不显示侧边栏和标签栏 -->
    <template v-if="isPopupWindow">
      <RouterView />
    </template>

    <!-- 主窗口布局：包含侧边栏、标签栏、工具栏、内容区 -->
    <template v-else>
      <!-- 左侧可收缩导航侧边栏 -->
      <Sidebar :collapsed="sidebarCollapsed" @toggle="sidebarCollapsed = !sidebarCollapsed" />
      <div class="main-area">
        <!-- 顶部标签栏 - 显示已打开的页面标签 -->
        <TabBar />
        <!-- 主内容区 - 渲染当前路由对应的组件 -->
        <main class="main-content">
          <RouterView />
        </main>
      </div>
    </template>
  </div>
</template>
