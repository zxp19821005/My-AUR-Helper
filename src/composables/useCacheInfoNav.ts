/**
 * useCacheInfoNav.ts - 缓存管理页「详情弹窗导航 + 选择 + 列定义」逻辑
 *
 * 功能：
 * - openCacheInfo / prevEntry / nextEntry / onCacheInfoNavigate: 基于当前列表顺序
 *   在缓存详情弹窗中上一页/下一页切换
 * - handleSelectionChange: 将表格勾选行映射为 filteredEntries 索引集合
 * - selectedFilenames: 由选中索引推导出的文件名列表（供备份动作使用）
 * - cacheColumns: 缓存表格列定义
 *
 * 将这部分与视图解耦，避免 CacheManager.vue 超过 300 行限制。
 */
import { computed, ref } from "vue";
import type { Ref } from "vue";

export interface CacheInfoNavDeps {
  filteredEntries: Ref<any[]>;
  selectedIds: Ref<Set<number>>;
  viewPackageInfo: (row: any) => void;
}

export function useCacheInfoNav({ filteredEntries, selectedIds, viewPackageInfo }: CacheInfoNavDeps) {
  const infoDialogIndex = ref(-1);

  function openCacheInfo(row: any) {
    infoDialogIndex.value = filteredEntries.value.indexOf(row);
    viewPackageInfo(row);
  }

  const prevEntry = computed(() => {
    const i = infoDialogIndex.value;
    return i > 0 ? filteredEntries.value[i - 1] : null;
  });

  const nextEntry = computed(() => {
    const i = infoDialogIndex.value;
    const list = filteredEntries.value;
    return i >= 0 && i < list.length - 1 ? list[i + 1] : null;
  });

  function onCacheInfoNavigate(target: any) {
    const i = filteredEntries.value.indexOf(target);
    if (i >= 0) {
      infoDialogIndex.value = i;
      viewPackageInfo(target);
    }
  }

  function handleSelectionChange(selectedRows: any[]) {
    const newSelected = new Set<number>();
    selectedRows.forEach((row) => {
      const idx = filteredEntries.value.findIndex((e) => e.filename === row.filename);
      if (idx !== -1) newSelected.add(idx);
    });
    selectedIds.value = newSelected;
  }

  const selectedFilenames = computed(() =>
    filteredEntries.value
      .filter((_, i) => selectedIds.value.has(i))
      .map((e) => e.filename),
  );

  return {
    infoDialogIndex,
    openCacheInfo,
    prevEntry,
    nextEntry,
    onCacheInfoNavigate,
    handleSelectionChange,
    selectedFilenames,
  };
}

/** 缓存表格列定义 */
export const cacheColumns = [
  { key: "pkgname", title: "包名" },
  { key: "epoch", title: "Epoch" },
  { key: "pkgver", title: "版本" },
  { key: "pkgrel", title: "PkgRel" },
  { key: "arch", title: "架构" },
  { key: "cache_directory", title: "缓存目录" },
];
