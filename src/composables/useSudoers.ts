/**
 * useSudoers.ts - sudoers 免密配置共享 composable
 *
 * 功能：
 * - 统一管理三类场景（备份安装 / 缓存安装 / 缓存清理）的 sudoers 状态与操作：
 *   sudoersAvailable 检测、sudoersCommand 配置命令文本、showSudoersPrompt 提示弹窗控制
 * - 调用方注入 api 模块的具体检测/获取函数，消除 useBackupInstall / useCacheInstall /
 *   useCacheCleanup 三个 composable 中的重复实现，同时使命令字符串不再散落各处
 *
 * 使用场景：备份管理、缓存管理、设置页等需要 sudoers 免密配置提示的页面
 */
import { ref } from "vue";

export interface SudoersOptions {
  /** 检测 sudoers 是否已配置的 api 函数 */
  checkFn: () => Promise<boolean>;
  /** 获取 sudoers 配置命令文本的 api 函数 */
  getCommandFn: () => Promise<string>;
}

/**
 * 创建 sudoers 免密配置状态与操作集合
 *
 * @param options 注入 api 模块的检测/获取函数（不同场景对应不同 api 函数）
 * @returns sudoers 相关响应式状态与操作函数
 */
export function useSudoers(options: SudoersOptions) {
  /** 是否已配置 sudoers：true=免密可用 / null=尚未检测 / false=未配置 */
  const sudoersAvailable = ref<boolean | null>(null);
  /** sudoers 配置命令文本（弹窗中展示给用户复制执行） */
  const sudoersCommand = ref("");
  /** 是否显示 sudoers 配置提示弹窗 */
  const showSudoersPrompt = ref(false);

  /**
   * 检测 sudoers 免密配置是否可用
   * 后端调用失败时视为未配置（与历史行为一致），并输出诊断日志
   */
  async function checkSudoers() {
    try {
      sudoersAvailable.value = await options.checkFn();
    } catch (e) {
      console.error(`检查 sudoers 配置失败:`, e);
      sudoersAvailable.value = false;
    }
  }

  /**
   * 加载 sudoers 配置命令文本（用于提示弹窗展示）
   * 失败仅记录诊断日志，不阻断弹窗流程
   */
  async function loadSudoersCommand() {
    try {
      sudoersCommand.value = await options.getCommandFn();
    } catch (e) {
      console.error(`加载 sudoers 配置命令失败:`, e);
    }
  }

  /** 关闭 sudoers 提示弹窗 */
  function closeSudoersPrompt() {
    showSudoersPrompt.value = false;
  }

  return {
    sudoersAvailable,
    sudoersCommand,
    showSudoersPrompt,
    checkSudoers,
    loadSudoersCommand,
    closeSudoersPrompt,
  };
}
