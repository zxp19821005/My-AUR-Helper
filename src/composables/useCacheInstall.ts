/**
 * useCacheInstall.ts - 缓存包安装逻辑
 *
 * 功能：
 * - 查看缓存包信息（pacman -Qip）
 * - 安装缓存包（sudo pacman -U）
 * - sudoers 免密复用缓存清理规则（已包含 pacman -U <cache_dir>/*）
 *
 * 与 useBackupInstall 保持一致的对外接口，便于在缓存管理页复用信息弹窗与
 * sudoers 提示弹窗。
 */
import { ref, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FOOTER_KEY, addMessage } from "./footer";

export function useCacheInstall() {
  const footer = inject(FOOTER_KEY)!;
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
  const infoDialogEntry = ref<any>(null);

  /** 检测缓存安装 sudoers 是否已配置（允许 pacman -U 作用于缓存目录） */
  async function checkSudoers() {
    try {
      sudoersAvailable.value = await invoke<boolean>("check_cache_install_sudoers");
    } catch {
      sudoersAvailable.value = false;
    }
  }

  async function loadSudoersCommand() {
    try {
      sudoersCommand.value = await invoke<string>("get_cache_install_sudoers_command");
    } catch { /* ignore */ }
  }

  /** 由列表行解析出完整文件路径（兜底：full_path 缺失时用 缓存目录+文件名 拼接） */
  function resolveFullPath(entry: any): string {
    if (entry.full_path) return entry.full_path;
    return [entry.cache_directory, entry.filename].filter(Boolean).join("/");
  }

  /** 打开缓存包信息弹窗并加载 pacman -Qip 输出 */
  async function viewPackageInfo(entry: any) {
    const fullPath = resolveFullPath(entry);
    infoDialogEntry.value = entry;
    infoDialogPkgname.value = entry.pkgname;
    infoDialogVisible.value = true;
    infoDialogLoading.value = true;
    infoDialogContent.value = "";
    try {
      const output = await invoke<string>("get_cache_package_info", { fullPath });
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

  /** 发起安装：未配置 sudoers 时先提示配置命令 */
  async function handleInstall(fullPath: string, pkgname: string) {
    // 防重入：安装进行中再次触发直接忽略，避免同一包并发安装、争抢 pacman 数据库锁
    if (installing.value) return;
    pendingInstallPath.value = fullPath;
    pendingInstallPkgname.value = pkgname;

    if (sudoersAvailable.value !== true) {
      await loadSudoersCommand();
      showSudoersPrompt.value = true;
      return;
    }

    doInstall(fullPath, pkgname);
  }

  async function doInstall(fullPath: string, pkgname: string) {
    installing.value = true;
    try {
      await invoke<string>("install_cache_package", { fullPath });
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

  function closeSudoersPrompt() {
    showSudoersPrompt.value = false;
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
    resolveFullPath,
    closeInfoDialog,
    handleInstall,
    doInstall,
    closeSudoersPrompt,
  };
}
