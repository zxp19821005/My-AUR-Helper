/**
 * sudoers.ts - sudoers 免密配置领域 API 封装
 *
 * 功能：
 * - 集中封装备份安装 / 缓存安装 / 缓存清理三类场景的 sudoers 检测与命令获取
 * - 消除 useSudoers composable 中动态命令名调度，使命令字符串不再散落各处
 */

import { invoke } from "@tauri-apps/api/core";

/** 检测备份安装 sudoers 免密是否已配置 */
export async function checkBackupInstallSudoers(): Promise<boolean> {
  return await invoke<boolean>("check_sudoers_config");
}

/** 获取备份安装 sudoers 配置命令文本 */
export async function getBackupInstallSudoersCommand(): Promise<string> {
  return await invoke<string>("get_sudoers_command");
}

/** 检测缓存安装 sudoers 免密是否已配置 */
export async function checkCacheInstallSudoers(): Promise<boolean> {
  return await invoke<boolean>("check_cache_install_sudoers");
}

/** 获取缓存安装 sudoers 配置命令文本 */
export async function getCacheInstallSudoersCommand(): Promise<string> {
  return await invoke<string>("get_cache_install_sudoers_command");
}

/** 检测缓存清理 sudoers 免密是否已配置 */
export async function checkCacheCleanupSudoers(): Promise<boolean> {
  return await invoke<boolean>("check_cache_cleanup_sudoers");
}

/** 获取缓存清理 sudoers 配置命令文本 */
export async function getCacheCleanupSudoersCommand(): Promise<string> {
  return await invoke<string>("get_cache_cleanup_sudoers_command");
}
