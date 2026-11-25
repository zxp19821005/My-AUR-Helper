/**
 * usePackageSyncActions.ts - 软件包同步操作逻辑
 *
 * 功能：
 * - 提供包的同步操作（AUR 同步、PKGBUILD 同步、AUR 信息更新）
 * - 支持批量操作和单行操作
 * - 管理加载状态和进度反馈
 */
import { ref, inject, type Ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { FOOTER_KEY, addMessage } from "./footer";
import { formatErrorMessage } from "@/utils/errors";
import { showActionError } from "./useToast";
import * as softwareApi from "@/api/software";

/** 批量操作进度统计（由本模块更新，PackageList 顶部进度条消费） */
export interface PackageBatchStats {
  success: number;
  failed: number;
  total: number;
}

export interface SyncActionsReturn {
  loading: ReturnType<typeof ref<boolean>>;
  loadingKeys: ReturnType<typeof ref<Set<string>>>;
  isRowLoading: (pkgname: string, action?: string) => boolean;
  setRowLoading: (pkgname: string, action: string) => void;
  clearRowLoading: (pkgname: string, action: string) => void;
  syncFromAur: (selectedPkgnames: Set<string>) => Promise<void>;
  syncByMaintainer: () => Promise<void>;
  syncFromPkgbuild: (selectedPkgnames: Set<string>) => Promise<void>;
  updateAurInfo: (selectedPkgnames: Set<string>) => Promise<void>;
  rowSyncFromAur: (pkgname: string) => Promise<void>;
  rowSyncFromPkgbuild: (pkgname: string) => Promise<void>;
}

/**
 * 同步操作钩子
 * @param fetchView - 整表刷新列表的回调函数
 * @param refreshEntries - 定向刷新指定软件包条目的回调函数
 * @param syncToolbar - 同步工具栏状态的回调函数
 */
export function usePackageSyncActions(
  fetchView: () => Promise<void>,
  refreshEntries: (pkgnames: string[]) => Promise<void>,
  syncToolbar: () => void,
  batchStats: Ref<PackageBatchStats>
) {
  const footer = inject(FOOTER_KEY)!;

  // 同步操作进行中状态（仅用于禁用工具栏同类按钮，防止重复提交；
  // 界面其余部分保持可用——长任务进度走 footer 局部进度条，不设全屏遮罩）
  const loading = ref(false);
  // 按包名+操作类型追踪加载状态（用于行操作）
  const loadingKeys = ref(new Set<string>());
  let unlistenProgress: (() => void) | null = null;

  function isRowLoading(pkgname: string, action?: string): boolean {
    if (action) {
      return loadingKeys.value.has(`${pkgname}:${action}`);
    }
    return Array.from(loadingKeys.value).some(k => k.startsWith(`${pkgname}:`));
  }

  function setRowLoading(pkgname: string, action: string) {
    loadingKeys.value.add(`${pkgname}:${action}`);
  }

  function clearRowLoading(pkgname: string, action: string) {
    loadingKeys.value.delete(`${pkgname}:${action}`);
  }

  /** 清除进度条（不清除 stats，便于结果条短暂停留；由下一次操作或刷新覆盖） */
  function clearProgress() {
    footer.progress = null;
  }

  async function syncFromAur(selectedPkgnames: Set<string>) {
    loading.value = true;
    try {
      const list = Array.from(selectedPkgnames);
      if (list.length) {
        // 逐包同步：真实计数进度，界面保持可用
        batchStats.value = { success: 0, failed: 0, total: list.length };
        let done = 0;
        for (const pkgname of list) {
          footer.progress = {
            current: done,
            total: list.length,
            message: `AUR 同步：${pkgname}`,
          };
          try {
            await softwareApi.updateAurInfo([pkgname]);
            await refreshEntries([pkgname]);
            batchStats.value.success++;
          } catch (e) {
            showActionError(formatErrorMessage(e, `${pkgname} AUR同步`));
            batchStats.value.failed++;
          }
          done++;
        }
        batchStats.value.success = done - batchStats.value.failed;
        footer.progress = { current: done, total: list.length, message: "AUR 同步完成" };
      } else {
        // 全量同步：单条长 IPC，用不确定进度提示，避免用户误以为界面卡死
        footer.progress = { current: 0, total: 1, message: "正在从 AUR 全量同步（含归档下载）…" };
        await softwareApi.syncFromAur();
        await fetchView();
        footer.progress = { current: 1, total: 1, message: "AUR 同步完成" };
      }
    } catch (e) {
      showActionError(formatErrorMessage(e, "AUR同步"));
    } finally {
      clearProgress();
      loading.value = false;
      syncToolbar();
    }
  }

  /** 按配置的 AUR 维护者用户名批量同步（补齐缺失包 + 更新已有包元数据） */
  async function syncByMaintainer() {
    loading.value = true;
    // 按维护者同步是单条长 IPC（搜索 + 批量补全），用不确定进度提示
    footer.progress = { current: 0, total: 1, message: "正在按 AUR 维护者同步（搜索 + 补全）…" };
    try {
      const result = await softwareApi.syncPackagesByMaintainer();
      const missing = result.missing_detail.length
        ? `，${result.missing_detail.length} 个包未补全完整字段`
        : "";
      batchStats.value = {
        success: result.created + result.updated,
        failed: result.failed,
        total: result.aur_total,
      };
      footer.progress = { current: 1, total: 1, message: "AUR 维护者同步完成" };
      addMessage(
        footer,
        result.failed > 0 ? "warning" : "success",
        `AUR 维护者同步完成：共 ${result.aur_total} 个包，新建 ${result.created}，更新 ${result.updated}，失败 ${result.failed}${missing}`
      );
      await fetchView();
    } catch (e) {
      showActionError(formatErrorMessage(e, "AUR 维护者同步"));
    } finally {
      clearProgress();
      loading.value = false;
      syncToolbar();
    }
  }

  async function syncFromPkgbuild(selectedPkgnames: Set<string>) {
    loading.value = true;
    footer.progress = { current: 0, total: 1, message: "准备中..." };
    try {
      unlistenProgress = await listen<{
        current: number;
        total: number;
        pkgname: string;
        message: string;
      }>("sync-progress", (event) => {
        const { current, total, message } = event.payload;
        footer.progress = { current, total, message };
      });

      const list = Array.from(selectedPkgnames);
      if (list.length) {
        batchStats.value = { success: 0, failed: 0, total: list.length };
        for (const pkgname of list) {
          try {
            await softwareApi.syncFromPkgbuild(pkgname);
            batchStats.value.success++;
          } catch (e) {
            showActionError(formatErrorMessage(e, `${pkgname} PKGBUILD同步`));
            batchStats.value.failed++;
          }
        }
        await refreshEntries(list);
      } else {
        batchStats.value = { success: 0, failed: 0, total: 1 };
        await softwareApi.syncFromPkgbuild(null);
        batchStats.value.success = 1;
        await fetchView();
      }
    } catch (e) {
      showActionError(formatErrorMessage(e, "PKGBUILD同步"));
    } finally {
      unlistenProgress?.();
      unlistenProgress = null;
      footer.progress = null;
      loading.value = false;
      syncToolbar();
    }
  }

  async function updateAurInfo(selectedPkgnames: Set<string>) {
    loading.value = true;
    try {
      const list = Array.from(selectedPkgnames);
      if (list.length) {
        batchStats.value = { success: 0, failed: 0, total: list.length };
        let done = 0;
        for (const pkgname of list) {
          footer.progress = {
            current: done,
            total: list.length,
            message: `更新 AUR 信息：${pkgname}`,
          };
          try {
            await softwareApi.updateAurInfo([pkgname]);
            await refreshEntries([pkgname]);
            batchStats.value.success++;
          } catch (e) {
            showActionError(formatErrorMessage(e, `${pkgname} AUR信息更新`));
            batchStats.value.failed++;
          }
          done++;
        }
        footer.progress = { current: done, total: list.length, message: "AUR 信息更新完成" };
      } else {
        footer.progress = { current: 0, total: 1, message: "正在全量更新 AUR 信息（含归档下载）…" };
        const count = await softwareApi.updateAurInfo(null);
        batchStats.value = { success: count, failed: 0, total: count };
        footer.progress = { current: 1, total: 1, message: `AUR 信息更新完成，共 ${count} 个` };
        await fetchView();
      }
    } catch (e) {
      showActionError(formatErrorMessage(e, "AUR信息更新"));
    } finally {
      clearProgress();
      loading.value = false;
      syncToolbar();
    }
  }

  async function rowSyncFromAur(pkgname: string) {
    setRowLoading(pkgname, "sync-aur");
    try {
      await softwareApi.updateAurInfo([pkgname]);
      await refreshEntries([pkgname]);
    } catch (e) {
      showActionError(formatErrorMessage(e, `${pkgname} AUR同步`));
    } finally {
      clearRowLoading(pkgname, "sync-aur");
      syncToolbar();
    }
  }

  async function rowSyncFromPkgbuild(pkgname: string) {
    setRowLoading(pkgname, "sync-pkgbuild");
    try {
      await softwareApi.syncFromPkgbuild(pkgname);
      await refreshEntries([pkgname]);
    } catch (e) {
      showActionError(formatErrorMessage(e, `${pkgname} PKGBUILD同步`));
    } finally {
      clearRowLoading(pkgname, "sync-pkgbuild");
      syncToolbar();
    }
  }

  return {
    loading,
    loadingKeys,
    isRowLoading,
    setRowLoading,
    clearRowLoading,
    syncFromAur,
    syncByMaintainer,
    syncFromPkgbuild,
    updateAurInfo,
    rowSyncFromAur,
    rowSyncFromPkgbuild,
  };
}
