/**
 * useCacheManagerInit.ts - 缓存管理页初始化逻辑
 *
 * 功能：
 * - 聚合缓存管理页 onMounted 阶段的 5 项初始化任务：
 *   加载缓存数据 / 加载缓存目录 / 读取 backup_dir 设置 / 加载备份子目录 / 检查 sudoers 配置
 * - 集中错误处理，避免在组件中散落空 catch 吞掉错误（至少保留 console.error 诊断信息）
 *
 * @param footer 底部工具栏状态（用于展示关键错误消息）
 * @param deps 依赖注入：loadEntries（加载缓存表）、setSourceDirs（设置缓存目录下拉）、checkSudoers（sudoers 检查）
 * @returns 初始化后共享给组件的响应式状态
 */
import { ref, onMounted } from "vue";
import { addMessage } from "./footer";
import { loadEnabledCacheDirs } from "./useCacheDirs";
import type { FooterState } from "../types";
import * as settingsApi from "@/api/settings";
import * as backupApi from "@/api/backup";

export interface CacheManagerInitDeps {
  /** 从 cache_software 表读取存量数据（关键任务，失败需提示用户） */
  loadEntries: () => Promise<void>;
  /** 将启用的缓存目录写入 sourceDirs 下拉选项 */
  setSourceDirs: (dirs: { name: string; path: string }[]) => void;
  /** 检查 sudoers 免密配置 */
  checkSudoers: () => Promise<void>;
}

export function useCacheManagerInit(
  footer: FooterState,
  deps: CacheManagerInitDeps,
) {
  const backupPath = ref("");
  const backupSubdirectories = ref<string[]>([]);

  onMounted(async () => {
    // 首次进入即从 cache_software 表读取存量数据（与备份管理页一致）
    try {
      await deps.loadEntries();
    } catch (e) {
      console.error("加载缓存数据失败:", e);
      addMessage(footer, "error", `加载缓存数据失败: ${e}`);
    }
    try {
      deps.setSourceDirs(await loadEnabledCacheDirs());
    } catch (e) {
      console.error("加载缓存目录失败:", e);
    }
    try {
      const setting = await settingsApi.getSetting("backup_dir");
      if (setting) backupPath.value = setting.value;
    } catch (e) {
      console.error("加载备份目录设置失败:", e);
    }
    try {
      backupSubdirectories.value = await backupApi.listBackupSubdirectories();
    } catch (e) {
      console.error("加载备份子目录失败:", e);
    }
    try {
      await deps.checkSudoers();
    } catch (e) {
      console.error("检查 sudoers 配置失败:", e);
    }
  });

  return { backupPath, backupSubdirectories };
}
