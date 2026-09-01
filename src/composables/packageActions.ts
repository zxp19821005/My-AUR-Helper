/**
 * packageActions.ts - 软件包操作逻辑（组合版）
 *
 * 功能：
 * - 组合同步、检查、删除等操作
 * - 提供统一的加载状态管理
 *
 * 子模块：
 * - packageSyncActions: 同步操作
 * - packageCheckActions: 版本检查操作
 * - packageDeleteActions: 删除操作
 */
import { ref } from "vue";
import { usePackageSyncActions } from "./packageSyncActions";
import { usePackageCheckActions } from "./packageCheckActions";
import { usePackageDeleteActions } from "./packageDeleteActions";

export interface PackageBatchStats {
  success: number;
  failed: number;
  total: number;
}

/**
 * 软件包操作钩子（组合版）
 * @param fetchView - 整表刷新列表的回调函数
 * @param refreshEntries - 定向刷新指定软件包条目的回调函数
 * @param syncToolbar - 同步工具栏状态的回调函数
 */
export function usePackageActions(
  fetchView: () => Promise<void>,
  refreshEntries: (pkgnames: string[]) => Promise<void>,
  syncToolbar: () => void
) {
  // 全局加载状态（用于工具栏批量操作）
  const loading = ref(false);
  // 批量操作统计
  const batchStats = ref<PackageBatchStats>({ success: 0, failed: 0, total: 0 });

  // 组合各操作模块
  const syncActions = usePackageSyncActions(fetchView, refreshEntries, syncToolbar);
  const checkActions = usePackageCheckActions(refreshEntries, syncToolbar);
  const deleteActions = usePackageDeleteActions(refreshEntries, syncToolbar);

  return {
    // 加载状态
    loading,
    batchStats,
    isRowLoading: syncActions.isRowLoading,
    setRowLoading: syncActions.setRowLoading,
    clearRowLoading: syncActions.clearRowLoading,
    // 同步操作
    syncFromAur: syncActions.syncFromAur,
    syncFromPkgbuild: syncActions.syncFromPkgbuild,
    updateAurInfo: syncActions.updateAurInfo,
    rowSyncFromAur: syncActions.rowSyncFromAur,
    rowSyncFromPkgbuild: syncActions.rowSyncFromPkgbuild,
    // 检查操作
    checkSelectedUpstream: checkActions.checkSelectedUpstream,
    rowCheckUpstream: checkActions.rowCheckUpstream,
    // 删除操作
    deleteSelected: deleteActions.deleteSelected,
    rowDeleteSelected: deleteActions.rowDeleteSelected,
  };
}
