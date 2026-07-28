/**
 * useCacheList.ts - 缓存管理列表页面逻辑
 *
 * 功能：
 * - 管理列表分页、搜索、选择状态
 * - 扫描所有启用的缓存目录
 */
import { computed, ref, watch, inject, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../stores/settings";
import { FOOTER_KEY } from "./footer";
import type { CachePackage } from "../types";

export function useCacheList() {
  const footer = inject(FOOTER_KEY)!;
  const settingsStore = useSettingsStore();

  const pageSize = ref(50);
  const currentPage = ref(1);
  const entries = ref<CachePackage[]>([]);
  const selectedIds = ref(new Set<number>());
  const searchQuery = ref("");
  const loading = ref(false);

  onMounted(async () => {
    pageSize.value = await settingsStore.getSettingNumber("list_page_size_cache", 50);
  });

  const filteredEntries = computed(() => {
    if (!searchQuery.value) return entries.value;
    const q = searchQuery.value.toLowerCase();
    return entries.value.filter((e) =>
      e.name.toLowerCase().includes(q) ||
      e.filename.toLowerCase().includes(q)
    );
  });

  const totalRecords = computed(() => filteredEntries.value.length);

  const pageData = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value;
    return filteredEntries.value.slice(start, start + pageSize.value);
  });

  function syncToolbar() {
    const s = filteredEntries.value;
    footer.infoText = `总计: ${s.length} 个缓存文件`;
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
  watch(currentPage, (p) => {
    footer.currentPage = p;
    footer.onPageChange = goToPage;
  });

  async function fetchEntries() {
    loading.value = true;
    try {
      entries.value = await invoke<CachePackage[]>("scan_all_cache_dirs");
    } finally {
      loading.value = false;
      syncToolbar();
    }
  }

  function toggleSelect(index: number) {
    const s = new Set(selectedIds.value);
    if (s.has(index)) s.delete(index);
    else s.add(index);
    selectedIds.value = s;
  }

  function toggleSelectAll() {
    if (pageData.value.every((_, i) => {
      const globalIdx = (currentPage.value - 1) * pageSize.value + i;
      return selectedIds.value.has(globalIdx);
    })) {
      selectedIds.value = new Set();
    } else {
      const newSet = new Set<number>();
      pageData.value.forEach((_, i) => {
        newSet.add((currentPage.value - 1) * pageSize.value + i);
      });
      selectedIds.value = newSet;
    }
  }

  return {
    pageSize,
    currentPage,
    entries,
    selectedIds,
    searchQuery,
    loading,
    filteredEntries,
    totalRecords,
    pageData,
    fetchEntries,
    toggleSelect,
    toggleSelectAll,
    syncToolbar,
  };
}

/**
 * 格式化文件大小为人类可读字符串
 */
export function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(1) + " " + units[i];
}
