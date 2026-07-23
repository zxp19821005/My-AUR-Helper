/**
 * usePackageList.ts - 软件包列表页面逻辑
 *
 * 功能：
 * - 管理列表分页、搜索、选择状态
 * - 提供格式化函数和弹窗控制逻辑
 * - 支持筛选器功能（快速筛选 + 条件筛选）
 */
import { computed, ref, watch, inject, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePackageStore } from "../stores/packages";
import { useSettingsStore } from "../stores/settings";
import { FOOTER_KEY } from "./footer";
import type { SoftwareListEntry } from "../types";

/** 筛选条件类型 */
export interface FilterState {
  /** 快速筛选条件（OR 逻辑） */
  quickFilters: {
    /** 上游 URL 为空 */
    upstreamUrlEmpty: boolean;
    /** AUR 更新失败（aur_version 为空） */
    aurUpdateFailed: boolean;
    /** 上游更新失败（upstream_version 为空） */
    upstreamUpdateFailed: boolean;
    /** 上游地址异常（upstream_url_status != "ok" 且非空） */
    upstreamUrlAbnormal: boolean;
    /** License 缺失（upstream_license_id 为空） */
    licenseMissing: boolean;
  };
  /** 条件筛选（AND 逻辑） */
  conditionFilters: {
    /** 软件包类型 */
    packageType: number | null;
    /** 检查器类型 */
    checkerType: number | null;
  };
}

/** 默认筛选状态 */
function createDefaultFilterState(): FilterState {
  return {
    quickFilters: {
      upstreamUrlEmpty: false,
      aurUpdateFailed: false,
      upstreamUpdateFailed: false,
      upstreamUrlAbnormal: false,
      licenseMissing: false,
    },
    conditionFilters: {
      packageType: null,
      checkerType: null,
    },
  };
}

export function usePackageList() {
  const pkgStore = usePackageStore();
  const footer = inject(FOOTER_KEY)!;
  const settingsStore = useSettingsStore();

  const pageSize = ref(50);
  const currentPage = ref(1);
  const entries = ref<SoftwareListEntry[]>([]);
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

  /** 检查是否满足快速筛选条件（OR 逻辑） */
  function matchesQuickFilters(entry: SoftwareListEntry): boolean {
    const qf = filterState.value.quickFilters;
    // 如果没有任何快速筛选条件激活，则通过
    const hasActiveQuickFilter =
      qf.upstreamUrlEmpty || qf.aurUpdateFailed || qf.upstreamUpdateFailed ||
      qf.upstreamUrlAbnormal || qf.licenseMissing;
    if (!hasActiveQuickFilter) return true;

    // OR 逻辑：满足任一条件即通过
    if (qf.upstreamUrlEmpty && !entry.upstream_url) return true;
    if (qf.aurUpdateFailed && !entry.aur_version) return true;
    if (qf.upstreamUpdateFailed && !entry.upstream_version) return true;
    if (qf.upstreamUrlAbnormal && entry.upstream_url_status && entry.upstream_url_status !== "ok") return true;
    if (qf.licenseMissing && !entry.upstream_license_id) return true;

    return false;
  }

  /** 检查是否满足条件筛选（AND 逻辑） */
  function matchesConditionFilters(entry: SoftwareListEntry): boolean {
    const cf = filterState.value.conditionFilters;
    if (cf.packageType !== null && entry.package_type_id !== cf.packageType) return false;
    if (cf.checkerType !== null && entry.checker_type_id !== cf.checkerType) return false;
    return true;
  }

  /** 计算活跃筛选条件数量 */
  const activeFilterCount = computed(() => {
    let count = 0;
    const qf = filterState.value.quickFilters;
    if (qf.upstreamUrlEmpty) count++;
    if (qf.aurUpdateFailed) count++;
    if (qf.upstreamUpdateFailed) count++;
    if (qf.upstreamUrlAbnormal) count++;
    if (qf.licenseMissing) count++;
    if (filterState.value.conditionFilters.packageType !== null) count++;
    if (filterState.value.conditionFilters.checkerType !== null) count++;
    return count;
  });

  const filteredEntries = computed(() => {
    let result = entries.value;

    // 搜索过滤
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter((e) =>
        e.pkgname.toLowerCase().includes(q) ||
        (e.aur_version && e.aur_version.toLowerCase().includes(q)) ||
        (e.upstream_version && e.upstream_version.toLowerCase().includes(q))
      );
    }

    // 快速筛选（OR 逻辑）
    result = result.filter(matchesQuickFilters);

    // 条件筛选（AND 逻辑）
    result = result.filter(matchesConditionFilters);

    return result;
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

  async function fetchView() {
    try {
      entries.value = await invoke<SoftwareListEntry[]>("list_software_view");
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

  function onModalSaved() {
    Promise.all([fetchView(), pkgStore.fetchPackages()]);
  }

  const setSelected = (v: Set<string>) => { selectedPkgnames.value = v; };

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
    fetchView,
    toggleSelect,
    toggleSelectAll,
    openAddModal,
    openEditModal,
    openDetailModal,
    onModalSaved,
    setSelected,
    syncToolbar,
    resetFilters,
  };
}

/**
 * 格式化时间戳为中文日期
 * @param ts - Unix 时间戳（秒）
 * @returns 格式化的日期字符串，如 "2024/01/15"
 */
export function fmtTimestamp(ts: number | null): string {
  if (ts == null) return "-";
  const d = new Date(ts * 1000);
  return d.toLocaleDateString("zh-CN", {
    year: "numeric", month: "2-digit", day: "2-digit",
  });
}