/**
 * usePackageDeleteActions.ts - 软件包删除操作逻辑
 *
 * 功能：
 * - 提供删除操作（批量删除、单行删除）
 * - 显示确认对话框
 */
import { formatErrorMessage } from "@/utils/errors";
import { showActionError } from "./useToast";
import { openConfirm as confirm } from "./useConfirm";
import * as softwareApi from "@/api/software";

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
          showActionError(formatErrorMessage(e, `${pkgname} 删除`));
        }
      }
      setSelectedPkgnames(new Set());
      await refreshEntries(list);
    } catch (e) {
      showActionError(formatErrorMessage(e, "批量删除"));
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
      showActionError(formatErrorMessage(e, `${pkgname} 删除`));
    } finally {
      syncToolbar();
    }
  }

  return {
    deleteSelected,
    rowDeleteSelected,
  };
}
