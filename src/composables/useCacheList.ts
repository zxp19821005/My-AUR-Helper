/**
 * useCacheList.ts - 缓存管理列表页面逻辑
 *
 * 功能：
 * - 管理列表分页、搜索、选择状态
 * - 从 cache_software 数据库表读取已存在的存量数据（页面打开自动执行）
 * - 扫描所有启用的缓存目录重新写入数据库（手动触发 "扫描" 按钮）
 *
 * 列表通用逻辑由 useListBase 提供。
 * 缓存条目因磁盘扫描结果 id 为占位值（跨页不唯一），故选择键采用「filteredEntries 全局索引」。
 */
import { computed, ref } from "vue";
import { useListBase } from "./useListBase";
import type { CacheSoftwareEntry } from "../types";
import * as cacheApi from "@/api/cache";

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
  /** 完整文件路径（cache_directory/filename），用于安装/查看信息 */
  full_path: string;
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
  const sourceDirFilter = ref("");
  const sourceDirs = ref<{ name: string; path: string }[]>([]);
  const archFilter = ref("");

  const base = useListBase<CacheListEntry>({
    pageSizeSetting: "list_page_size_cache",
    // 选择键 = filteredEntries 中的全局索引（缓存 id 为占位值，不可作为稳定键）
    getKey: (_e, globalIndex) => globalIndex,
    infoText: (t) => `总计: ${t} 个缓存文件`,
    pageResetRefs: [sourceDirFilter, archFilter],
    filter: (all, q) => {
      let result = all;
      if (sourceDirFilter.value) {
        result = result.filter((e) => e.cache_directory === sourceDirFilter.value);
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

  /** 把后端 CacheSoftwareEntry 转成列表条目 */
  function fromCacheSoftwareEntry(e: CacheSoftwareEntry): CacheListEntry {
    return {
      id: e.id,
      pkgname: e.pkgname || pkgFromFilename(e.filename),
      filename: e.filename,
      full_path: e.full_path,
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

  const architectures = computed(() => {
    const set = new Set<string>();
    for (const e of base.entries.value) if (e.arch) set.add(e.arch);
    return Array.from(set).sort();
  });

  /**
   * 从数据库 cache_software 表读取所有存量数据（页面打开时用）
   * 不会扫描磁盘，只执行 SELECT 查询，速度快
   */
  async function loadEntries() {
    base.loading.value = true;
    try {
      const data = await cacheApi.listCacheSoftware();
      base.entries.value = data.map(fromCacheSoftwareEntry);
      base.selectedIds.value = new Set();
    } finally {
      base.loading.value = false;
      base.syncToolbar();
    }
  }

  /**
   * 扫描所有启用的缓存目录（磁盘），清空并重建 cache_software 表
   * 仅在用户点击 "扫描" 按钮时触发
   *
   * 注意：scan_all_cache_dirs 的返回值（PkgFileInfo）不含缓存目录字段，
   * 且版本字段名为 version 与前端 CachePackage.pkgver 不匹配，因此扫描后
   * 直接重新从数据库读取（loadEntries），以展示完整的缓存目录与版本信息。
   * 这与备份管理页扫描后重新读库的行为保持一致。
   */
  async function rescanAllDirs() {
    base.loading.value = true;
    try {
      await cacheApi.scanAllCacheDirs();
      // 扫描结果已写入 cache_software 表，重新从数据库读取以展示完整字段
      await loadEntries();
    } finally {
      base.loading.value = false;
      base.syncToolbar();
    }
  }

  /** 兼容旧代码，保留 fetchEntries 指向 rescanAllDirs（按钮点击仍可用） */
  async function fetchEntries() {
    await rescanAllDirs();
  }

  return {
    ...base,
    sourceDirFilter,
    sourceDirs,
    archFilter,
    architectures,
    loadEntries,
    rescanAllDirs,
    fetchEntries,
  };
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
