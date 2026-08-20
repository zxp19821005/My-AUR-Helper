/**
 * useCacheCleanup.ts - 缓存清理操作 composable
 *
 * 功能：
 * - cleanSystemCache: 清理系统缓存 /var/cache/pacman/pkg
 * - cleanCustomCacheDirs: 清理自定义 AUR 软件助手缓存目录
 * - handleFullCleanup: 执行完整缓存清理
 *
 * 使用场景：CacheManager.vue 页面的缓存清理按钮
 */
import { ref, inject } from "vue";
import { FOOTER_KEY, addMessage } from "./footer";
import { openConfirm as confirm } from "./useConfirm";
import { useSudoers } from "./useSudoers";
import * as cacheApi from "@/api/cache";
import * as sudoersApi from "@/api/sudoers";

/**
 * 创建缓存清理操作集合
 *
 * @returns 缓存清理操作函数和状态
 */
export function useCacheCleanup() {
  const footer = inject(FOOTER_KEY)!;
  const loading = ref(false);

  // sudoers 免密配置状态与操作（清理系统缓存需要 root 权限）
  const {
    sudoersCommand,
    showSudoersPrompt,
    loadSudoersCommand,
    closeSudoersPrompt,
  } = useSudoers({
    checkFn: sudoersApi.checkCacheCleanupSudoers,
    getCommandFn: sudoersApi.getCacheCleanupSudoersCommand,
  });

  /**
   * 清理系统缓存 /var/cache/pacman/pkg
   */
  async function cleanSystemCache() {
    loading.value = true;
    try {
      const result = await cacheApi.cleanSystemCache();
      addMessage(footer, "success", result);
    } catch (e) {
      const errorStr = String(e);
      // 如果是权限错误，显示 sudoers 配置提示
      if (errorStr.includes("权限不够") || errorStr.includes("permission denied") || errorStr.includes("sudo")) {
        await loadSudoersCommand();
        showSudoersPrompt.value = true;
      } else {
        addMessage(footer, "error", `清理系统缓存失败: ${e}`);
      }
    } finally {
      loading.value = false;
    }
  }

  /**
   * 清理自定义 AUR 软件助手缓存目录
   */
  async function cleanCustomCacheDirs() {
    loading.value = true;
    try {
      const result = await cacheApi.cleanCustomCacheDirs();
      addMessage(footer, "success", result);
    } catch (e) {
      addMessage(footer, "error", `清理自定义缓存目录失败: ${e}`);
    } finally {
      loading.value = false;
    }
  }

  /**
   * 执行完整缓存清理（系统缓存 + 自定义缓存目录）
   *
   * @param onComplete 清理完成（无论成功或失败）后执行的回调，用于刷新列表。
   *                   清理会删除磁盘上的缓存文件，但 cache_software 表仍为旧数据，
   *                   因此调用方应传入重新扫描函数（rescanAllDirs）以反映实际磁盘状态。
   */
  async function handleFullCleanup(onComplete?: () => Promise<void>) {
    if (!(await confirm({ message: "确定要清理所有缓存吗？这将删除系统缓存和自定义缓存目录中的所有文件。", variant: "danger" }))) {
      return;
    }

    loading.value = true;
    try {
      // 先清理系统缓存
      const systemResult = await cacheApi.cleanSystemCache();
      addMessage(footer, "success", systemResult);

      // 再清理自定义缓存目录
      const customResult = await cacheApi.cleanCustomCacheDirs();
      addMessage(footer, "success", customResult);
    } catch (e) {
      const errorStr = String(e);
      // 如果是权限错误，显示 sudoers 配置提示
      if (errorStr.includes("权限不够") || errorStr.includes("permission denied") || errorStr.includes("sudo")) {
        await loadSudoersCommand();
        showSudoersPrompt.value = true;
      } else {
        addMessage(footer, "error", `缓存清理失败: ${e}`);
      }
    } finally {
      loading.value = false;
      // 重新扫描磁盘，使列表与已删除的缓存文件保持一致
      if (onComplete) {
        try {
          await onComplete();
        } catch (e) {
          addMessage(footer, "error", `清理后刷新缓存列表失败: ${e}`);
        }
      }
    }
  }

  return {
    loading,
    sudoersCommand,
    showSudoersPrompt,
    cleanSystemCache,
    cleanCustomCacheDirs,
    handleFullCleanup,
    closeSudoersPrompt,
  };
}
