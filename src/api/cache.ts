/**
 * cache.ts - 缓存领域 API 封装
 *
 * 功能：
 * - 集中封装缓存软件列表、目录扫描、包信息查询/安装、缓存清理、缓存备份等 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 */

import { invoke } from "@tauri-apps/api/core";
import type { CacheSoftwareEntry, MemoryCacheStats } from "@/types";

/** 清空缓存软件表，返回清空条目数 */
export async function clearCacheSoftware(): Promise<number> {
  return await invoke<number>("clear_cache_software");
}

/** 列出全部缓存软件 */
export async function listCacheSoftware(): Promise<CacheSoftwareEntry[]> {
  return await invoke<CacheSoftwareEntry[]>("list_cache_software");
}

/** 扫描全部缓存目录，返回扫描条目 */
export async function scanAllCacheDirs(): Promise<CacheSoftwareEntry[]> {
  return await invoke<CacheSoftwareEntry[]>("scan_all_cache_dirs");
}

/** 查询缓存包文件信息（pacman -Qip 输出） */
export async function getCachePackageInfo(fullPath: string): Promise<string> {
  return await invoke<string>("get_cache_package_info", { fullPath });
}

/** 安装缓存包，返回安装输出 */
export async function installCachePackage(fullPath: string): Promise<string> {
  return await invoke<string>("install_cache_package", { fullPath });
}

/** 清理系统缓存，返回清理输出 */
export async function cleanSystemCache(): Promise<string> {
  return await invoke<string>("clean_system_cache");
}

/** 清理自定义缓存目录，返回清理输出 */
export async function cleanCustomCacheDirs(): Promise<string> {
  return await invoke<string>("clean_custom_cache_dirs");
}

/** 备份缓存包到指定子目录，返回 [新增条目数, 跳过文件名列表] */
export async function backupCacheToSubdirectory(
  filenames: string[],
  backupPath: string,
  subdirectory: string,
): Promise<[number, string[]]> {
  return await invoke<[number, string[]]>("backup_cache_to_subdirectory", {
    filenames,
    backupPath,
    subdirectory,
  });
}

/** 自动比较版本，将更新的缓存包备份到已有备份目录，返回 [成功数, 错误列表] */
export async function backupCacheToExisting(
  backupPath: string,
): Promise<[number, string[]]> {
  return await invoke<[number, string[]]>("backup_cache_to_existing", {
    backupPath,
  });
}

// ===================== 内存缓存管理 =====================

/** 获取内存缓存运行状态（配置 + 各域状态） */
export async function getMemoryCacheStats(): Promise<MemoryCacheStats> {
  return await invoke<MemoryCacheStats>("get_memory_cache_stats");
}

/** 立即将脏缓存写盘，返回写入的缓存域数量 */
export async function flushMemoryCache(): Promise<number> {
  return await invoke<number>("flush_memory_cache");
}

/** 清空内存缓存与磁盘缓存文件 */
export async function clearMemoryCache(): Promise<void> {
  await invoke("clear_memory_cache");
}
