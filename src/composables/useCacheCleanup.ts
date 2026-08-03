/**
 * useCacheCleanup.ts - 缓存清理操作 composable
 *
 * 功能：
 * - cleanSystemCache: 清理系统缓存 /var/cache/pacman/pkg
 * - cleanCustomCacheDirs: 清理自定义 AUR 软件助手缓存目录
 * - checkSudoersConfig: 检测缓存清理 sudoers 配置
 * - getSudoersCommand: 获取缓存清理 sudoers 配置命令
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
  const sudoersAvailable = ref<boolean | null>(null);
  const sudoersCommand = ref("");
  const showSudoersPrompt = ref(false);

  /**
   * 检测 sudoers 配置是否可用
   */
  async function checkSudoersConfig() {
    try {
      sudoersAvailable.value = await invoke<boolean>("check_cache_cleanup_sudoers");
    } catch {
      sudoersAvailable.value = false;
    }
  }

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
    if (sudoersAvailable.value === false) {
      await loadSudoersCommand();
      showSudoersPrompt.value = true;
      return;
    }

    loading.value = true;
    try {
      const result = await invoke<string>("clean_system_cache");
      addMessage(footer, "success", result);
    } catch (e) {
      addMessage(footer, "error", `清理系统缓存失败: ${e}`);
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
    if (sudoersAvailable.value === false) {
      await loadSudoersCommand();
      showSudoersPrompt.value = true;
      return;
    }

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
      addMessage(footer, "error", `缓存清理失败: ${e}`);
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
    sudoersAvailable,
    sudoersCommand,
    showSudoersPrompt,
    checkSudoersConfig,
    cleanSystemCache,
    cleanCustomCacheDirs,
    handleFullCleanup,
    closeSudoersPrompt,
  };
}
