/**
 * usePackageActions.ts - 软件包操作逻辑
 *
 * 提供包管理的同步、检查、删除等操作
 */
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { FooterState } from "../types";

export function usePackageActions(
  fetchView: () => Promise<void>,
  footer: FooterState
) {
  // 全局加载状态（用于工具栏批量操作）
  const loading = ref(false);
  // 按包名+操作类型追踪加载状态（用于行操作）
  const loadingKeys = ref(new Set<string>());
  let unlistenProgress: (() => void) | null = null;

  function isRowLoading(pkgname: string, action?: string): boolean {
    if (action) {
      return loadingKeys.value.has(`${pkgname}:${action}`);
    }
    // 检查该包是否有任何操作在进行
    return Array.from(loadingKeys.value).some(k => k.startsWith(`${pkgname}:`));
  }

  function setRowLoading(pkgname: string, action: string) {
    loadingKeys.value.add(`${pkgname}:${action}`);
  }

  function clearRowLoading(pkgname: string, action: string) {
    loadingKeys.value.delete(`${pkgname}:${action}`);
  }

  async function syncFromAur(selectedPkgnames: Set<string>) {
    loading.value = true;
    try {
      const list = Array.from(selectedPkgnames);
      if (list.length) {
        await invoke("update_aur_info", { pkgnameList: list });
      } else {
        await invoke("sync_from_aur");
      }
      await fetchView();
    } finally {
      loading.value = false;
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
        for (const pkgname of list) {
          await invoke("sync_from_pkgbuild", { pkgname });
        }
      } else {
        await invoke("sync_from_pkgbuild", { pkgname: null });
      }
      await fetchView();
    } finally {
      unlistenProgress?.();
      unlistenProgress = null;
      footer.progress = null;
      loading.value = false;
    }
  }

  async function updateAurInfo(selectedPkgnames: Set<string>) {
    loading.value = true;
    try {
      const list = Array.from(selectedPkgnames);
      await invoke("update_aur_info", { pkgnameList: list.length ? list : null });
      await fetchView();
    } finally {
      loading.value = false;
    }
  }

  async function checkSelectedUpstream(selectedPkgnames: Set<string>) {
    loading.value = true;
    try {
      const list = Array.from(selectedPkgnames);
      if (list.length) {
        await invoke("check_selected_upstream", { pkgnameList: list });
      } else {
        await invoke("check_all_upstream");
      }
      await fetchView();
    } finally {
      loading.value = false;
    }
  }

  async function deleteSelected(
    selectedPkgnames: Set<string>,
    setSelectedPkgnames: (v: Set<string>) => void
  ) {
    const list = Array.from(selectedPkgnames);
    if (!list.length) return;
    if (!confirm(`确认删除选中的 ${list.length} 个软件包？`)) return;
    loading.value = true;
    try {
      await invoke("batch_delete_software", { pkgnameList: list });
      setSelectedPkgnames(new Set());
      await fetchView();
    } finally {
      loading.value = false;
    }
  }

  async function checkAll() {
    loading.value = true;
    footer.progress = { current: 0, total: 1 };
    try {
      await invoke("check_all_upstream");
      await fetchView();
    } finally {
      loading.value = false;
      footer.progress = null;
    }
  }

  async function rowSyncFromAur(pkgname: string) {
    setRowLoading(pkgname, "sync-aur");
    try {
      await invoke("update_aur_info", { pkgnameList: [pkgname] });
      await fetchView();
    } finally {
      clearRowLoading(pkgname, "sync-aur");
    }
  }

  async function rowSyncFromPkgbuild(pkgname: string) {
    setRowLoading(pkgname, "sync-pkgbuild");
    try {
      await invoke("sync_from_pkgbuild", { pkgname });
      await fetchView();
    } finally {
      clearRowLoading(pkgname, "sync-pkgbuild");
    }
  }

  async function rowCheckUpstream(pkgname: string) {
    setRowLoading(pkgname, "check-upstream");
    try {
      await invoke("check_selected_upstream", { pkgnameList: [pkgname] });
      await fetchView();
    } finally {
      clearRowLoading(pkgname, "check-upstream");
    }
  }

  async function rowDelete(
    pkgname: string,
    selectedPkgnames: Set<string>,
    setSelectedPkgnames: (v: Set<string>) => void
  ) {
    if (!confirm(`确认删除 ${pkgname}？`)) return;
    setRowLoading(pkgname, "delete");
    try {
      await invoke("batch_delete_software", { pkgnameList: [pkgname] });
      setSelectedPkgnames(
        new Set(Array.from(selectedPkgnames).filter((n) => n !== pkgname))
      );
      await fetchView();
    } finally {
      clearRowLoading(pkgname, "delete");
    }
  }

  return {
    loading,
    loadingKeys,
    isRowLoading,
    setRowLoading,
    clearRowLoading,
    syncFromAur,
    syncFromPkgbuild,
    updateAurInfo,
    checkSelectedUpstream,
    deleteSelected,
    checkAll,
    rowSyncFromAur,
    rowSyncFromPkgbuild,
    rowCheckUpstream,
    rowDelete,
  };
}