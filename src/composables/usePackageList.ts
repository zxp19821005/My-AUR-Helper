/**
 * usePackageList.ts - 软件包列表页面逻辑
 *
 * 功能：
 * - 管理列表分页、搜索、选择状态
 * - 提供格式化函数和弹窗控制逻辑
 */
import { computed, ref, watch, inject, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { usePackageStore } from "../stores/packages";
import { useSettingsStore } from "../stores/settings";
import { FOOTER_KEY } from "./footer";
import type { SoftwareListEntry } from "../types";

export function usePackageList() {
  const pkgStore = usePackageStore();
  const footer = inject(FOOTER_KEY)!;
  const settingsStore = useSettingsStore();

  const pageSize = ref(50);
  const currentPage = ref(1);
  const entries = ref<SoftwareListEntry[]>([]);
  const selectedPkgnames = ref(new Set<string>());
  const searchQuery = ref("");

  const showModal = ref(false);
  const modalMode = ref<"add" | "edit">("add");
  const modalPkgname = ref("");
  const showDetailModal = ref(false);
  const detailPkgname = ref("");

  onMounted(async () => {
    pageSize.value = await settingsStore.getSettingNumber("list_page_size_software", 50);
  });

  const filteredEntries = computed(() => {
    if (!searchQuery.value) return entries.value;
    const q = searchQuery.value.toLowerCase();
    return entries.value.filter((e) =>
      e.pkgname.toLowerCase().includes(q) ||
      (e.aur_version && e.aur_version.toLowerCase().includes(q)) ||
      (e.upstream_version && e.upstream_version.toLowerCase().includes(q))
    );
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

  return {
    pageSize,
    currentPage,
    entries,
    selectedPkgnames,
    searchQuery,
    showModal,
    modalMode,
    modalPkgname,
    showDetailModal,
    detailPkgname,
    filteredEntries,
    totalRecords,
    pageData,
    fetchView,
    toggleSelect,
    toggleSelectAll,
    openAddModal,
    openEditModal,
    openDetailModal,
    onModalSaved,
    setSelected,
    syncToolbar,
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