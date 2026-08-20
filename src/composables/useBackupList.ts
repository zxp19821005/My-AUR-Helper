/**
 * useBackupList.ts - 备份管理列表页面逻辑
 *
 * 功能：
 * - 管理列表分页、搜索、选择状态
 * - 提供格式化函数和操作控制逻辑
 *
 * 列表通用逻辑（分页/搜索/过滤/选择/工具栏同步）由 useListBase 提供，
 * 本文件仅保留备份特有的：子目录筛选、架构筛选、加载函数。
 */
import { computed, ref } from "vue";
import { useListBase } from "./useListBase";
import type { BackupSoftwareEntry } from "../types";
import * as backupApi from "@/api/backup";

export function useBackupList() {
  const subdirectoryFilter = ref("");
  const subdirectories = ref<string[]>([]);
  const archFilter = ref("");

  const base = useListBase<BackupSoftwareEntry>({
    pageSizeSetting: "list_page_size_backup",
    getKey: (e) => e.id,
    infoText: (t) => `总计: ${t} 个备份文件`,
    pageResetRefs: [subdirectoryFilter, archFilter],
    filter: (all, q) => {
      let result = all;
      if (subdirectoryFilter.value) {
        result = result.filter((e) => e.subdirectory === subdirectoryFilter.value);
      }
      if (archFilter.value) {
        result = result.filter((e) => e.arch === archFilter.value);
      }
      if (q) {
        result = result.filter(
          (e) => e.pkgname.toLowerCase().includes(q) || e.filename.toLowerCase().includes(q)
        );
      }
      return result;
    },
  });

  const architectures = computed(() => {
    const set = new Set<string>();
    for (const e of base.entries.value) if (e.arch) set.add(e.arch);
    return Array.from(set).sort();
  });

  async function fetchEntries() {
    base.loading.value = true;
    try {
      base.entries.value = await backupApi.listBackupSoftware();
    } finally {
      base.loading.value = false;
      base.syncToolbar();
    }
  }

  return {
    ...base,
    subdirectoryFilter,
    subdirectories,
    archFilter,
    architectures,
    fetchEntries,
  };
}

/**
 * 格式化 epoch 为显示文本
 * @param epoch - 版本 epoch
 * @returns 格式化的字符串
 */
export function fmtEpoch(epoch: number): string {
  if (epoch === 0) return "-";
  return `${epoch}`;
}
