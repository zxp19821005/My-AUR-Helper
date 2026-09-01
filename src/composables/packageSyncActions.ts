/**
 * packageSyncActions.ts - 软件包同步操作逻辑
 *
 * 功能：
 * - 提供包的同步操作（AUR 同步、PKGBUILD 同步、AUR 信息更新）
 * - 支持批量操作和单行操作
 * - 管理加载状态和进度反馈
 */
import { ref, inject } from "vue";
import { listen } from "@tauri-apps/api/event";
import { FOOTER_KEY, addMessage } from "./footer";
import * as softwareApi from "@/api/software";

/** 判断错误是否为网络相关错误 */
function isNetworkError(e: unknown): boolean {
  const msg = String(e).toLowerCase();
  return (
    msg.includes("网络") ||
    msg.includes("timeout") ||
    msg.includes("连接") ||
    msg.includes("connect") ||
    msg.includes("fetch") ||
    msg.includes("request failed")
  );
}

/** 格式化错误提示信息 */
function formatErrorMessage(e: unknown, context: string): string {
  const msg = String(e);
  if (isNetworkError(e)) {
    return `${context}：网络异常，请检查网络连接后重试。${msg.length > 60 ? "（详情见日志）" : ""}`;
  }
  return `${context}失败：${msg}`;
}

export interface SyncActionsReturn {
  loading: ReturnType<typeof ref<boolean>>;
  loadingKeys: ReturnType<typeof ref<Set<string>>>;
  isRowLoading: (pkgname: string, action?: string) => boolean;
  setRowLoading: (pkgname: string, action: string) => void;
  clearRowLoading: (pkgname: string, action: string) => void;
  syncFromAur: (selectedPkgnames: Set<string>) => Promise<void>;
  syncFromPkgbuild: (selectedPkgnames: Set<string>) => Promise<void>;
  updateAurInfo: (selectedPkgnames: Set<string>) => Promise<void>;
  rowSyncFromAur: (pkgname: string) => Promise<void>;
  rowSyncFromPkgbuild: (pkgname: string) => Promise<void>;
}

/**
 * 同步操作钩子
 * @param fetchView - 整表刷新列表的回调函数
 * @param refreshEntries - 定向刷新指定软件包条目的回调函数
 * @param syncToolbar - 同步工具栏状态的回调函数
 */
export function usePackageSyncActions(
  fetchView: () => Promise<void>,
  refreshEntries: (pkgnames: string[]) => Promise<void>,
  syncToolbar: () => void
) {
  const footer = inject(FOOTER_KEY)!;

  // 全局加载状态（用于工具栏批量操作）
  const loading = ref(false);
  // 按包名+操作类型追踪加载状态（用于行操作）
  const loadingKeys = ref(new Set<string>());
  let unlistenProgress: (() => void) | null = null;

  function isRowLoading(pkgname: string, action?: string): boolean {
    if (action) {
      return loadingKeys.value.has(`${pkgname}:${action}`);
    }
    return Array.from(loadingKeys.value).some(k => k.startsWith(`${pkgname}:`));
  }

  function setRowLoading(pkgname: string, action: string) {
    loadingKeys.value.add(`${pkgname}:${action}`);
  }

  function clearRowLoading(pkgname: string, action: string) {
    loadingKeys.value.delete(`${pkgname}:${action}`);
  }

  /** 显示错误信息（记录到日志面板） */
  function showError(msg: string) {
    addMessage(footer, "error", msg);
  }

  async function syncFromAur(selectedPkgnames: Set<string>) {
    loading.value = true;
    try {
      const list = Array.from(selectedPkgnames);
      if (list.length) {
        for (const pkgname of list) {
          try {
            await softwareApi.updateAurInfo([pkgname]);
            await refreshEntries([pkgname]);
          } catch (e) {
            showError(formatErrorMessage(e, `${pkgname} AUR同步`));
          }
        }
        await refreshEntries(list);
      } else {
        await softwareApi.syncFromAur();
        await fetchView();
      }
    } catch (e) {
      showError(formatErrorMessage(e, "AUR同步"));
    } finally {
      loading.value = false;
      syncToolbar();
    }
  }

  async function syncFromPkgbuild(selectedPkgnames: Set<string>) {
    loading.value = true;
    footer.progress = { current: 0, total: 1, message: "准备中..." };
    try {
      unlistenProgress = await listen<{
        current: number;
        total: number;
        pkgname: string;
        message: string;
      }>("sync-progress", (event) => {
        const { current, total, message } = event.payload;
        footer.progress = { current, total, message };
      });

      const list = Array.from(selectedPkgnames);
      if (list.length) {
        for (const pkgname of list) {
          try {
            await softwareApi.syncFromPkgbuild(pkgname);
          } catch (e) {
            showError(formatErrorMessage(e, `${pkgname} PKGBUILD同步`));
          }
        }
        await refreshEntries(list);
      } else {
        await softwareApi.syncFromPkgbuild(null);
        await fetchView();
      }
    } catch (e) {
      showError(formatErrorMessage(e, "PKGBUILD同步"));
    } finally {
      unlistenProgress?.();
      unlistenProgress = null;
      footer.progress = null;
      loading.value = false;
      syncToolbar();
    }
  }

  async function updateAurInfo(selectedPkgnames: Set<string>) {
    loading.value = true;
    try {
      const list = Array.from(selectedPkgnames);
      if (list.length) {
        for (const pkgname of list) {
          try {
            await softwareApi.updateAurInfo([pkgname]);
            await refreshEntries([pkgname]);
          } catch (e) {
            showError(formatErrorMessage(e, `${pkgname} AUR信息更新`));
          }
        }
        await refreshEntries(list);
      } else {
        await softwareApi.updateAurInfo(null);
        await fetchView();
      }
    } catch (e) {
      showError(formatErrorMessage(e, "AUR信息更新"));
    } finally {
      loading.value = false;
      syncToolbar();
    }
  }

  async function rowSyncFromAur(pkgname: string) {
    setRowLoading(pkgname, "sync-aur");
    try {
      await softwareApi.updateAurInfo([pkgname]);
      await refreshEntries([pkgname]);
    } catch (e) {
      showError(formatErrorMessage(e, `${pkgname} AUR同步`));
    } finally {
      clearRowLoading(pkgname, "sync-aur");
      syncToolbar();
    }
  }

  async function rowSyncFromPkgbuild(pkgname: string) {
    setRowLoading(pkgname, "sync-pkgbuild");
    try {
      await softwareApi.syncFromPkgbuild(pkgname);
      await refreshEntries([pkgname]);
    } catch (e) {
      showError(formatErrorMessage(e, `${pkgname} PKGBUILD同步`));
    } finally {
      clearRowLoading(pkgname, "sync-pkgbuild");
      syncToolbar();
    }
  }

  return {
    loading,
    loadingKeys,
    isRowLoading,
    setRowLoading,
    clearRowLoading,
    syncFromAur,
    syncFromPkgbuild,
    updateAurInfo,
    rowSyncFromAur,
    rowSyncFromPkgbuild,
  };
}
