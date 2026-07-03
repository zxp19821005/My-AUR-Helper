<!--
  TabBar.vue - VS Code 风格标签栏组件

  功能：
  - 显示所有已打开的页面标签
  - 支持点击切换标签页
  - 支持关闭标签页（悬停显示关闭按钮）
  - 标签栏支持水平滚动

  依赖：
  - tabs store: 管理标签页状态
  - vue-router: 页面导航
-->
<script setup lang="ts">
import { useRouter } from "vue-router";
import { X } from "@lucide/vue";
import { useTabStore } from "../stores/tabs";
import { iconMap } from "../utils/icons";

const router = useRouter();
const tabStore = useTabStore();

/**
 * 切换到指定标签页
 * 更新 Store 中的活跃标签并执行路由导航
 * @param path - 目标标签页的路由路径
 */
function switchTab(path: string) {
  tabStore.switchTab(path);
  router.push(path);
}

/**
 * 关闭指定标签页
 * 阻止点击事件冒泡（避免触发 switchTab），然后调用 Store 关闭方法
 * 关闭后自动导航到 Store 中新的活跃标签
 * @param path - 要关闭的标签页路径
 * @param event - 点击事件对象
 */
function closeTab(path: string, event: Event) {
  event.stopPropagation();
  tabStore.closeTab(path);
  router.push(tabStore.activeTab);
}
</script>

<template>
  <!-- 标签栏容器 - 水平滚动显示所有已打开的标签 -->
  <div class="tab-bar">
    <div
      v-for="tab in tabStore.openTabs"
      :key="tab.path"
      class="tab"
      :class="{ active: tabStore.activeTab === tab.path }"
      @click="switchTab(tab.path)"
    >
      <!-- 标签图标 -->
      <component :is="iconMap[tab.icon]" :size="14" class="tab-icon" />
      <!-- 标签文字 -->
      <span class="tab-label">{{ tab.label }}</span>
      <!-- 关闭按钮 - 默认隐藏，悬停时显示 -->
      <button class="tab-close" @click="closeTab(tab.path, $event)" title="关闭">
        <X :size="12" />
      </button>
    </div>
  </div>
</template>

<style scoped>
/* 标签栏容器 - 水平排列，可滚动 */
.tab-bar {
  display: flex;
  align-items: center;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  height: 36px;
  overflow-x: auto;
  scrollbar-width: none;  /* Firefox 隐藏滚动条 */
}

.tab-bar::-webkit-scrollbar {
  display: none;  /* WebKit 隐藏滚动条 */
}

/* 单个标签 - 图标+文字+关闭按钮 */
.tab {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0 0.75rem;
  height: 100%;
  cursor: pointer;
  color: var(--text-secondary);
  border-right: 1px solid var(--border);
  white-space: nowrap;
  font-size: 0.8125rem;
  transition: background-color 0.15s;
  user-select: none;
}

.tab:hover {
  background-color: var(--bg-card);
}

/* 选中标签 - 底部强调色边框 */
.tab.active {
  background-color: var(--bg-primary);
  color: var(--text-primary);
  border-bottom: 2px solid var(--accent);
}

/* 标签图标 - 禁止收缩 */
.tab-icon {
  flex-shrink: 0;
}

/* 标签文字 - 溢出省略 */
.tab-label {
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 关闭按钮 - 默认隐藏，父级悬停时显示 */
.tab-close {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.125rem;
  border-radius: 3px;
  margin-left: 0.25rem;
  opacity: 0;          /* 默认隐藏 */
  transition: opacity 0.15s;
}

.tab:hover .tab-close {
  opacity: 1;          /* 悬停时显示 */
}

.tab-close:hover {
  background-color: var(--bg-card);
  color: var(--error);  /* 关闭按钮悬停变红 */
}
</style>
