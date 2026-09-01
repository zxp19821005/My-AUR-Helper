/**
 * packageDeleteActions.ts - 软件包删除操作逻辑
 *
 * 功能：
 * - 提供删除操作（批量删除、单行删除）
 * - 显示确认对话框
 */
import { inject } from "vue";
import { FOOTER_KEY, addMessage } from "./footer";
import { openConfirm as confirm } from "./useConfirm";
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

export interface DeleteActionsReturn {
  deleteSelected: (
    selectedPkgnames: Set<string>,
    setSelectedPkgnames: (v: Set<string>) => void
  ) => Promise<void>;
  rowDeleteSelected: (
    pkgname: string,
    selectedPkgnames: Set<string>,
    setSelectedPkgnames: (v: Set<string>) => void
  ) => Promise<void>;
}

/**
 * 删除操作钩子
 * @param refreshEntries - 定向刷新指定软件包条目的回调函数
 * @param syncToolbar - 同步工具栏状态的回调函数
 */
export function usePackageDeleteActions(
  refreshEntries: (pkgnames: string[]) => Promise<void>,
  syncToolbar: () => void
) {
  const footer = inject(FOOTER_KEY)!;

  /** 显示错误信息（记录到日志面板） */
  function showError(msg: string) {
    addMessage(footer, "error", msg);
  }

  async function deleteSelected(
    selectedPkgnames: Set<string>,
    setSelectedPkgnames: (v: Set<string>) => void
  ) {
    const list = Array.from(selectedPkgnames);
    if (!list.length) return;
    if (!(await confirm({ message: `确认删除选中的 ${list.length} 个软件包？`, variant: "danger" }))) return;

    try {
      for (const pkgname of list) {
        try {
          await softwareApi.batchDeleteSoftware([pkgname]);
        } catch (e) {
          showError(formatErrorMessage(e, `${pkgname} 删除`));
        }
      }
      setSelectedPkgnames(new Set());
      await refreshEntries(list);
    } catch (e) {
      showError(formatErrorMessage(e, "批量删除"));
    } finally {
      syncToolbar();
    }
  }

  async function rowDeleteSelected(
    pkgname: string,
    selectedPkgnames: Set<string>,
    setSelectedPkgnames: (v: Set<string>) => void
  ) {
    if (!(await confirm({ message: `确认删除 ${pkgname}？`, variant: "danger" }))) return;

    try {
      await softwareApi.batchDeleteSoftware([pkgname]);
      setSelectedPkgnames(
        new Set(Array.from(selectedPkgnames).filter((n) => n !== pkgname))
      );
      await refreshEntries([pkgname]);
    } catch (e) {
      showError(formatErrorMessage(e, `${pkgname} 删除`));
    } finally {
      syncToolbar();
    }
  }

  return {
    deleteSelected,
    rowDeleteSelected,
  };
}
