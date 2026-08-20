/**
 * proxy.ts - 代理领域 API 封装
 *
 * 功能：
 * - 集中封装代理的增删改查、文件下载/解析、连通性测试等 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 */

import { invoke } from "@tauri-apps/api/core";
import type { ProxyInfo, ProxyTestResult, ProxyType } from "@/types";

/** 列出全部代理 */
export async function getProxies(): Promise<ProxyInfo[]> {
  return await invoke<ProxyInfo[]>("get_proxies");
}

/** 切换代理启用状态 */
export async function setProxyActive(proxyId: number, isActive: boolean): Promise<void> {
  await invoke("set_proxy_active", { proxyId, isActive });
}

/** 更新代理信息（名称 / URL / 类型） */
export async function updateProxy(
  proxyId: number,
  proxyName: string,
  url: string,
  proxyType: ProxyType,
): Promise<void> {
  await invoke("update_proxy", { proxyId, proxyName, url, proxyType });
}

/** 删除单个代理 */
export async function deleteProxy(proxyId: number): Promise<void> {
  await invoke("delete_proxy", { proxyId });
}

/** 下载代理文件，返回新增条目数 */
export async function downloadProxyFile(): Promise<number> {
  return await invoke<number>("download_proxy_file");
}

/** 解析代理文件，返回解析条目数 */
export async function parseProxyFile(): Promise<number> {
  return await invoke<number>("parse_proxy_file");
}

/** 清空代理相关表，返回清空条目数 */
export async function clearProxyTables(): Promise<number> {
  return await invoke<number>("clear_proxy_tables");
}

/** 批量测试代理连通性，返回测试结果列表 */
export async function testProxiesBatch(proxyIds: number[] | null): Promise<ProxyTestResult[]> {
  return await invoke<ProxyTestResult[]>("test_proxies_batch", { proxyIds });
}

/** 测试单个代理连通性，返回测试结果 */
export async function testProxySingle(proxyId: number): Promise<ProxyTestResult> {
  return await invoke<ProxyTestResult>("test_proxy_single", { proxyId });
}
