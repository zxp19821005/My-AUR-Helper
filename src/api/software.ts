/**
 * software.ts - 软件包领域 API 封装
 *
 * 功能：
 * - 集中封装「软件包 CRUD / AUR 同步 / 上游版本检查 / URL 校验」相关的 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 *
 * 设计：每个函数对应一个后端 Tauri 命令，参数与返回类型与原有 invoke 调用保持一致。
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  SoftwareInfo,
  SoftwareDetail,
  SoftwareListEntry,
  ValidateResult,
} from "@/types";

/** 新增软件包入参（packageType / checkerType 在前端表单中以 number 建模，与后端 i32/i64 对应） */
export interface AddSoftwareInput {
  pkgname: string;
  upstreamUrl: string | null;
  packageType: number;
  checkerType: number;
  checkTestVersions: boolean;
  checkBinaryFiles: boolean;
  autoCheckEnabled: boolean;
  languageIds: number[];
  versionExtractRegex: string | null;
}

/** 更新软件包入参（在新增入参基础上附带 ID 与过期标记） */
export interface UpdateSoftwareInput extends AddSoftwareInput {
  softwareId: number;
  isOutdated: boolean;
}

/** 获取相邻软件包（上一包 / 下一包 pkgname，无则 null） */
export async function getPrevNextSoftware(
  pkgname: string,
): Promise<[string | null, string | null]> {
  return await invoke<[string | null, string | null]>("get_prev_next_software", { pkgname });
}

/** 按 pkgname 获取软件包完整详情 */
export async function getSoftwareDetail(pkgname: string): Promise<SoftwareDetail | null> {
  return await invoke<SoftwareDetail | null>("get_software_detail", { pkgname });
}

/** 按 pkgname 获取列表条目（轻量视图） */
export async function getSoftwareListEntry(pkgname: string): Promise<SoftwareListEntry | null> {
  return await invoke<SoftwareListEntry | null>("get_software_list_entry", { pkgname });
}

/** 列出全部软件包（完整信息） */
export async function listSoftware(): Promise<SoftwareInfo[]> {
  return await invoke<SoftwareInfo[]>("list_software");
}

/** 列出软件包列表视图（分页/展示用轻量结构） */
export async function listSoftwareView(): Promise<SoftwareListEntry[]> {
  return await invoke<SoftwareListEntry[]>("list_software_view");
}

/** 新增软件包，返回新记录 software_id */
export async function addSoftware(input: AddSoftwareInput): Promise<number> {
  return await invoke<number>("add_software", { ...input });
}

/** 更新软件包 */
export async function updateSoftware(input: UpdateSoftwareInput): Promise<void> {
  await invoke("update_software", { ...input });
}

/** 删除单个软件包 */
export async function deleteSoftware(softwareId: number): Promise<void> {
  await invoke("delete_software", { softwareId });
}

/** 批量删除软件包 */
export async function batchDeleteSoftware(pkgnameList: string[]): Promise<void> {
  await invoke("batch_delete_software", { pkgnameList });
}

/** 设置软件包关联的 License（单个 AUR license 名称，null 表示清除） */
export async function setSoftwareLicense(softwareId: number, licenseId: string | null): Promise<void> {
  await invoke("set_software_license", { softwareId, licenseId });
}

/** 更新选中包的 AUR 信息（pkgnameList 为 null 表示全部） */
export async function updateAurInfo(pkgnameList: string[] | null): Promise<number> {
  return await invoke<number>("update_aur_info", { pkgnameList });
}

/** 从 AUR 全量同步软件包信息 */
export async function syncFromAur(): Promise<void> {
  await invoke("sync_from_aur");
}

/** 从 PKGBUILD 同步（pkgname 为 null 表示全部） */
export async function syncFromPkgbuild(pkgname: string | null): Promise<void> {
  await invoke("sync_from_pkgbuild", { pkgname });
}

/** 检查单个软件包的上游版本，返回最新上游版本号 */
export async function checkUpstreamVersion(pkgname: string): Promise<string> {
  return await invoke<string>("check_upstream_version", { pkgname });
}

/** 检查选中软件包的上游版本，返回 [(pkgname, version)] */
export async function checkSelectedUpstream(pkgnameList: string[]): Promise<[string, string][]> {
  return await invoke<[string, string][]>("check_selected_upstream", { pkgnameList });
}

/** 检查全部软件包的上游版本，返回 [(pkgname, version)] */
export async function checkAllUpstream(): Promise<[string, string][]> {
  return await invoke<[string, string][]>("check_all_upstream");
}

/** 校验上游 URL 可达性 */
export async function validateUpstreamUrls(pkgnameList: string[] | null): Promise<ValidateResult[]> {
  return await invoke<ValidateResult[]>("validate_upstream_urls", { pkgnameList });
}
