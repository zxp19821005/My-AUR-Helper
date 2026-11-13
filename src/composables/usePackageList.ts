/**
 * usePackageList.ts - 软件包列表页面逻辑
 *
 * 管理列表分页、搜索、选择状态，提供格式化函数和弹窗控制逻辑。
 * 筛选相关的类型 / 选项 / 默认态 / 纯匹配函数已拆分至 filterConfig.ts，
 * 仅在此 re-export 以兼容 FilterBar 等既有引用方。
 */
import { computed, ref, shallowRef, triggerRef, watch, inject, onMounted } from "vue";
import { useSettingsStore } from "../stores/settings";
import { FOOTER_KEY } from "./footer";
import type { SoftwareListEntry } from "../types";
import * as softwareApi from "@/api/software";
import type { FilterState, QuickFilterKey } from "./filterConfig";
import {
  createDefaultFilterState,
  matchesQuickFilters,
  matchesConditionFilters,
  quickFilterOptions,
} from "./filterConfig";

export type { FilterState, QuickFilterKey } from "./filterConfig";
export { quickFilterOptions };

export function usePackageList() {
  const footer = inject(FOOTER_KEY)!;
  const settingsStore = useSettingsStore();

  const pageSize = ref(50);
  const currentPage = ref(1);
  // 记录「进入搜索前」所在页码：搜索时跳到第 1 页，清除搜索后恢复该页码，
  // 避免在第 N 页搜索后清空搜索直接被拽回第 1 页（问题 3）
  const pageBeforeSearch = ref(1);
  // 使用 shallowRef 承载千级大数组：深度响应式会把 1932 个对象的每个字段
  // 都转换为 getter/setter（约 6 万个代理），每次 invoke 赋值都会同步阻塞
  // 数百毫秒到数秒（dev/debug 构建更明显）。列表行内更新走 Object.assign
  // + triggerRef 手动刷新，详见 refreshEntries。
  const entries = shallowRef<SoftwareListEntry[]>([]);
  const selectedPkgnames = ref(new Set<string>());
  const searchQuery = ref("");
  const filterState = ref<FilterState>(createDefaultFilterState());
  const showFilterBar = ref(false);

  const showModal = ref(false);
  const modalMode = ref<"add" | "edit">("add");
  const modalPkgname = ref("");
  const showDetailModal = ref(false);
  const detailPkgname = ref("");

  onMounted(async () => {
    pageSize.value = await settingsStore.getSettingNumber("list_page_size_software", 50);
  });

  /** 计算活跃筛选条件数量（含单选快速筛选与条件筛选） */
  const activeFilterCount = computed(() => {
    let count = 0;
    const qf = filterState.value.quickFilters;
    if (qf.upstreamUrlEmpty) count++;
    if (qf.aurUpdateFailed) count++;
    if (qf.upstreamUpdateFailed) count++;
    if (qf.upstreamUrlAbnormal) count++;
    if (qf.licenseMissing) count++;
    if (filterState.value.quickFilter) count++;
    if (filterState.value.conditionFilters.packageType !== null) count++;
    if (filterState.value.conditionFilters.checkerType !== null) count++;
    return count;
  });

  /** 工具栏下拉选中某项（再次点击同一项则取消，保证单选） */
  function selectQuickFilter(key: QuickFilterKey) {
    filterState.value = {
      ...filterState.value,
      quickFilter: filterState.value.quickFilter === key ? null : key,
    };
  }

  /** 清除工具栏单选快速筛选 */
  function clearQuickFilter() {
    filterState.value = { ...filterState.value, quickFilter: null };
  }

  /** 当前激活的单选筛选标签（用于工具栏按钮展示） */
  const activeQuickFilterLabel = computed(() => {
    const k = filterState.value.quickFilter;
    if (!k) return "";
    return quickFilterOptions.find((o) => o.key === k)?.label ?? "";
  });

  const filteredEntries = computed(() => {
    const state = filterState.value;
    return entries.value
      .filter((e) => matchesQuickFilters(e, state))
      .filter((e) => matchesConditionFilters(e, state));
  });

  const totalRecords = computed(() => filteredEntries.value.length);

  const pageData = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value;
    return filteredEntries.value.slice(start, start + pageSize.value);
  });

  function syncToolbar() {
    const s = filteredEntries.value;
    const outdated = s.filter((x) => x.is_outdated).length;
    footer.infoText = `总计: ${s.length}  |  已最新: ${s.length - outdated}  |  需更新: ${outdated}`;
    footer.showPagination = s.length > pageSize.value;
    footer.totalRecords = s.length;

    // 边界检查：确保页码不超过总页数
    const totalPages = Math.ceil(s.length / pageSize.value);
    if (totalPages > 0 && currentPage.value > totalPages) {
      currentPage.value = totalPages;
    }

    footer.currentPage = currentPage.value;
    footer.pageSize = pageSize.value;
    footer.onPageChange = goToPage;
  }

  function goToPage(page: number) {
    currentPage.value = page;
  }

  watch(totalRecords, syncToolbar);
  // 问题 3：搜索与页码联动。从「无搜索」进入搜索时记录当前页并跳第 1 页；
  // 从「有搜索」清除搜索时恢复到搜索前所在页码，而非始终回到第 1 页。
  watch(searchQuery, (val, old) => {
    const nowSearching = val.trim() !== "";
    const wasEmpty = (old ?? "").trim() === "";
    if (nowSearching && wasEmpty) {
      pageBeforeSearch.value = currentPage.value;
      currentPage.value = 1;
    } else if (!nowSearching && !wasEmpty) {
      currentPage.value = pageBeforeSearch.value;
    }
  });
  watch(currentPage, (p) => {
    footer.currentPage = p;
    footer.onPageChange = goToPage;
  });

  async function fetchView() {
    try {
      entries.value = await softwareApi.listSoftwareView();
    } finally {
      syncToolbar();
    }
  }

  /**
   * 定向刷新指定软件包的列表条目，避免整表（近两千条）重载。
   * - 传入非空 pkgname 列表：逐条拉取最新条目，就地更新对应数据对象的字段
   *   （保持数组引用不变，Vue 仅重渲染发生变化的那一行，而非整页）
   * - 若某 pkgname 后端返回 null（如已被删除）：从列表原地移除该条目
   * - 传入空列表（语义为"全部"）：回退为整表重载
   * @param pkgnames - 需要刷新的包名列表
   */
  async function refreshEntries(pkgnames: string[]) {
    if (pkgnames.length === 0) {
      await fetchView();
      return;
    }
    try {
      const results = await Promise.all(
        pkgnames.map((name) => softwareApi.getSoftwareListEntry(name)),
      );
      const byName = new Map(results.filter(Boolean).map((e) => [e!.pkgname, e!]));
      const removed = new Set(pkgnames.filter((name) => !byName.has(name)));
      const list = entries.value;
      for (let i = 0; i < list.length; i++) {
        const updated = byName.get(list[i].pkgname);
        if (updated) {
          // 就地更新字段，保持对象引用与数组引用不变，避免整页重渲染
          Object.assign(list[i], updated);
        }
      }
      if (removed.size) {
        entries.value = list.filter((e) => !removed.has(e.pkgname));
      } else {
        // 行内字段已就地更新，但 shallowRef 不追踪嵌套对象变化，需手动触发刷新
        triggerRef(entries);
      }
    } finally {
      syncToolbar();
    }
  }

  function toggleSelect(pkgname: string) {
    const s = new Set(selectedPkgnames.value);
    if (s.has(pkgname)) s.delete(pkgname);
    else s.add(pkgname);
    selectedPkgnames.value = s;
  }

  function toggleSelectAll() {
    if (pageData.value.every((p) => selectedPkgnames.value.has(p.pkgname))) {
      selectedPkgnames.value = new Set();
    } else {
      selectedPkgnames.value = new Set(pageData.value.map((p) => p.pkgname));
    }
  }

  function openAddModal() {
    modalMode.value = "add";
    modalPkgname.value = "";
    showModal.value = true;
  }

  function openEditModal(pkgname: string) {
    modalMode.value = "edit";
    modalPkgname.value = pkgname;
    showModal.value = true;
  }

  function openDetailModal(pkgname: string) {
    detailPkgname.value = pkgname;
    showDetailModal.value = true;
  }

  const setSelected = (v: Set<string>) => {
    selectedPkgnames.value = v;
  };

  function resetFilters() {
    filterState.value = createDefaultFilterState();
  }

  return {
    pageSize,
    currentPage,
    entries,
    selectedPkgnames,
    searchQuery,
    filterState,
    showFilterBar,
    showModal,
    modalMode,
    modalPkgname,
    showDetailModal,
    detailPkgname,
    filteredEntries,
    totalRecords,
    pageData,
    activeFilterCount,
    activeQuickFilterLabel,
    quickFilterOptions,
    selectQuickFilter,
    clearQuickFilter,
    fetchView,
    refreshEntries,
    toggleSelect,
    toggleSelectAll,
    openAddModal,
    openEditModal,
    openDetailModal,
    setSelected,
    syncToolbar,
    resetFilters,
  };
}
