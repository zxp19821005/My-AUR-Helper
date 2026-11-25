/**
 * settings.ts - 设置与日志领域 API 封装
 *
 * 功能：
 * - 集中封装应用设置（get/set/list）与日志（读取/清空/应用日志设置）相关的 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 */

import { commands } from "@/bindings";
import type { Setting, LogEntry } from "@/types";

/** 按 key 读取单个设置，不存在返回 null */
export async function getSetting(key: string): Promise<Setting | null> {
  return await commands.getSetting(key);
}

/** 读取全部设置 */
export async function getSettings(): Promise<Setting[]> {
  return await commands.getSettings();
}

/** 写入单个设置 */
export async function setSetting(key: string, value: string): Promise<void> {
  await commands.setSetting(key, value);
}

/** 应用日志相关设置（maxSize 单位 bytes，maxFiles 为保留文件数） */
export async function applyLogSettings(maxSize: number, maxFiles: number): Promise<void> {
  await commands.applyLogSettings(maxSize, maxFiles);
}

/** 读取最近 limit 条日志 */
export async function getLogs(limit: number): Promise<LogEntry[]> {
  return await commands.getLogs(limit);
}

/** 清空日志 */
export async function clearLogs(): Promise<void> {
  await commands.clearLogs();
}
