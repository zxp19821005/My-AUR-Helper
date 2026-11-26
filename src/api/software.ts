/**
 * software.ts - 软件包领域 API 封装
 *
 * 功能：
 * - 集中封装「软件包 CRUD / AUR 同步 / 上游版本检查 / URL 校验」相关的 Tauri 命令调用
 * - 组件与 composable 统一通过本模块访问后端，避免 invoke 命令字符串散落各处
 *
 * 设计：每个函数对应一个后端 Tauri 命令，命令名/参数/返回值经 tauri-specta 生成的
 *   bindings 做编译期校验，命令字符串不再手写。
 * 边界适配：后端 enum（PackageType/CheckerType）经 serde(into=i32) 序列化为 number，
 *   specta 据此生成 `number`；前端用更严格的字面量联合（1|2|3|4）。在 IPC 边界处用
 *   `as unknown as T` 收窄到前端类型，运行时值与前端契约一致，仅类型精度不同。
 */

import { commands } from "@/bindings";
import type {
  SoftwareInfo,
  SoftwareDetail,
  SoftwareListEntry,
  ValidateResult,
  AurSearchResult,
  MaintainerSyncResult,
  SyncFromAurResult,
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
  skipCheckUpstream: boolean;
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
  return await commands.getPrevNextSoftware(pkgname);
}

/** 按 pkgname 获取软件包完整详情 */
export async function getSoftwareDetail(pkgname: string): Promise<SoftwareDetail | null> {
  return await commands.getSoftwareDetail(pkgname) as unknown as SoftwareDetail | null;
}

/** 按 pkgname 获取列表条目（轻量视图） */
export async function getSoftwareListEntry(pkgname: string): Promise<SoftwareListEntry | null> {
  return await commands.getSoftwareListEntry(pkgname) as unknown as SoftwareListEntry | null;
}

/** 列出全部软件包（完整信息） */
export async function listSoftware(): Promise<SoftwareInfo[]> {
  return await commands.listSoftware() as unknown as SoftwareInfo[];
}

/** 列出软件包列表视图（分页/展示用轻量结构） */
export async function listSoftwareView(): Promise<SoftwareListEntry[]> {
  return await commands.listSoftwareView() as unknown as SoftwareListEntry[];
}

/** 新增软件包，返回新记录 software_id */
export async function addSoftware(input: AddSoftwareInput): Promise<number> {
  return await commands.addSoftware(input);
}

/** 更新软件包 */
export async function updateSoftware(input: UpdateSoftwareInput): Promise<void> {
  await commands.updateSoftware(input);
}

/** 删除单个软件包 */
export async function deleteSoftware(softwareId: number): Promise<void> {
  await commands.deleteSoftware(softwareId);
}

/** 批量删除软件包 */
export async function batchDeleteSoftware(pkgnameList: string[]): Promise<void> {
  await commands.batchDeleteSoftware(pkgnameList);
}

/** 设置软件包关联的 License（单个 AUR license 名称，null 表示清除） */
export async function setSoftwareLicense(softwareId: number, licenseId: string | null): Promise<void> {
  await commands.setSoftwareLicense(softwareId, licenseId);
}

/** 更新选中包的 AUR 信息（pkgnameList 为 null 表示全部） */
export async function updateAurInfo(pkgnameList: string[] | null): Promise<number> {
  return await commands.updateAurInfo(pkgnameList);
}

/** 从 AUR 全量对齐软件包（刷新本地已有 + 补齐新维护的包） */
export async function syncFromAur(): Promise<SyncFromAurResult> {
  return await commands.syncFromAur();
}

/**
 * 按配置的 AUR 维护者用户名批量同步软件包
 *
 * 从 AUR 拉取该用户名维护/共同维护的全部包：本地已存在的更新（保留上游地址），
 * 不存在的则新建（上游地址留空）。返回本次新增/更新/失败统计。
 */
export async function syncPackagesByMaintainer(): Promise<MaintainerSyncResult> {
  return await commands.syncPackagesByMaintainer();
}

/** 搜索 AUR 中的软件包 */
export async function searchAurPackages(keyword: string): Promise<AurSearchResult[]> {
  return await commands.searchAurPackages(keyword) as unknown as AurSearchResult[];
}

/** 从 AUR 导入软件包到本地数据库 */
export interface ImportAurPackageInput {
  pkgname: string;
  packageType: number;
  checkerType: number;
}
export async function importAurPackage(input: ImportAurPackageInput): Promise<number> {
  return await commands.importAurPackage(input.pkgname, input.packageType, input.checkerType);
}

/** 从 PKGBUILD 同步（pkgname 为 null 表示全部） */
export async function syncFromPkgbuild(pkgname: string | null): Promise<void> {
  await commands.syncFromPkgbuild(pkgname);
}

/** 检查单个软件包的上游版本，返回最新上游版本号 */
export async function checkUpstreamVersion(pkgname: string): Promise<string> {
  return await commands.checkUpstreamVersion(pkgname);
}

/** 检查选中软件包的上游版本，返回 [(pkgname, version)] */
export async function checkSelectedUpstream(pkgnameList: string[]): Promise<[string, string][]> {
  return await commands.checkSelectedUpstream(pkgnameList);
}

/** 检查全部软件包的上游版本，返回 [(pkgname, version)] */
export async function checkAllUpstream(): Promise<[string, string][]> {
  return await commands.checkAllUpstream();
}

/**
 * 预置 GitHub 标签缓存（上游版本检查加速）
 *
 * 为库内全部 GitHub 仓库（或指定的 owner/repo）拉取 tags 并写入 github_tag_cache，
 * 使带版本锁定的 -bin 包（如 electron10-bin）后续检查直接命中缓存、零网络、瞬时完成。
 * 返回成功写入缓存的仓库数量。
 */
export async function preloadGithubTagCache(owner?: string, repo?: string): Promise<number> {
  return await commands.preloadGithubTagCache(owner ?? null, repo ?? null);
}

/** 校验上游 URL 可达性 */
export async function validateUpstreamUrls(pkgnameList: string[] | null): Promise<ValidateResult[]> {
  return await commands.validateUpstreamUrls(pkgnameList);
}
