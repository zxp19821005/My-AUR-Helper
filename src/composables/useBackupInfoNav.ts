/**
 * useBackupInfoNav.ts - 备份管理页「详情弹窗导航 + 选择」逻辑
 *
 * 功能：
 * - dialogIndex / prevEntry / nextEntry: 基于当前列表顺序在备份详情弹窗中
 *   上一条/下一条切换（以 infoDialogEntry.id 定位）
 * - onDialogNavigate: 详情弹窗内的导航回调
 * - handleSelectionChange: 将表格勾选行映射为 filteredEntries 索引集合
 *
 * 将这部分与视图解耦，避免 BackupManager.vue 超过 300 行限制。
 */
import { computed } from "vue";
import type { Ref } from "vue";

export interface BackupInfoNavDeps {
  filteredEntries: Ref<any[]>;
  selectedIds: Ref<Set<number>>;
  infoDialogEntry: Ref<any>;
  viewPackageInfo: (entry: any) => void;
}

export function useBackupInfoNav({
  filteredEntries,
  selectedIds,
  infoDialogEntry,
  viewPackageInfo,
}: BackupInfoNavDeps) {
  /** 当前弹窗条目在 filteredEntries 中的索引 */
  const dialogIndex = computed(() => {
    if (!infoDialogEntry.value) return -1;
    return filteredEntries.value.findIndex(
      (e: any) => e.id === infoDialogEntry.value!.id,
    );
  });

  /** 上一个条目 */
  const prevEntry = computed(() => {
    const idx = dialogIndex.value;
    return idx > 0 ? filteredEntries.value[idx - 1] : null;
  });

  /** 下一个条目 */
  const nextEntry = computed(() => {
    const idx = dialogIndex.value;
    const list = filteredEntries.value;
    return idx >= 0 && idx < list.length - 1 ? list[idx + 1] : null;
  });

  /** 详情弹窗：上一个/下一个 导航 */
  function onDialogNavigate(target: any) {
    viewPackageInfo(target);
  }

  function handleSelectionChange(selectedRows: any[]) {
    const newSelected = new Set<number>();
    selectedRows.forEach((row: any) => {
      const idx = filteredEntries.value.findIndex((e: any) => e.id === row.id);
      if (idx !== -1) newSelected.add(idx);
    });
    selectedIds.value = newSelected;
  }

  return { dialogIndex, prevEntry, nextEntry, onDialogNavigate, handleSelectionChange };
}
