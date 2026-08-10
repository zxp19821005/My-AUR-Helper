/**
 * useProxyActions.ts - 代理管理批量/单行操作逻辑
 *
 * 功能：
 * - 代理文件获取、解析、清空
 * - 批量/单行代理连通性测试
 * - 选中代理批量删除、单行删除
 *
 * 设计：复用主视图唯一的 useProxyList 实例（通过参数传入），
 * 避免二次调用 useProxyList 产生独立实例导致状态不同步；
 * 内部自行 inject FOOTER_KEY 进行消息提示。
 */
import { ref, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { FOOTER_KEY, addMessage } from "./footer";
import { useProxyList, type ProxyTestResult } from "./useProxyList";
import type { ProxyInfo } from "../types";

type ProxyListApi = ReturnType<typeof useProxyList>;

export function useProxyActions(list: ProxyListApi) {
  const footer = inject(FOOTER_KEY)!;

  const downloading = ref(false);
  const parsing = ref(false);
  const clearing = ref(false);
  const showClearConfirm = ref(false);

  /** 获取代理文件（从远程下载并写入本地） */
  async function handleDownloadProxyFile() {
    downloading.value = true;
    try {
      const count = await invoke<number>("download_proxy_file");
      addMessage(footer, "success", `成功下载代理文件，获取到 ${count} 个代理`);
      await list.fetchEntries();
    } catch (e) {
      addMessage(footer, "error", `下载失败: ${e}`);
    } finally {
      downloading.value = false;
    }
  }

  /** 解析代理文件（提取其中的代理条目入库） */
  async function handleParseProxyFile() {
    parsing.value = true;
    try {
      const count = await invoke<number>("parse_proxy_file");
      addMessage(footer, "success", `成功解析代理文件，新增 ${count} 个代理`);
      await list.fetchEntries();
    } catch (e) {
      addMessage(footer, "error", `解析失败: ${e}`);
    } finally {
      parsing.value = false;
    }
  }

  /** 清空代理表（proxies_info + proxies_test，重置 proxy_id） */
  async function handleClearProxyTables() {
    showClearConfirm.value = false;
    clearing.value = true;
    try {
      const count = await invoke<number>("clear_proxy_tables");
      addMessage(footer, "success", `已清空 ${count} 个代理记录，proxy_id 已重置`);
      list.selectedIds.value = new Set();
      await list.fetchEntries();
    } catch (e) {
      addMessage(footer, "error", `清空失败: ${e}`);
    } finally {
      clearing.value = false;
    }
  }

  /** 批量测试代理（未选中则测试全部） */
  async function handleTestProxies() {
    const proxyIds = list.selectedIds.value.size > 0
      ? Array.from(list.selectedIds.value)
      : [];

    if (proxyIds.length === 0) {
      addMessage(footer, "info", "开始测试所有代理...");
    } else {
      addMessage(footer, "info", `开始测试 ${proxyIds.length} 个选中代理...`);
    }

    try {
      const results = await invoke<ProxyTestResult[]>("test_proxies_batch", {
        proxyIds: proxyIds.length > 0 ? proxyIds : null,
      });

      for (const result of results) {
        list.setTestResult(result.proxy_id, result);
      }

      const successCount = results.filter((r) => r.success).length;
      const failCount = results.filter((r) => !r.success).length;
      addMessage(footer, "success", `测试完成: ${successCount} 个成功, ${failCount} 个失败`);
    } catch (e) {
      addMessage(footer, "error", `测试失败: ${e}`);
    }
  }

  /** 单行测试代理 */
  async function handleTestSingleProxy(proxyId: number) {
    list.testingIds.value.add(proxyId);
    list.testingIds.value = new Set(list.testingIds.value);

    try {
      const result = await invoke<ProxyTestResult>("test_proxy_single", { proxyId });
      list.setTestResult(proxyId, result);
      if (result.success) {
        addMessage(footer, "success", `代理测试成功: ${result.latency}ms`);
      } else {
        addMessage(footer, "warning", `代理测试失败: ${result.error}`);
      }
    } catch (e) {
      list.setTestResult(proxyId, {
        proxy_id: proxyId,
        success: false,
        latency: null,
        error: String(e),
        test_url: "",
      });
      addMessage(footer, "error", `测试失败: ${e}`);
    } finally {
      list.testingIds.value.delete(proxyId);
      list.testingIds.value = new Set(list.testingIds.value);
    }
  }

  /** 批量删除选中代理 */
  async function handleDeleteSelected() {
    if (list.selectedIds.value.size === 0) return;
    if (!confirm(`确定要删除选中的 ${list.selectedIds.value.size} 个代理吗？`)) return;
    list.loading.value = true;
    try {
      await list.deleteSelectedProxies();
      addMessage(footer, "success", `已删除 ${list.selectedIds.value.size} 个代理`);
      await list.fetchEntries();
    } catch (e) {
      addMessage(footer, "error", `删除失败: ${e}`);
    } finally {
      list.loading.value = false;
    }
  }

  /** 单行删除代理 */
  function handleDeleteSingleProxy(row: ProxyInfo) {
    if (row.proxy_id !== null) {
      list.selectedIds.value = new Set([row.proxy_id]);
      handleDeleteSelected();
    }
  }

  return {
    downloading,
    parsing,
    clearing,
    showClearConfirm,
    handleDownloadProxyFile,
    handleParseProxyFile,
    handleClearProxyTables,
    handleTestProxies,
    handleTestSingleProxy,
    handleDeleteSelected,
    handleDeleteSingleProxy,
  };
}
