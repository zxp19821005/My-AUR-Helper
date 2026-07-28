/**
 * useBackupList.ts - 备份管理列表页面逻辑
 *
 * 功能：
 * - 管理列表分页、搜索、选择状态
 * - 提供格式化函数和操作控制逻辑
 */
import { computed, ref, watch, inject, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../stores/settings";
import { FOOTER_KEY } from "./footer";
import type { BackupSoftwareEntry } from "../types";

export function useBackupList() {
  const footer = inject(FOOTER_KEY)!;
  const settingsStore = useSettingsStore();

  const pageSize = ref(50);
  const currentPage = ref(1);
  const entries = ref<BackupSoftwareEntry[]>([]);
  const selectedIds = ref(new Set<number>());
  const searchQuery = ref("");
  const subdirectoryFilter = ref("");
  const subdirectories = ref<string[]>([]);
  const loading = ref(false);

  onMounted(async () => {
    pageSize.value = await settingsStore.getSettingNumber("list_page_size_backup", 50);
  });

  const filteredEntries = computed(() => {
    let result = entries.value;
    if (subdirectoryFilter.value) {
      result = result.filter((e) => e.subdirectory === subdirectoryFilter.value);
    }
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter((e) =>
        e.pkgname.toLowerCase().includes(q) ||
        e.filename.toLowerCase().includes(q)
      );
    }
    return result;
  });

  const totalRecords = computed(() => filteredEntries.value.length);

  const pageData = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value;
    return filteredEntries.value.slice(start, start + pageSize.value);
  });

  function syncToolbar() {
    const s = filteredEntries.value;
    footer.infoText = `总计: ${s.length} 个备份文件`;
    footer.showPagination = s.length > pageSize.value;
    footer.totalRecords = s.length;
    footer.currentPage = currentPage.value;
    footer.pageSize = pageSize.value;
    footer.onPageChange = goToPage;
  }

  function goToPage(page: number) {
    currentPage.value = page;
  }

  watch(totalRecords, syncToolbar);
  watch(searchQuery, () => { currentPage.value = 1; });
  watch(subdirectoryFilter, () => { currentPage.value = 1; });
  watch(currentPage, (p) => {
    footer.currentPage = p;
    footer.onPageChange = goToPage;
  });

  async function fetchEntries() {
    loading.value = true;
    try {
      entries.value = await invoke<BackupSoftwareEntry[]>("list_backup_software");
    } finally {
      loading.value = false;
      syncToolbar();
    }
  }

  function toggleSelect(id: number) {
    const s = new Set(selectedIds.value);
    if (s.has(id)) s.delete(id);
    else s.add(id);
    selectedIds.value = s;
  }

  function toggleSelectAll() {
    if (pageData.value.every((p) => selectedIds.value.has(p.id))) {
      selectedIds.value = new Set();
    } else {
      selectedIds.value = new Set(pageData.value.map((p) => p.id));
    }
  }

  const setSelected = (v: Set<number>) => { selectedIds.value = v; };

  return {
    pageSize,
    currentPage,
    entries,
    selectedIds,
    searchQuery,
    subdirectoryFilter,
    subdirectories,
    loading,
    filteredEntries,
    totalRecords,
    pageData,
    fetchEntries,
    toggleSelect,
    toggleSelectAll,
    setSelected,
    syncToolbar,
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
