/**
 * useBackupInstall.ts - 备份包安装逻辑
 *
 * 功能：
 * - 查看包信息（pacman -Qip）
 * - 安装备份包（sudo pacman -U）
 * - sudoers 配置检测与提示
 */
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export function useBackupInstall() {
  const installing = ref(false);
  const sudoersAvailable = ref<boolean | null>(null);
  const sudoersCommand = ref("");
  const showSudoersPrompt = ref(false);
  const pendingInstallPath = ref("");
  const pendingInstallPkgname = ref("");

  // 信息弹窗状态
  const infoDialogVisible = ref(false);
  const infoDialogLoading = ref(false);
  const infoDialogContent = ref("");
  const infoDialogPkgname = ref("");

  async function checkSudoers() {
    try {
      sudoersAvailable.value = await invoke<boolean>("check_sudoers_config");
    } catch {
      sudoersAvailable.value = false;
    }
  }

  async function loadSudoersCommand() {
    try {
      sudoersCommand.value = await invoke<string>("get_sudoers_command");
    } catch { /* ignore */ }
  }

  async function viewPackageInfo(fullPath: string, pkgname: string) {
    infoDialogPkgname.value = pkgname;
    infoDialogVisible.value = true;
    infoDialogLoading.value = true;
    infoDialogContent.value = "";
    try {
      const output = await invoke<string>("get_package_file_info", { fullPath });
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
  }

  async function handleInstall(fullPath: string, pkgname: string) {
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
      const output = await invoke<string>("install_backup_package", { fullPath });
      alert(`${pkgname} 安装成功！\n\n${output}`);
    } catch (e) {
      alert(`${pkgname} 安装失败: ${e}`);
    } finally {
      installing.value = false;
      showSudoersPrompt.value = false;
    }
  }

  function closeSudoersPrompt() {
    showSudoersPrompt.value = false;
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
    if (!confirm(`确定要安装选中的 ${selectedIds.size} 个备份包吗？`)) return;

    installing.value = true;
    let successCount = 0;
    let failCount = 0;
    const errors: string[] = [];

    for (const entry of entries) {
      if (!selectedIds.has(entry.id)) continue;
      try {
        await invoke<string>("install_backup_package", { fullPath: entry.full_path });
        successCount++;
      } catch (e) {
        failCount++;
        errors.push(`${entry.pkgname}: ${e}`);
      }
    }

    installing.value = false;
    const msg = `批量安装完成：成功 ${successCount} 个，失败 ${failCount} 个`;
    if (errors.length > 0) {
      alert(`${msg}\n\n错误:\n${errors.join("\n")}`);
    } else {
      alert(msg);
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
    checkSudoers,
    viewPackageInfo,
    closeInfoDialog,
    handleInstall,
    doInstall,
    closeSudoersPrompt,
    batchInstall,
  };
}
