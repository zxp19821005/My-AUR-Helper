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
import { ref, watch, onMounted, reactive, provide } from "vue";
import { useRoute } from "vue-router";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import Sidebar from "./components/Sidebar.vue";
import TabBar from "./components/TabBar.vue";
import BottomToolbar from "./components/BottomToolbar.vue";
import { useTabStore } from "./stores/tabs";
import { FOOTER_KEY, defaultFooterState } from "./composables/footer";
import type { FooterState } from "./composables/footer";

const route = useRoute();
const tabStore = useTabStore();

const sidebarCollapsed = ref(true);
const isPopupWindow = ref(false);

/** 底部工具栏状态 - 通过 provide/inject 共享 */
const footerState = reactive<FooterState>(defaultFooterState());
provide(FOOTER_KEY, footerState);

onMounted(async () => {
  const win = getCurrentWebviewWindow();
  isPopupWindow.value = win.label === "settings" || win.label === "enums" || win.label === "logs";
});

/** 监听路由变化 */
watch(
  () => route.path,
  (path) => {
    Object.assign(footerState, defaultFooterState());
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
        <!-- 底部工具栏 - 信息显示、分页、进度条 -->
        <BottomToolbar />
      </div>
    </template>
  </div>
</template>
