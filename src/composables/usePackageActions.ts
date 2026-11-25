/**
 * usePackageActions.ts - 软件包操作逻辑（组合版）
 *
 * 功能：
 * - 组合同步、检查、删除等操作
 * - 提供统一的加载状态管理
 *
 * 子模块：
 * - usePackageSyncActions: 同步操作
 * - usePackageCheckActions: 版本检查操作
 * - usePackageDeleteActions: 删除操作
 */
import { computed, ref } from "vue";
import { usePackageSyncActions, type PackageBatchStats } from "./usePackageSyncActions";
import { usePackageCheckActions } from "./usePackageCheckActions";
import { usePackageDeleteActions } from "./usePackageDeleteActions";

export type { PackageBatchStats } from "./usePackageSyncActions";

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
  // 批量操作统计（由 syncActions 更新，PackageList 顶部进度条消费）
  const batchStats = ref<PackageBatchStats>({ success: 0, failed: 0, total: 0 });

  // 组合各操作模块
  const syncActions = usePackageSyncActions(fetchView, refreshEntries, syncToolbar, batchStats);
  const checkActions = usePackageCheckActions(refreshEntries, syncToolbar, syncActions.setRowLoading, syncActions.clearRowLoading);
  const deleteActions = usePackageDeleteActions(refreshEntries, syncToolbar);

  // 工具栏按钮禁用态：任一同步/检查操作进行中即禁用同类按钮，防止重复提交。
  // 注意这只是按钮级 guard，界面其余部分保持可用（长任务进度走 footer 局部进度条）。
  const loading = computed(
    () => syncActions.loading.value || checkActions.loading.value
  );

  return {
    // 加载状态
    loading,
    batchStats,
    isRowLoading: syncActions.isRowLoading,
    setRowLoading: syncActions.setRowLoading,
    clearRowLoading: syncActions.clearRowLoading,
    // 同步操作
    syncFromAur: syncActions.syncFromAur,
    syncByMaintainer: syncActions.syncByMaintainer,
    syncFromPkgbuild: syncActions.syncFromPkgbuild,
    updateAurInfo: syncActions.updateAurInfo,
    rowSyncFromAur: syncActions.rowSyncFromAur,
    rowSyncFromPkgbuild: syncActions.rowSyncFromPkgbuild,
    // 检查操作
    checkLoading: checkActions.loading,
    checkSelectedUpstream: checkActions.checkSelectedUpstream,
    rowCheckUpstream: checkActions.rowCheckUpstream,
    // 删除操作
    deleteSelected: deleteActions.deleteSelected,
    rowDeleteSelected: deleteActions.rowDeleteSelected,
  };
}
