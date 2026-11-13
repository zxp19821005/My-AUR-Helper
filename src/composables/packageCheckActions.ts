/**
 * packageCheckActions.ts - 软件包版本检查操作逻辑
 *
 * 功能：
 * - 提供版本检查操作
 * - 支持批量检查和单行检查
 */
import { ref, inject } from "vue";
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

export interface CheckActionsReturn {
  loading: import("vue").Ref<boolean>;
  setRowLoading: (pkgname: string, action: string) => void;
  clearRowLoading: (pkgname: string, action: string) => void;
  checkSelectedUpstream: (selectedPkgnames: Set<string>) => Promise<void>;
  rowCheckUpstream: (pkgname: string) => Promise<void>;
}

/**
 * 版本检查操作钩子
 * @param refreshEntries - 定向刷新指定软件包条目的回调函数
 * @param syncToolbar - 同步工具栏状态的回调函数
 * @param setRowLoading - 设置行级加载状态
 * @param clearRowLoading - 清除行级加载状态
 */
export function usePackageCheckActions(
  refreshEntries: (pkgnames: string[]) => Promise<void>,
  syncToolbar: () => void,
  setRowLoading: CheckActionsReturn['setRowLoading'],
  clearRowLoading: CheckActionsReturn['clearRowLoading'],
) {
  const footer = inject(FOOTER_KEY)!;
  /** 检查操作进行中状态（用于工具栏按钮 disabled） */
  const loading = ref(false);

  /** 显示错误信息（记录到日志面板） */
  function showError(msg: string) {
    addMessage(footer, "error", msg);
  }

  async function checkSelectedUpstream(selectedPkgnames: Set<string>) {
    loading.value = true;
    try {
      const list = Array.from(selectedPkgnames);
      if (list.length) {
        for (const pkgname of list) {
          try {
            await softwareApi.checkSelectedUpstream([pkgname]);
            await refreshEntries([pkgname]);
          } catch (e) {
            showError(formatErrorMessage(e, `${pkgname} 上游检查`));
          }
        }
        await refreshEntries(list);
      } else {
        await softwareApi.checkAllUpstream();
        await refreshEntries([]);
      }
    } catch (e) {
      showError(formatErrorMessage(e, "上游检查"));
    } finally {
      loading.value = false;
      syncToolbar();
    }
  }

  async function rowCheckUpstream(pkgname: string) {
    loading.value = true;
    setRowLoading(pkgname, 'check-upstream');
    try {
      await softwareApi.checkSelectedUpstream([pkgname]);
      await refreshEntries([pkgname]);
    } catch (e) {
      showError(formatErrorMessage(e, `${pkgname} 上游检查`));
    } finally {
      loading.value = false;
      clearRowLoading(pkgname, 'check-upstream');
      syncToolbar();
    }
  }

  return {
    loading,
    checkSelectedUpstream,
    rowCheckUpstream,
  };
}
