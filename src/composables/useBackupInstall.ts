/**
 * useBackupInstall.ts - 备份包安装逻辑
 *
 * 功能：
 * - 查看包信息（pacman -Qip）
 * - 安装备份包（sudo pacman -U）
 * - sudoers 配置检测与提示
 */
import { ref, inject } from "vue";
import { FOOTER_KEY, addMessage } from "./footer";
import { openConfirm as confirm } from "./useConfirm";
import { useSudoers } from "./useSudoers";
import type { BackupSoftwareEntry } from "../types";
import * as backupApi from "@/api/backup";
import * as sudoersApi from "@/api/sudoers";

export function useBackupInstall() {
  const footer = inject(FOOTER_KEY)!;
  const installing = ref(false);
  const pendingInstallPath = ref("");
  const pendingInstallPkgname = ref("");

  // sudoers 免密配置状态与操作（检测/获取命令文本/弹窗控制）
  const {
    sudoersAvailable,
    sudoersCommand,
    showSudoersPrompt,
    checkSudoers,
    loadSudoersCommand,
    closeSudoersPrompt,
  } = useSudoers({
    checkFn: sudoersApi.checkBackupInstallSudoers,
    getCommandFn: sudoersApi.getBackupInstallSudoersCommand,
  });

  // 信息弹窗状态
  const infoDialogVisible = ref(false);
  const infoDialogLoading = ref(false);
  const infoDialogContent = ref("");
  const infoDialogPkgname = ref("");
  const infoDialogEntry = ref<BackupSoftwareEntry | null>(null);

  async function viewPackageInfo(entry: BackupSoftwareEntry) {
    infoDialogEntry.value = entry;
    infoDialogPkgname.value = entry.pkgname;
    infoDialogVisible.value = true;
    infoDialogLoading.value = true;
    infoDialogContent.value = "";
    try {
      const output = await backupApi.getPackageFileInfo(entry.full_path);
      infoDialogContent.value = output;
    } catch (e) {
      infoDialogContent.value = `获取信息失败: ${e}`;
    } finally {
      infoDialogLoading.value = false;
    }
  }

  function closeInfoDialog() {
    infoDialogVisible.value = false;
    infoDialogContent.value = "";
    infoDialogPkgname.value = "";
    infoDialogEntry.value = null;
  }

  async function handleInstall(fullPath: string, pkgname: string) {
    // 防重入：安装进行中再次触发（如快速连点、详情弹窗与行操作同时触发）
    // 直接忽略，避免同一包被并发安装、争抢 pacman 数据库锁
    if (installing.value) return;
    pendingInstallPath.value = fullPath;
    pendingInstallPkgname.value = pkgname;

    if (sudoersAvailable.value === false) {
      await loadSudoersCommand();
      showSudoersPrompt.value = true;
      return;
    }

    doInstall(fullPath, pkgname);
  }

  async function doInstall(fullPath: string, pkgname: string) {
    installing.value = true;
    try {
      await backupApi.installBackupPackage(fullPath);
      addMessage(footer, "success", `${pkgname} 安装成功`);
      // 成功：关闭 sudoers 提示弹窗
      showSudoersPrompt.value = false;
    } catch (e) {
      addMessage(footer, "error", `${pkgname} 安装失败: ${e}`);
      // 失败：保留弹窗，按钮恢复可点击（由 installing=false 驱动）
    } finally {
      installing.value = false;
    }
  }

  async function batchInstall(
    selectedIds: Set<number>,
    entries: { id: number; pkgname: string; full_path: string }[],
  ) {
    if (selectedIds.size === 0) return;
    if (sudoersAvailable.value === false) {
      await loadSudoersCommand();
      showSudoersPrompt.value = true;
      return;
    }
    if (!(await confirm({ message: `确定要安装选中的 ${selectedIds.size} 个备份包吗？` }))) return;

    installing.value = true;
    let successCount = 0;
    let failCount = 0;
    const errors: string[] = [];

    for (const entry of entries) {
      if (!selectedIds.has(entry.id)) continue;
      try {
        await backupApi.installBackupPackage(entry.full_path);
        successCount++;
      } catch (e) {
        failCount++;
        errors.push(`${entry.pkgname}: ${e}`);
      }
    }

    installing.value = false;
    const msg = `批量安装完成：成功 ${successCount} 个，失败 ${failCount} 个`;
    if (errors.length > 0) {
      addMessage(footer, "warning", `${msg}，错误: ${errors.join("; ")}`);
    } else {
      addMessage(footer, "success", msg);
    }
  }

  return {
    installing,
    sudoersAvailable,
    sudoersCommand,
    showSudoersPrompt,
    pendingInstallPath,
    pendingInstallPkgname,
    infoDialogVisible,
    infoDialogLoading,
    infoDialogContent,
    infoDialogPkgname,
    infoDialogEntry,
    checkSudoers,
    viewPackageInfo,
    closeInfoDialog,
    handleInstall,
    doInstall,
    closeSudoersPrompt,
    batchInstall,
  };
}
