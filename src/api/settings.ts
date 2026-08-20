/**
 * settings.ts - 设置与日志领域 API 封装
 *
 * 功能：
 * - 集中封装应用设置（get/set/list）与日志（读取/清空/应用日志设置）相关的 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 */

import { invoke } from "@tauri-apps/api/core";
import type { Setting, LogEntry } from "@/types";

/** 按 key 读取单个设置，不存在返回 null */
export async function getSetting(key: string): Promise<Setting | null> {
  return await invoke<Setting | null>("get_setting", { key });
}

/** 读取全部设置 */
export async function getSettings(): Promise<Setting[]> {
  return await invoke<Setting[]>("get_settings");
}

/** 写入单个设置 */
export async function setSetting(key: string, value: string): Promise<void> {
  await invoke("set_setting", { key, value });
}

/** 应用日志相关设置 */
export async function applyLogSettings(): Promise<void> {
  await invoke("apply_log_settings");
}

/** 读取最近 limit 条日志 */
export async function getLogs(limit: number): Promise<LogEntry[]> {
  return await invoke<LogEntry[]>("get_logs", { limit });
}

/** 清空日志 */
export async function clearLogs(): Promise<void> {
  await invoke("clear_logs");
}
