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
import { useTabStore } from "../../stores/tabs";
import { useRefreshStore } from "../../stores/refresh";
import { Icon } from "../../icons";
import { openPopup } from "../../composables/usePopupWindow";

const router = useRouter();
const tabStore = useTabStore();
const refreshStore = useRefreshStore();

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

function openEnums() {
  openPopup("enums", "/enums", "枚举值管理");
}

function openLogs() {
  openPopup("logs", "/logs", "日志");
}

function openSettings() {
  openPopup("settings", "/settings", "设置");
}

/**
 * 通用刷新按钮 - 触发当前路由页面的刷新回调
 * 由各页面通过 useRefreshSubscription 订阅
 */
function handleRefresh() {
  refreshStore.trigger();
}
</script>

<template>
  <!-- 标签栏容器 - 水平排列，左侧标签页，右侧工具按钮 -->
  <div class="tab-bar">
    <!-- 左侧：已打开的标签页 -->
    <div class="tab-bar-tabs">
      <div
        v-for="tab in tabStore.openTabs"
        :key="tab.path"
        class="tab"
        :class="{ active: tabStore.activeTab === tab.path }"
        @click="switchTab(tab.path)"
      >
        <!-- 标签图标 -->
        <component :is="tab.icon" :size="14" class="tab-icon" />
        <!-- 标签文字 -->
        <span class="tab-label">{{ tab.label }}</span>
        <!-- 关闭按钮 - 默认隐藏，悬停时显示 -->
        <button class="tab-close" @click="closeTab(tab.path, $event)" title="关闭">
          <component :is="Icon.actionClear" :size="12" />
        </button>
      </div>
    </div>

    <!-- 右侧：工具按钮 -->
    <div class="tab-bar-actions">
      <button class="tab-action-btn" @click="openEnums" title="枚举值管理">
        <component :is="Icon.menuEnums" :size="16" />
      </button>
      <button class="tab-action-btn" @click="openLogs" title="日志">
        <component :is="Icon.menuLogs" :size="16" />
      </button>
      <button class="tab-action-btn" @click="openSettings" title="设置">
        <component :is="Icon.navSettings" :size="16" />
      </button>
      <button class="tab-action-btn tab-action-refresh" @click="handleRefresh" title="刷新数据">
        <component :is="Icon.actionRefresh" :size="16" />
      </button>
    </div>
  </div>
</template>

<style scoped>
/* 标签栏容器 - 水平排列，左右分布 */
.tab-bar {
  display: flex;
  align-items: center;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  height: 36px;
}

/* 左侧标签页容器 */
.tab-bar-tabs {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
  overflow-x: auto;
  scrollbar-width: none;
}

.tab-bar-tabs::-webkit-scrollbar {
  display: none;
}

/* 右侧工具按钮容器 */
.tab-bar-actions {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  padding-right: 0.5rem;
  flex-shrink: 0;
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
  opacity: 0;
  transition: opacity 0.15s;
}

.tab:hover .tab-close {
  opacity: 1;
}

.tab-close:hover {
  background-color: var(--bg-card);
  color: var(--error);
}

/* 工具按钮 */
.tab-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.tab-action-btn:hover {
  background-color: var(--bg-card);
  color: var(--text-primary);
}

/* 刷新按钮 - 蓝色强调 */
.tab-action-refresh {
  color: #3b82f6;
}

.tab-action-refresh:hover {
  background-color: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}
</style>
