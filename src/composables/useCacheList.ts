/**
 * useCacheList.ts - 缓存管理列表页面逻辑
 *
 * 功能：
 * - 管理列表分页、搜索、选择状态
 * - 从 cache_software 数据库表读取已存在的存量数据（页面打开自动执行）
 * - 扫描所有启用的缓存目录重新写入数据库（手动触发 "扫描" 按钮）
 */
import { computed, ref, watch, inject, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../stores/settings";
import { FOOTER_KEY } from "./footer";
import type { CachePackage, CacheSoftwareEntry } from "../types";

/**
 * 列表展示时使用的统一条目类型（兼容两种数据源）
 * - 从数据库读取：字段 id/pkgname/cache_directory 有值
 * - 从磁盘扫描（PkgFileInfo）：用 name 当 pkgname，epoch 为字符串或 null
 */
export interface CacheListEntry {
  /** 数据库记录 ID（仅从数据库读有值，磁盘扫描为 -1 占位） */
  id: number;
  /** 包名，用于展示和搜索 */
  pkgname: string;
  /** 缓存文件名 */
  filename: string;
  /** epoch */
  epoch: number;
  /** 版本号 */
  pkgver: string;
  /** pkgrel */
  pkgrel: string;
  /** 架构 */
  arch: string;
  /** 缓存目录完整路径 */
  cache_directory: string;
}

export function useCacheList() {
  const footer = inject(FOOTER_KEY)!;
  const settingsStore = useSettingsStore();

  const pageSize = ref(50);
  const currentPage = ref(1);
  const entries = ref<CacheListEntry[]>([]);
  const selectedIds = ref(new Set<number>());
  const searchQuery = ref("");
  const loading = ref(false);

  onMounted(async () => {
    pageSize.value = await settingsStore.getSettingNumber("list_page_size_cache", 50);
    // 页面打开时，自动从 cache_software 表读取存量数据
    try {
      await loadEntries();
    } catch (e) {
      console.error("[缓存管理] 页面打开时加载存量缓存数据失败:", e);
    }
  });

  /** 把前端 CachePackage（来自 PkgFileInfo 扫描结果）转成列表条目 */
  function fromCachePackage(p: CachePackage, idx: number): CacheListEntry {
    const epoch = p.epoch ? parseInt(p.epoch, 10) || 0 : 0;
    return {
      id: -1 - idx,
      pkgname: p.name || pkgFromFilename(p.filename),
      filename: p.filename,
      epoch,
      pkgver: p.pkgver,
      pkgrel: p.pkgrel,
      arch: p.arch,
      cache_directory: "",
    };
  }

  /** 把后端 CacheSoftwareEntry 转成列表条目 */
  function fromCacheSoftwareEntry(e: CacheSoftwareEntry): CacheListEntry {
    return {
      id: e.id,
      pkgname: e.pkgname || pkgFromFilename(e.filename),
      filename: e.filename,
      epoch: e.epoch,
      pkgver: e.pkgver,
      pkgrel: e.pkgrel,
      arch: e.arch,
      cache_directory: e.cache_directory,
    };
  }

  /** 从文件名解析包名（兜底） */
  function pkgFromFilename(fn: string): string {
    const base = fn.replace(/\.pkg\.tar\.zst$/, "");
    const parts = base.rsplit("-", 3);
    if (parts.length < 3) return base;
    const nv = parts[0];
    const dash = nv.lastIndexOf("-");
    return dash >= 0 ? nv.substring(0, dash) : nv;
  }

  const filteredEntries = computed(() => {
    if (!searchQuery.value) return entries.value;
    const q = searchQuery.value.toLowerCase();
    return entries.value.filter((e) =>
      e.pkgname.toLowerCase().includes(q) ||
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

  /**
   * 从数据库 cache_software 表读取所有存量数据（页面打开时用）
   * 不会扫描磁盘，只执行 SELECT 查询，速度快
   */
  async function loadEntries() {
    loading.value = true;
    try {
      const data = await invoke<CacheSoftwareEntry[]>("list_cache_software");
      entries.value = data.map(fromCacheSoftwareEntry);
      selectedIds.value = new Set();
    } finally {
      loading.value = false;
      syncToolbar();
    }
  }

  /**
   * 扫描所有启用的缓存目录（磁盘），清空并重建 cache_software 表
   * 对应原来的 fetchEntries，仅在用户点击 "扫描" 按钮时触发
   */
  async function rescanAllDirs() {
    loading.value = true;
    try {
      const scanned = await invoke<CachePackage[]>("scan_all_cache_dirs");
      entries.value = scanned.map(fromCachePackage);
      selectedIds.value = new Set();
    } finally {
      loading.value = false;
      syncToolbar();
    }
  }

  /** 兼容旧代码，保留 fetchEntries 指向 rescanAllDirs（按钮点击仍可用） */
  async function fetchEntries() {
    await rescanAllDirs();
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
    loadEntries,
    rescanAllDirs,
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

// 给 js/ts 使用的 rsplit 工具（避免修改 String.prototype）
declare global {
  interface String {
    rsplit(sep: string, maxsplit?: number): string[];
  }
}
if (typeof String.prototype.rsplit !== "function") {
  // eslint-disable-next-line no-extend-native
  String.prototype.rsplit = function (sep: string, maxsplit?: number): string[] {
    const split = this.split(sep);
    if (typeof maxsplit === "undefined" || maxsplit <= 0 || maxsplit >= split.length) {
      return split;
    }
    const start = split.length - maxsplit;
    return [
      split.slice(0, start).join(sep),
      ...split.slice(start),
    ];
  };
}