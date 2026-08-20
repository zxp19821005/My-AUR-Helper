/**
 * useCacheDirs - 缓存目录通用 composable
 *
 * 功能：
 * - 加载所有缓存目录配置（系统缓存、paru 缓存、yay 缓存、自定义缓存）
 * - 保存自定义缓存目录
 * - 统一处理缓存目录的增删改查逻辑，避免在多个组件中重复
 */
import { ref } from "vue";
import { useSettingsStore } from "../stores/settings";
import * as settingsApi from "@/api/settings";

export interface CacheDir {
  name: string;
  path: string;
  is_enabled: boolean;
  is_default: boolean;
}

export interface CacheDirSimple {
  name: string;
  path: string;
}

/**
 * 加载所有缓存目录配置（用于 SettingsCacheSection 等需要完整配置的场景）
 *
 * 并行发起全部 get_setting IPC，避免串行往返；首个异常会中断整体加载
 *
 * @returns 包含完整信息的缓存目录列表（含 is_default、is_enabled）
 */
export async function loadCacheDirs(): Promise<CacheDir[]> {
  const dirs: CacheDir[] = [];

  // 7 个设置键并行读取（Promise.all 并发，显著快于逐个 await）
  const [systemPath, systemEnabled, paruPath, paruEnabled, yayPath, yayEnabled, customDirs] =
    await Promise.all([
      settingsApi.getSetting("cache_dir_system"),
      settingsApi.getSetting("cache_dir_system_enabled"),
      settingsApi.getSetting("cache_dir_paru"),
      settingsApi.getSetting("cache_dir_paru_enabled"),
      settingsApi.getSetting("cache_dir_yay"),
      settingsApi.getSetting("cache_dir_yay_enabled"),
      settingsApi.getSetting("cache_dirs_custom"),
    ]);

  // 系统缓存（默认）
  dirs.push({
    name: "系统缓存",
    path: systemPath?.value || "/var/cache/pacman/pkg",
    is_enabled: systemEnabled?.value !== "false",
    is_default: true,
  });

  // paru 缓存（默认）
  if (paruPath?.value) {
    dirs.push({
      name: "paru 缓存",
      path: paruPath.value,
      is_enabled: paruEnabled?.value !== "false",
      is_default: true,
    });
  }

  // yay 缓存（默认）
  if (yayPath?.value) {
    dirs.push({
      name: "yay 缓存",
      path: yayPath.value,
      is_enabled: yayEnabled?.value !== "false",
      is_default: true,
    });
  }

  // 自定义缓存目录（从 cache_dirs_custom 读取）
  if (customDirs?.value) {
    const customList: { name: string; path: string; is_enabled: boolean }[] =
      JSON.parse(customDirs.value);
    for (const dir of customList) {
      dirs.push({
        name: dir.name,
        path: dir.path,
        is_enabled: dir.is_enabled,
        is_default: false,
      });
    }
  }

  return dirs;
}

/**
 * 加载启用的缓存目录列表（用于 CacheManager 等只需要简单列表的场景）
 *
 * 并行发起全部 get_setting IPC，避免串行往返
 *
 * @returns 简化的缓存目录列表（仅包含已启用的 name 和 path）
 */
export async function loadEnabledCacheDirs(): Promise<CacheDirSimple[]> {
  const dirs: CacheDirSimple[] = [];

  const [systemDir, systemEnabled, paruDir, paruEnabled, yayDir, yayEnabled, customDirs] =
    await Promise.all([
      settingsApi.getSetting("cache_dir_system"),
      settingsApi.getSetting("cache_dir_system_enabled"),
      settingsApi.getSetting("cache_dir_paru"),
      settingsApi.getSetting("cache_dir_paru_enabled"),
      settingsApi.getSetting("cache_dir_yay"),
      settingsApi.getSetting("cache_dir_yay_enabled"),
      settingsApi.getSetting("cache_dirs_custom"),
    ]);

  // 系统缓存
  if (systemDir?.value && systemEnabled?.value !== "false") {
    dirs.push({ name: "系统缓存", path: systemDir.value });
  }

  // paru 缓存
  if (paruDir?.value && paruEnabled?.value !== "false") {
    dirs.push({ name: "paru 缓存", path: paruDir.value });
  }

  // yay 缓存
  if (yayDir?.value && yayEnabled?.value !== "false") {
    dirs.push({ name: "yay 缓存", path: yayDir.value });
  }

  // 自定义缓存目录
  if (customDirs?.value) {
    const customList: { name: string; path: string; is_enabled: boolean }[] =
      JSON.parse(customDirs.value);
    for (const dir of customList) {
      if (dir.path && dir.is_enabled) {
        dirs.push({ name: dir.name, path: dir.path });
      }
    }
  }

  return dirs;
}

/**
 * 保存自定义缓存目录列表
 *
 * 通过 settings store 的 setSetting 集中写入，同步更新 store 缓存
 *
 * @param dirs 完整的缓存目录列表（会自动过滤掉默认目录）
 */
export async function saveCustomCacheDirs(dirs: CacheDir[]): Promise<void> {
  const customDirs = dirs
    .filter((d) => !d.is_default)
    .map((d) => ({
      name: d.name,
      path: d.path,
      is_enabled: d.is_enabled,
    }));

  await useSettingsStore().setSetting("cache_dirs_custom", JSON.stringify(customDirs));
}

/**
 * 获取默认缓存目录的 settings key
 *
 * @param index 在完整列表中的索引
 * @param dirs 完整的缓存目录列表（用于计算默认目录的位置）
 * @returns settings key，如果不是默认目录则返回空字符串
 */
export function getDefaultCacheKey(index: number, dirs: CacheDir[]): string {
  const customDirsCount = dirs.slice(0, index).filter((d) => !d.is_default).length;
  const defaultIndex = index - customDirsCount;

  switch (defaultIndex) {
    case 0:
      return "cache_dir_system";
    case 1:
      return "cache_dir_paru";
    case 2:
      return "cache_dir_yay";
    default:
      return "";
  }
}

/**
 * useCacheDirs composable - 响应式封装
 *
 * 提供响应式的缓存目录状态和操作方法，适用于大多数场景。
 */
export function useCacheDirs() {
  const cacheDirs = ref<CacheDir[]>([]);
  const loading = ref(false);
  const message = ref("");

  async function load() {
    loading.value = true;
    try {
      cacheDirs.value = await loadCacheDirs();
    } catch (e) {
      message.value = "加载缓存目录失败: " + String(e);
    } finally {
      loading.value = false;
    }
  }

  async function saveCustom() {
    try {
      await saveCustomCacheDirs(cacheDirs.value);
    } catch (e) {
      throw e;
    }
  }

  function showMessage(msg: string, duration = 2000) {
    message.value = msg;
    setTimeout(() => (message.value = ""), duration);
  }

  return {
    cacheDirs,
    loading,
    message,
    load,
    saveCustom,
    showMessage,
  };
}
