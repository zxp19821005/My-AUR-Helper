/**
 * viewMode.ts - 视图模式状态管理
 *
 * 功能：
 * - 管理列表页面和块页面的视图模式（list/grid）
 * - 持久化用户选择到 localStorage
 */
import { defineStore } from "pinia";
import { ref, watch } from "vue";

export type ViewMode = "list" | "grid";

const STORAGE_KEY = "view_mode";

export const useViewModeStore = defineStore("viewMode", () => {
  const mode = ref<ViewMode>(
    (localStorage.getItem(STORAGE_KEY) as ViewMode) || "list"
  );

  watch(mode, (newMode) => {
    localStorage.setItem(STORAGE_KEY, newMode);
  });

  function setMode(m: ViewMode) {
    mode.value = m;
  }

  function toggleMode() {
    mode.value = mode.value === "list" ? "grid" : "list";
  }

  return { mode, setMode, toggleMode };
});
