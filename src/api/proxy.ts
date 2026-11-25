/**
 * proxy.ts - 代理领域 API 封装
 *
 * 功能：
 * - 集中封装代理的增删改查、文件下载/解析、连通性测试等 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，命令经 tauri-specta bindings 编译期校验
 * - ProxyType 枚举后端 serde 为首字母大写（Download）、前端用小写在 IPC 边界适配
 */

import { commands } from "@/bindings";
import type { ProxyInfo, ProxyTestResult, ProxyType } from "@/types";

/** 列出全部代理 */
export async function getProxies(): Promise<ProxyInfo[]> {
  return await commands.getProxies() as unknown as ProxyInfo[];
}

/** 切换代理启用状态 */
export async function setProxyActive(proxyId: number, isActive: boolean): Promise<void> {
  await commands.setProxyActive(proxyId, isActive);
}

/** 更新代理信息（名称 / URL / 类型） */
export async function updateProxy(
  proxyId: number,
  proxyName: string,
  url: string,
  proxyType: ProxyType,
): Promise<void> {
  await commands.updateProxy(proxyId, proxyName, url, proxyType);
}

/** 删除单个代理 */
export async function deleteProxy(proxyId: number): Promise<void> {
  await commands.deleteProxy(proxyId);
}

/** 下载代理文件，返回新增条目数 */
export async function downloadProxyFile(): Promise<number> {
  return await commands.downloadProxyFile();
}

/** 解析代理文件，返回解析条目数 */
export async function parseProxyFile(): Promise<number> {
  return await commands.parseProxyFile();
}

/** 清空代理相关表，返回清空条目数 */
export async function clearProxyTables(): Promise<number> {
  return await commands.clearProxyTables();
}

/** 批量测试代理连通性，返回测试结果列表 */
export async function testProxiesBatch(proxyIds: number[] | null): Promise<ProxyTestResult[]> {
  return await commands.testProxiesBatch(proxyIds) as unknown as ProxyTestResult[];
}

/** 测试单个代理连通性，返回测试结果 */
export async function testProxySingle(proxyId: number): Promise<ProxyTestResult> {
  return await commands.testProxySingle(proxyId) as unknown as ProxyTestResult;
}
