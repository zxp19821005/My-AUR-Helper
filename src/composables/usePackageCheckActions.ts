/**
 * usePackageCheckActions.ts - 软件包版本检查操作逻辑
 *
 * 功能：
 * - 提供版本检查操作
 * - 支持批量检查和单行检查
 */
import { ref, inject } from "vue";
import { FOOTER_KEY } from "./footer";
import { formatErrorMessage } from "@/utils/errors";
import { showActionError } from "./useToast";
import * as softwareApi from "@/api/software";

export interface CheckActionsReturn {
  loading: import("vue").Ref<boolean>;
  setRowLoading: (pkgname: string, action: string) => void;
  clearRowLoading: (pkgname: string, action: string) => void;
  checkSelectedUpstream: (selectedPkgnames: Set<string>) => Promise<void>;
  rowCheckUpstream: (pkgname: string) => Promise<void>;
}

/**
 * 版本检查操作钩子
 * @param refreshEntries - 定向刷新指定软件包条目的回调函数
 * @param syncToolbar - 同步工具栏状态的回调函数
 * @param setRowLoading - 设置行级加载状态
 * @param clearRowLoading - 清除行级加载状态
 */
export function usePackageCheckActions(
  refreshEntries: (pkgnames: string[]) => Promise<void>,
  syncToolbar: () => void,
  setRowLoading: CheckActionsReturn['setRowLoading'],
  clearRowLoading: CheckActionsReturn['clearRowLoading'],
) {
  const footer = inject(FOOTER_KEY)!;
  /** 检查操作进行中状态（用于工具栏按钮 disabled） */
  const loading = ref(false);

  async function checkSelectedUpstream(selectedPkgnames: Set<string>) {
    loading.value = true;
    try {
      const list = Array.from(selectedPkgnames);
      if (list.length) {
        // 逐包检查：真实计数进度；检查涉及 GitHub/AUR 网络，期间界面保持可用
        let done = 0;
        for (const pkgname of list) {
          footer.progress = {
            current: done,
            total: list.length,
            message: `检查上游：${pkgname}`,
          };
          try {
            await softwareApi.checkSelectedUpstream([pkgname]);
            await refreshEntries([pkgname]);
          } catch (e) {
            showActionError(formatErrorMessage(e, `${pkgname} 上游检查`));
          }
          done++;
        }
        footer.progress = { current: done, total: list.length, message: "上游检查完成" };
      } else {
        footer.progress = { current: 0, total: 1, message: "正在全量检查上游（可能需要一些时间）…" };
        await softwareApi.checkAllUpstream();
        await refreshEntries([]);
        footer.progress = { current: 1, total: 1, message: "上游检查完成" };
      }
    } catch (e) {
      showActionError(formatErrorMessage(e, "上游检查"));
    } finally {
      footer.progress = null;
      loading.value = false;
      syncToolbar();
    }
  }

  async function rowCheckUpstream(pkgname: string) {
    loading.value = true;
    setRowLoading(pkgname, 'check-upstream');
    try {
      await softwareApi.checkSelectedUpstream([pkgname]);
      await refreshEntries([pkgname]);
    } catch (e) {
      showActionError(formatErrorMessage(e, `${pkgname} 上游检查`));
    } finally {
      loading.value = false;
      clearRowLoading(pkgname, 'check-upstream');
      syncToolbar();
    }
  }

  return {
    loading,
    checkSelectedUpstream,
    rowCheckUpstream,
  };
}
