/**
 * license.ts - 许可证（License）领域 API 封装
 *
 * 功能：
 * - 集中封装 License 的增删改查与从 SPDX 同步等 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 */

import { invoke } from "@tauri-apps/api/core";
import type { License } from "@/types";

/** 列出全部 License */
export async function getLicenses(): Promise<License[]> {
  return await invoke<License[]>("get_licenses");
}

/** 新增 License */
export async function addLicense(spdxId: string, fullName: string): Promise<void> {
  await invoke("add_license", { spdxId, fullName });
}

/** 更新 License */
export async function updateLicense(id: number, spdxId: string, fullName: string): Promise<void> {
  await invoke("update_license", { id, spdxId, fullName });
}

/** 删除 License */
export async function deleteLicense(id: number): Promise<void> {
  await invoke("delete_license", { id });
}

/** 从 SPDX 同步 License 列表，返回同步条目数 */
export async function syncLicensesFromSpdx(): Promise<number> {
  return await invoke<number>("sync_licenses_from_spdx");
}
