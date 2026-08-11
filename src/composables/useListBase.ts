/**
 * useListBase.ts - 列表页通用逻辑基类（泛型）
 *
 * 提取备份/缓存/代理三个列表 composable 的共有部分：
 * - 分页状态（pageSize / currentPage）与列表页大小设置项
 * - 搜索、条目集合、加载态、选择集合
 * - 过滤（搜索 + 类型筛选）、分页切片、总数
 * - 工具栏同步（footer.infoText / 分页 / onPageChange）
 * - 选择键提取（getKey）、toggleSelect / toggleSelectAll / setSelected
 * - 筛选变化自动回到第 1 页
 *
 * 调用方只需提供差异部分：过滤谓词、选择键、工具栏文案、额外筛选源、加载函数。
 */
import { computed, ref, watch, inject, onMounted, type Ref, type WatchSource } from "vue";
import { useSettingsStore } from "../stores/settings";
import { FOOTER_KEY } from "./footer";

export interface UseListBaseOptions<T> {
  /** 列表页大小设置项名（读取自设置 store） */
  pageSizeSetting: string;
  /**
   * 全量过滤谓词：传入条目数组与已 lowercase 的搜索串 q（空串表示无搜索）。
   * 闭包内读取调用方的类型筛选 ref，computed 会自动追踪其变化。
   */
  filter: (all: T[], q: string) => T[];
  /** 选择键提取：传入条目及其在 filteredEntries 中的全局索引，返回稳定的选择键 */
  getKey: (entry: T, globalIndex: number) => number;
  /** 工具栏信息文本生成（参数为过滤后总条数） */
  infoText: (total: number) => string;
  /** 变化时需重置到第 1 页的额外筛选源（searchQuery 已内置监听） */
  pageResetRefs?: WatchSource<unknown>[];
}

export function useListBase<T>(options: UseListBaseOptions<T>) {
  const footer = inject(FOOTER_KEY)!;
  const settingsStore = useSettingsStore();

  const pageSize = ref(50);
  const currentPage = ref(1);
  const entries = ref<T[]>([]) as Ref<T[]>;
  const selectedIds = ref(new Set<number>());
  const searchQuery = ref("");
  const loading = ref(false);

  onMounted(async () => {
    pageSize.value = await settingsStore.getSettingNumber(options.pageSizeSetting, 50);
  });

  const filteredEntries = computed(() => {
    const q = searchQuery.value.trim().toLowerCase();
    return options.filter(entries.value, q);
  });

  const totalRecords = computed(() => filteredEntries.value.length);

  const pageData = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value;
    return filteredEntries.value.slice(start, start + pageSize.value);
  });

  function goToPage(page: number) {
    currentPage.value = page;
  }

  function syncToolbar() {
    const s = filteredEntries.value;
    footer.infoText = options.infoText(s.length);
    footer.showPagination = s.length > pageSize.value;
    footer.totalRecords = s.length;
    footer.currentPage = currentPage.value;
    footer.pageSize = pageSize.value;
    footer.onPageChange = goToPage;
  }

  watch(totalRecords, syncToolbar);
  watch(searchQuery, () => { currentPage.value = 1; });
  watch(currentPage, (p) => {
    footer.currentPage = p;
    footer.onPageChange = goToPage;
  });
  if (options.pageResetRefs) {
    watch(options.pageResetRefs, () => { currentPage.value = 1; });
  }

  function toggleSelect(key: number) {
    const s = new Set(selectedIds.value);
    if (s.has(key)) s.delete(key);
    else s.add(key);
    selectedIds.value = s;
  }

  function toggleSelectAll() {
    const allKeys = pageData.value.map((e, i) =>
      options.getKey(e, (currentPage.value - 1) * pageSize.value + i)
    );
    if (allKeys.length > 0 && allKeys.every((k) => selectedIds.value.has(k))) {
      selectedIds.value = new Set();
    } else {
      selectedIds.value = new Set(allKeys);
    }
  }

  const setSelected = (v: Set<number>) => { selectedIds.value = v; };

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
    goToPage,
    syncToolbar,
    toggleSelect,
    toggleSelectAll,
    setSelected,
  };
}
