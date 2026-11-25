/**
 * update.ts - 软件更新模块 IPC 封装
 *
 * 功能：集中封装软件更新工作台涉及的 Tauri 命令，页面只从本文件调用，
 *      不直接写 invoke，与 src/api/ 其它领域文件保持一致。
 *
 * 调用链：listSoftwareUpdates → previewSoftwareUpdates（dry-run diff）
 *        → applySoftwareUpdates（用户确认后才执行写回）。
 */
import { commands } from "@/bindings";
import type {
  SoftwareUpdateApplyResult,
  SoftwareUpdateItem,
  SoftwareUpdatePreview,
} from "@/types";

/** 拉取全部软件包的更新状态（只读，含本地 PKGBUILD 实际版本） */
export async function listSoftwareUpdates(): Promise<SoftwareUpdateItem[]> {
  return await commands.listSoftwareUpdates();
}

/** 预览一批包的 PKGBUILD 变更（dry-run，绝不写文件） */
export async function previewSoftwareUpdates(
  pkgnames: string[]
): Promise<SoftwareUpdatePreview[]> {
  return await commands.previewSoftwareUpdates(pkgnames);
}

/** 执行一批包的 PKGBUILD 写回；调用前必须先展示预览并取得用户确认 */
export async function applySoftwareUpdates(
  pkgnames: string[]
): Promise<SoftwareUpdateApplyResult[]> {
  return await commands.applySoftwareUpdates(pkgnames);
}
