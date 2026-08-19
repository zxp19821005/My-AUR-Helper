/**
 * useProxyList.ts - 代理管理列表页面逻辑
 *
 * 功能：
 * - 管理列表分页、搜索、选择状态
 * - 提供格式化函数和操作控制逻辑
 * - 支持代理类型筛选
 *
 * 列表通用逻辑由 useListBase 提供；本文件保留代理特有的：类型筛选、
 * 测试状态、启停/更新/删除等操作。
 */
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useListBase } from "./useListBase";
import type { ProxyInfo, ProxyType } from "../types";

/** 从 URL 提取代理显示名称（域名部分） */
export function extractProxyName(url: string): string {
  // 纯字符串操作不会抛异常，无需 try/catch 包裹
  if (url.includes("://")) {
    const domain = url.split("://")[1]?.split("/")[0];
    if (domain) return domain;
  }
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
  const typeFilter = ref<ProxyType | "">("");
  /** 测试结果映射（proxy_id -> 测试结果） */
  const testResults = ref<Map<number, ProxyTestResult>>(new Map());
  /** 正在测试的代理 ID 列表 */
  const testingIds = ref<Set<number>>(new Set());

  const base = useListBase<ProxyInfo>({
    pageSizeSetting: "list_page_size_proxy",
    getKey: (p) => p.proxy_id!,
    infoText: (t) => `总计: ${t} 个代理`,
    pageResetRefs: [typeFilter],
    filter: (all, q) => {
      let result = all;
      if (typeFilter.value) {
        result = result.filter((e) => e.proxy_type === typeFilter.value);
      }
      if (q) {
        result = result.filter(
          (e) =>
            e.proxy_name.toLowerCase().includes(q) ||
            e.url.toLowerCase().includes(q) ||
            e.proxy_type.toLowerCase().includes(q) ||
            extractProxyName(e.url).toLowerCase().includes(q)
        );
      }
      return result;
    },
  });

  /** 加载代理列表 */
  async function fetchEntries() {
    base.loading.value = true;
    try {
      base.entries.value = await invoke<ProxyInfo[]>("get_proxies");
    } finally {
      base.loading.value = false;
      base.syncToolbar();
    }
  }

  /** 切换代理启用状态 */
  async function toggleProxyActive(proxy: ProxyInfo) {
    // 失败时让异常自然向上传播，由调用方统一处理
    await invoke("set_proxy_active", {
      proxyId: proxy.proxy_id,
      isActive: !proxy.is_active,
    });
    proxy.is_active = !proxy.is_active;
  }

  /** 更新代理信息（编辑名称/URL/类型） */
  async function updateProxy(proxyId: number, updates: Partial<ProxyInfo>) {
    const proxy = base.entries.value.find((p) => p.proxy_id === proxyId);
    if (!proxy) throw new Error("代理不存在");
    // invoke 失败时抛出异常，下方本地更新不会执行
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
  }

  /** 删除代理 */
  async function deleteProxy(proxyId: number) {
    await invoke("delete_proxy", { proxyId });
  }

  /** 批量删除代理 */
  async function deleteSelectedProxies() {
    if (base.selectedIds.value.size === 0) return;
    base.loading.value = true;
    try {
      for (const id of base.selectedIds.value) {
        await invoke("delete_proxy", { proxyId: id });
      }
      base.selectedIds.value = new Set();
      await fetchEntries();
    } finally {
      base.loading.value = false;
    }
  }

  /** 设置测试结果（同时更新本地条目数据） */
  function setTestResult(proxyId: number, result: ProxyTestResult) {
    testResults.value.set(proxyId, result);
    // 同步更新对应条目的持久化测试统计
    const entry = base.entries.value.find((e) => e.proxy_id === proxyId);
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
    ...base,
    typeFilter,
    testResults,
    testingIds,
    fetchEntries,
    toggleProxyActive,
    deleteProxy,
    deleteSelectedProxies,
    setTestResult,
    clearTestResults,
    getTestStatusText,
    getTestStatusClass,
    updateProxy,
    extractProxyName,
    isValidProxyName,
    getProxyDisplayName,
  };
}
