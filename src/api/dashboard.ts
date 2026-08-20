/**
 * dashboard.ts - 仪表盘领域 API 封装
 *
 * 功能：
 * - 集中封装仪表盘统计聚合查询的 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 */

import { invoke } from "@tauri-apps/api/core";
import type { DashboardStats } from "@/types";

/** 获取仪表盘统计（各模块计数汇总） */
export async function getDashboardStats(): Promise<DashboardStats> {
  return await invoke<DashboardStats>("get_dashboard_stats");
}
