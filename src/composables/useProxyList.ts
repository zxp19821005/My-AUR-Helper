/**
 * useProxyList.ts - 代理管理列表页面逻辑
 *
 * 功能：
 * - 管理列表分页、搜索、选择状态
 * - 提供格式化函数和操作控制逻辑
 * - 支持代理类型筛选
 */
import { computed, ref, watch, inject, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../stores/settings";
import { FOOTER_KEY } from "./footer";
import type { ProxyInfo, ProxyType } from "../types";

/** 从 URL 提取代理显示名称（域名部分） */
export function extractProxyName(url: string): string {
  try {
    if (url.includes("://")) {
      const domain = url.split("://")[1]?.split("/")[0];
      if (domain) return domain;
    }
  } catch { /* ignore */ }
  return url;
}

/** 判断代理名称是否有效（非空、非 SVG/数组垃圾数据） */
export function isValidProxyName(name: string): boolean {
  if (!name || name.trim().length === 0) return false;
  if (name.startsWith("<") || name.startsWith("[")) return false;
  if (name.length > 200) return false;
  return true;
}

/** 获取代理显示名称（优先使用存储名称，无效时从 URL 提取） */
export function getProxyDisplayName(proxy: ProxyInfo): string {
  if (isValidProxyName(proxy.proxy_name)) {
    return proxy.proxy_name;
  }
  return extractProxyName(proxy.url);
}

/** 代理类型选项 */
export const PROXY_TYPE_OPTIONS: { label: string; value: ProxyType | "" }[] = [
  { label: "全部类型", value: "" },
  { label: "下载代理", value: "download" },
  { label: "克隆代理", value: "clone" },
  { label: "RAW代理", value: "raw" },
  { label: "SSH代理", value: "ssh" },
];

/** 代理测试结果 */
export interface ProxyTestResult {
  proxy_id: number;
  success: boolean;
  latency: number | null;
  error: string | null;
  test_url: string;
}

export function useProxyList() {
  const footer = inject(FOOTER_KEY)!;
  const settingsStore = useSettingsStore();

  const pageSize = ref(50);
  const currentPage = ref(1);
  const entries = ref<ProxyInfo[]>([]);
  const selectedIds = ref(new Set<number>());
  const searchQuery = ref("");
  const typeFilter = ref<ProxyType | "">("");
  const loading = ref(false);

  /** 测试结果映射（proxy_id -> 测试结果） */
  const testResults = ref<Map<number, ProxyTestResult>>(new Map());
  /** 正在测试的代理 ID 列表 */
  const testingIds = ref<Set<number>>(new Set());

  onMounted(async () => {
    pageSize.value = await settingsStore.getSettingNumber("list_page_size_proxy", 50);
  });

  const filteredEntries = computed(() => {
    let result = entries.value;
    if (typeFilter.value) {
      result = result.filter((e) => e.proxy_type === typeFilter.value);
    }
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter((e) =>
        e.proxy_name.toLowerCase().includes(q) ||
        e.url.toLowerCase().includes(q) ||
        e.proxy_type.toLowerCase().includes(q) ||
        extractProxyName(e.url).toLowerCase().includes(q)
      );
    }
    return result;
  });

  const totalRecords = computed(() => filteredEntries.value.length);

  const pageData = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value;
    return filteredEntries.value.slice(start, start + pageSize.value);
  });

  function syncToolbar() {
    const s = filteredEntries.value;
    footer.infoText = `总计: ${s.length} 个代理`;
    footer.showPagination = s.length > pageSize.value;
    footer.totalRecords = s.length;
    footer.currentPage = currentPage.value;
    footer.pageSize = pageSize.value;
    footer.onPageChange = goToPage;
  }

  function goToPage(page: number) {
    currentPage.value = page;
  }

  watch(totalRecords, syncToolbar);
  watch(searchQuery, () => { currentPage.value = 1; });
  watch(typeFilter, () => { currentPage.value = 1; });
  watch(currentPage, (p) => {
    footer.currentPage = p;
    footer.onPageChange = goToPage;
  });

  /** 加载代理列表 */
  async function fetchEntries() {
    loading.value = true;
    try {
      entries.value = await invoke<ProxyInfo[]>("get_proxies");
    } finally {
      loading.value = false;
      syncToolbar();
    }
  }

  /** 切换单行选中 */
  function toggleSelect(id: number) {
    const s = new Set(selectedIds.value);
    if (s.has(id)) s.delete(id);
    else s.add(id);
    selectedIds.value = s;
  }

  /** 全选/取消全选 */
  function toggleSelectAll() {
    if (pageData.value.every((p) => selectedIds.value.has(p.proxy_id!))) {
      selectedIds.value = new Set();
    } else {
      selectedIds.value = new Set(pageData.value.map((p) => p.proxy_id!));
    }
  }

  const setSelected = (v: Set<number>) => { selectedIds.value = v; };

  /** 切换代理启用状态 */
  async function toggleProxyActive(proxy: ProxyInfo) {
    try {
      await invoke("set_proxy_active", {
        proxyId: proxy.proxy_id,
        isActive: !proxy.is_active,
      });
      proxy.is_active = !proxy.is_active;
    } catch (e) {
      throw e;
    }
  }

  /** 更新代理信息（编辑名称/URL/类型） */
  async function updateProxy(proxyId: number, updates: Partial<ProxyInfo>) {
    const proxy = entries.value.find((p) => p.proxy_id === proxyId);
    if (!proxy) throw new Error("代理不存在");
    try {
      await invoke("update_proxy", {
        proxyId,
        proxyName: updates.proxy_name ?? proxy.proxy_name,
        url: updates.url ?? proxy.url,
        proxyType: updates.proxy_type ?? proxy.proxy_type,
      });
      // 更新本地数据
      if (updates.proxy_name !== undefined) proxy.proxy_name = updates.proxy_name;
      if (updates.url !== undefined) proxy.url = updates.url;
      if (updates.proxy_type !== undefined) proxy.proxy_type = updates.proxy_type;
    } catch (e) {
      throw e;
    }
  }

  /** 删除代理 */
  async function deleteProxy(proxyId: number) {
    try {
      await invoke("delete_proxy", { proxyId });
    } catch (e) {
      throw e;
    }
  }

  /** 批量删除代理 */
  async function deleteSelectedProxies() {
    if (selectedIds.value.size === 0) return;
    loading.value = true;
    try {
      for (const id of selectedIds.value) {
        await invoke("delete_proxy", { proxyId: id });
      }
      selectedIds.value = new Set();
      await fetchEntries();
    } finally {
      loading.value = false;
    }
  }

  /** 设置测试结果（同时更新本地条目数据） */
  function setTestResult(proxyId: number, result: ProxyTestResult) {
    testResults.value.set(proxyId, result);
    // 同步更新对应条目的持久化测试统计
    const entry = entries.value.find((e) => e.proxy_id === proxyId);
    if (entry) {
      entry.avg_latency = result.latency;
      if (result.success) {
        entry.success_count = (entry.success_count || 0) + 1;
      } else {
        entry.fail_count = (entry.fail_count || 0) + 1;
      }
    }
    // 触发响应式更新
    testResults.value = new Map(testResults.value);
  }

  /** 清除测试结果 */
  function clearTestResults() {
    testResults.value = new Map();
  }

  /** 获取测试状态文本 */
  function getTestStatusText(proxyId: number): string {
    const result = testResults.value.get(proxyId);
    if (!result) return "未测试";
    if (result.success) return `${result.latency}ms`;
    return `失败: ${result.error || "未知错误"}`;
  }

  /** 获取测试状态样式类 */
  function getTestStatusClass(proxyId: number): string {
    const result = testResults.value.get(proxyId);
    if (!result) return "";
    return result.success ? "status-success" : "status-error";
  }

  return {
    pageSize,
    currentPage,
    entries,
    selectedIds,
    searchQuery,
    typeFilter,
    loading,
    filteredEntries,
    totalRecords,
    pageData,
    testResults,
    testingIds,
    fetchEntries,
    toggleSelect,
    toggleSelectAll,
    setSelected,
    toggleProxyActive,
    deleteProxy,
    deleteSelectedProxies,
    setTestResult,
    clearTestResults,
    getTestStatusText,
    getTestStatusClass,
    syncToolbar,
    updateProxy,
    extractProxyName,
    isValidProxyName,
    getProxyDisplayName,
  };
}
