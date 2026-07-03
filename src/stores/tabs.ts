/**
 * tabs.ts - 标签页状态管理（Pinia Store）
 *
 * 功能：
 * - 管理已打开的标签页列表
 * - 管理当前激活的标签页
 * - 提供打开、关闭、切换标签页方法
 * - 类 VS Code 风格的标签页行为
 */
import { defineStore } from "pinia";  // Pinia 状态管理库
import { ref } from "vue";            // Vue 响应式 API

/** 标签页接口 - 定义单个标签页的数据结构 */
export interface Tab {
  /** 路由路径 - 标签页对应的页面路由 */
  path: string;
  /** 显示标签 - 标签页上显示的文字 */
  label: string;
  /** 图标名称 - 标签页图标的标识符，用于动态渲染图标组件 */
  icon: string;
}

/** 创建 tabs Store，用于管理标签页的全局状态 */
export const useTabStore = defineStore("tabs", () => {
  /** 已打开的标签页列表 - 存储所有已打开页面的标签数组 */
  const openTabs = ref<Tab[]>([]);

  /** 当前激活的标签页路径 - 当前选中标签页的路由路径 */
  const activeTab = ref<string>("/");

  /**
   * 打开标签页
   * 如果标签页已存在则切换到该标签，否则添加新标签并切换到它
   * @param tab - 标签页信息（路径、标签、图标）
   */
  function openTab(tab: Tab) {
    const exists = openTabs.value.find((t) => t.path === tab.path);
    if (!exists) {
      openTabs.value.push(tab);
    }
    activeTab.value = tab.path;
  }

  /**
   * 关闭标签页
   * 如果关闭的是当前激活的标签页，则自动切换到相邻标签
   * 如果没有标签页了，则打开仪表盘作为默认页面
   * @param path - 要关闭的标签页路径
   */
  function closeTab(path: string) {
    const idx = openTabs.value.findIndex((t) => t.path === path);
    if (idx === -1) return;
    openTabs.value.splice(idx, 1);
    if (activeTab.value === path) {
      if (openTabs.value.length === 0) {
        // 没有标签页时，回退到仪表盘
        activeTab.value = "/";
        openTab({ path: "/", label: "仪表盘", icon: "LayoutDashboard" });
      } else {
        // 切换到相邻标签（优先左侧，如果被删的是最后一个则切换到前一个）
        const newIdx = Math.min(idx, openTabs.value.length - 1);
        activeTab.value = openTabs.value[newIdx].path;
      }
    }
  }

  /**
   * 切换到指定标签页
   * 仅更新活跃标签，不触发导航
   * @param path - 目标标签页路径
   */
  function switchTab(path: string) {
    activeTab.value = path;
  }

  return { openTabs, activeTab, openTab, closeTab, switchTab };
});
