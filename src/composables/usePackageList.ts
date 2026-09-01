/**
 * usePackageList.ts - 软件包列表页面逻辑
 *
 * 功能：
 * - 管理列表分页、搜索、选择状态
 * - 提供格式化函数和弹窗控制逻辑
 * - 支持筛选器功能（多重快速筛选 OR + 单选快速筛选 OR + 条件筛选 AND）
 *
 * 筛选说明：
 * - 多重快速筛选（quickFilters）：由筛选面板 FilterBar 以复选框控制，OR 逻辑
 * - 单选快速筛选（quickFilter）：由工具栏下拉控制，仅允许同时选中一项，
 *   与多重快速筛选之间同样为 OR 关系（命中任一即通过）
 * - 条件筛选（conditionFilters）：软件包类型 / 检查器类型，AND 逻辑
 */
import { computed, ref, shallowRef, triggerRef, watch, inject, onMounted } from "vue";
import { useSettingsStore } from "../stores/settings";
import { FOOTER_KEY } from "./footer";
import type { SoftwareListEntry } from "../types";
import * as softwareApi from "@/api/software";

/** 筛选状态 */
export interface FilterState {
  /** 多重快速筛选条件（OR 逻辑，由筛选面板 FilterBar 控制） */
  quickFilters: {
    /** 上游 URL 为空 */
    upstreamUrlEmpty: boolean;
    /** AUR 版本为空（旧语义，保留给 FilterBar） */
    aurUpdateFailed: boolean;
    /** 上游版本为空（旧语义，保留给 FilterBar） */
    upstreamUpdateFailed: boolean;
    /** 上游地址异常（upstream_url_status != "ok" 且非空） */
    upstreamUrlAbnormal: boolean;
    /** License 缺失（upstream_license_id 为空） */
    licenseMissing: boolean;
  };
  /** 单选快速筛选（工具栏下拉，与多重筛选 OR 叠加） */
  quickFilter: QuickFilterKey | null;
  /** 条件筛选（AND 逻辑） */
  conditionFilters: {
    /** 软件包类型 */
    packageType: number | null;
    /** 检查器类型 */
    checkerType: number | null;
  };
}

/** 单选快速筛选条件键（工具栏下拉，不支持多选） */
export type QuickFilterKey =
  | "aur_sync_failed"
  | "upstream_check_failed"
  | "github_url_invalid"
  | "no_aur_version"
  | "no_upstream_version"
  | "no_upstream_url"
  | "license_missing";

/** 工具栏快速筛选下拉选项（单选，不支持多选） */
export const quickFilterOptions: { key: QuickFilterKey; label: string }[] = [
  { key: "aur_sync_failed", label: "AUR更新失败" },
  { key: "upstream_check_failed", label: "上游更新失败" },
  { key: "github_url_invalid", label: "Github地址失效" },
  { key: "no_aur_version", label: "无AUR版本" },
  { key: "no_upstream_version", label: "无上游版本" },
  { key: "no_upstream_url", label: "无上游地址" },
  { key: "license_missing", label: "License缺失" },
];

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
    quickFilter: null,
    conditionFilters: {
      packageType: null,
      checkerType: null,
    },
  };
}

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

  /** 判断单条是否命中某个单选快速筛选条件（基于新错误字段，语义准确） */
  function matchesSingleQuickFilter(entry: SoftwareListEntry, key: QuickFilterKey): boolean {
    switch (key) {
      case "aur_sync_failed":
        // AUR 同步失败：最近一次同步写入了错误原因（aur_version 是否为空另算「无AUR版本」）
        return entry.aur_sync_error != null;
      case "upstream_check_failed":
        // 上游检查失败：最近一次检查写入了错误原因
        return entry.upstream_check_error != null;
      case "github_url_invalid":
        // Github 地址失效：URL 已配置但验证状态非 ok
        return !!entry.upstream_url_status && entry.upstream_url_status !== "ok";
      case "no_aur_version":
        return entry.aur_version == null;
      case "no_upstream_version":
        return entry.upstream_version == null;
      case "no_upstream_url":
        return entry.upstream_url == null;
      case "license_missing":
        return entry.upstream_license_id == null;
      default:
        return false;
    }
  }

  /** 检查是否满足快速筛选条件（多重 OR + 单选 OR，任一命中即通过） */
  function matchesQuickFilters(entry: SoftwareListEntry): boolean {
    const qf = filterState.value.quickFilters;
    const checks: boolean[] = [];
    if (qf.upstreamUrlEmpty) checks.push(entry.upstream_url == null);
    if (qf.aurUpdateFailed) checks.push(entry.aur_version == null);
    if (qf.upstreamUpdateFailed) checks.push(entry.upstream_version == null);
    if (qf.upstreamUrlAbnormal)
      checks.push(!!entry.upstream_url_status && entry.upstream_url_status !== "ok");
    if (qf.licenseMissing) checks.push(entry.upstream_license_id == null);
    const single = filterState.value.quickFilter;
    if (single) checks.push(matchesSingleQuickFilter(entry, single));
    // 没有任何激活的快筛 → 通过；有激活的 → 任一命中即通过
    return checks.length === 0 || checks.some((v) => v);
  }

  /** 检查是否满足条件筛选（AND 逻辑） */
  function matchesConditionFilters(entry: SoftwareListEntry): boolean {
    const cf = filterState.value.conditionFilters;
    if (cf.packageType !== null && entry.package_type_id !== cf.packageType) return false;
    if (cf.checkerType !== null && entry.checker_type_id !== cf.checkerType) return false;
    return true;
  }

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
    let result = entries.value;

    // 快速筛选（多重 OR + 单选 OR）
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
        pkgnames.map((name) =>
          softwareApi.getSoftwareListEntry(name)
        )
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

/**
 * 格式化时间戳为中文日期
 * @param ts - Unix 时间戳（秒）
 * @returns 格式化的日期字符串，如 "2024-01-15"
 */
export function fmtTimestamp(ts: number | null): string {
  if (ts == null) return "-";
  const d = new Date(ts * 1000);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}
