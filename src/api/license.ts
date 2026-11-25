/**
 * license.ts - 许可证（License）领域 API 封装
 *
 * 功能：
 * - 集中封装 License 的增删改查与从 SPDX 同步等 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 */

import { commands } from "@/bindings";
import type { License } from "@/types";

/** 列出全部 License */
export async function getLicenses(): Promise<License[]> {
  return await commands.getLicenses();
}

/** 新增 License，返回新记录 ID */
export async function addLicense(spdxId: string, fullName: string): Promise<number> {
  return await commands.addLicense(spdxId, fullName);
}

/** 更新 License */
export async function updateLicense(id: number, spdxId: string, fullName: string): Promise<void> {
  await commands.updateLicense(id, spdxId, fullName);
}

/** 删除 License */
export async function deleteLicense(id: number): Promise<void> {
  await commands.deleteLicense(id);
}

/** 从 SPDX 同步 License 列表，返回同步条目数 */
export async function syncLicensesFromSpdx(): Promise<number> {
  return await commands.syncLicensesFromSpdx();
}
