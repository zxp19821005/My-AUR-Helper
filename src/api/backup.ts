/**
 * backup.ts - 备份领域 API 封装
 *
 * 功能：
 * - 集中封装备份软件列表、包信息查询/安装、备份删除、目录扫描/去重等 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 */

import { commands } from "@/bindings";
import type { BackupSoftwareEntry, DeduplicateResult } from "@/types";

/** 列出全部备份软件 */
export async function listBackupSoftware(): Promise<BackupSoftwareEntry[]> {
  return await commands.listBackupSoftware();
}

/** 查询备份包文件信息（pacman -Qip 输出） */
export async function getPackageFileInfo(fullPath: string): Promise<string> {
  return await commands.getPackageFileInfo(fullPath);
}

/** 安装备份包，返回安装输出 */
export async function installBackupPackage(fullPath: string): Promise<string> {
  return await commands.installBackupPackage(fullPath);
}

/** 删除备份（按记录 id 与备份路径） */
export async function deleteBackup(id: number, backupPath: string): Promise<void> {
  await commands.deleteBackup(id, backupPath);
}

/** 扫描备份目录，返回扫描条目数 */
export async function scanBackupDirectory(backupPath: string): Promise<number> {
  return await commands.scanBackupDirectory(backupPath);
}

/** 列出备份子目录 */
export async function listBackupSubdirectories(): Promise<string[]> {
  return await commands.listBackupSubdirectories();
}

/** 清空备份软件表，返回清空条目数 */
export async function clearBackupSoftware(): Promise<number> {
  return await commands.clearBackupSoftware();
}

/** 对备份目录去重，返回去重结果 */
export async function deduplicateBackups(backupPath: string): Promise<DeduplicateResult> {
  return await commands.deduplicateBackups(backupPath);
}
