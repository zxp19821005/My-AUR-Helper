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
import { invoke } from "@tauri-apps/api/core";
import { FOOTER_KEY, addMessage } from "./footer";

/**
 * 创建缓存清理操作集合
 *
 * @returns 缓存清理操作函数和状态
 */
export function useCacheCleanup() {
  const footer = inject(FOOTER_KEY)!;
  const loading = ref(false);
  const sudoersCommand = ref("");
  const showSudoersPrompt = ref(false);

  /**
   * 获取 sudoers 配置命令
   */
  async function loadSudoersCommand() {
    try {
      sudoersCommand.value = await invoke<string>("get_cache_cleanup_sudoers_command");
    } catch { /* ignore */ }
  }

  /**
   * 清理系统缓存 /var/cache/pacman/pkg
   */
  async function cleanSystemCache() {
    loading.value = true;
    try {
      const result = await invoke<string>("clean_system_cache");
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
      const result = await invoke<string>("clean_custom_cache_dirs");
      addMessage(footer, "success", result);
    } catch (e) {
      addMessage(footer, "error", `清理自定义缓存目录失败: ${e}`);
    } finally {
      loading.value = false;
    }
  }

  /**
   * 执行完整缓存清理（系统缓存 + 自定义缓存目录）
   */
  async function handleFullCleanup() {
    if (!confirm("确定要清理所有缓存吗？这将删除系统缓存和自定义缓存目录中的所有文件。")) {
      return;
    }

    loading.value = true;
    try {
      // 先清理系统缓存
      const systemResult = await invoke<string>("clean_system_cache");
      addMessage(footer, "success", systemResult);

      // 再清理自定义缓存目录
      const customResult = await invoke<string>("clean_custom_cache_dirs");
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
    }
  }

  /**
   * 关闭 sudoers 提示
   */
  function closeSudoersPrompt() {
    showSudoersPrompt.value = false;
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
