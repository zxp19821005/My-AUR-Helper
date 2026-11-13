/**
 * filterConfig.ts - 软件包列表筛选配置与纯匹配函数
 *
 * 被 usePackageList 与 FilterBar 共享：
 * - FilterState / QuickFilterKey：筛选状态类型
 * - quickFilterOptions：工具栏单选快速筛选下拉选项
 * - createDefaultFilterState：默认筛选状态工厂
 * - matchesSingleQuickFilter / matchesQuickFilters / matchesConditionFilters：
 *   纯函数形式的匹配判定，便于在 computed 中复用而不污染 composable 体积
 *
 * 筛选说明：
 * - 多重快速筛选（quickFilters）：由筛选面板 FilterBar 以复选框控制，OR 逻辑
 * - 单选快速筛选（quickFilter）：由工具栏下拉控制，仅允许同时选中一项，
 *   与多重快速筛选之间同样为 OR 关系（命中任一即通过）
 * - 条件筛选（conditionFilters）：软件包类型 / 检查器类型，AND 逻辑
 */
import type { SoftwareListEntry } from "@/types";

/** 筛选状态 */
export interface FilterState {
  /** 多重快速筛选条件（OR 逻辑，由筛选面板 FilterBar 控制） */
  quickFilters: {
    /** 上游 URL 为空 */
    upstreamUrlEmpty: boolean;
    /** AUR 版本为空（旧语义，保留给 FilterBar） */
    aurUpdateFailed: boolean;
    /** 上游版本为空（旧语义，保留给 FilterBar） */
    upstreamUpdateFailed: boolean;
    /** 上游地址异常（upstream_url_status != "ok" 且非空） */
    upstreamUrlAbnormal: boolean;
    /** License 缺失（upstream_license_id 为空） */
    licenseMissing: boolean;
  };
  /** 单选快速筛选（工具栏下拉，与多重筛选 OR 叠加） */
  quickFilter: QuickFilterKey | null;
  /** 条件筛选（AND 逻辑） */
  conditionFilters: {
    /** 软件包类型 */
    packageType: number | null;
    /** 检查器类型 */
    checkerType: number | null;
  };
}

/** 单选快速筛选条件键（工具栏下拉，不支持多选） */
export type QuickFilterKey =
  | "aur_sync_failed"
  | "upstream_check_failed"
  | "github_url_invalid"
  | "no_aur_version"
  | "no_upstream_version"
  | "no_upstream_url"
  | "license_missing";

/** 工具栏快速筛选下拉选项（单选，不支持多选） */
export const quickFilterOptions: { key: QuickFilterKey; label: string }[] = [
  { key: "aur_sync_failed", label: "AUR更新失败" },
  { key: "upstream_check_failed", label: "上游更新失败" },
  { key: "github_url_invalid", label: "Github地址失效" },
  { key: "no_aur_version", label: "无AUR版本" },
  { key: "no_upstream_version", label: "无上游版本" },
  { key: "no_upstream_url", label: "无上游地址" },
  { key: "license_missing", label: "License缺失" },
];

/** 默认筛选状态 */
export function createDefaultFilterState(): FilterState {
  return {
    quickFilters: {
      upstreamUrlEmpty: false,
      aurUpdateFailed: false,
      upstreamUpdateFailed: false,
      upstreamUrlAbnormal: false,
      licenseMissing: false,
    },
    quickFilter: null,
    conditionFilters: {
      packageType: null,
      checkerType: null,
    },
  };
}

/** 判断单条是否命中某个单选快速筛选条件（基于新错误字段，语义准确） */
export function matchesSingleQuickFilter(
  entry: SoftwareListEntry,
  key: QuickFilterKey,
): boolean {
  switch (key) {
    case "aur_sync_failed":
      // AUR 同步失败：最近一次同步写入了错误原因（aur_version 是否为空另算「无AUR版本」）
      return entry.aur_sync_error != null;
    case "upstream_check_failed":
      // 上游检查失败：最近一次检查写入了错误原因
      return entry.upstream_check_error != null;
    case "github_url_invalid":
      // Github 地址失效：URL 已配置但验证状态非 ok
      return !!entry.upstream_url_status && entry.upstream_url_status !== "ok";
    case "no_aur_version":
      return entry.aur_version == null;
    case "no_upstream_version":
      return entry.upstream_version == null;
    case "no_upstream_url":
      return entry.upstream_url == null;
    case "license_missing":
      return entry.upstream_license_id == null;
    default:
      return false;
  }
}

/** 检查是否满足快速筛选条件（多重 OR + 单选 OR，任一命中即通过） */
export function matchesQuickFilters(
  entry: SoftwareListEntry,
  state: FilterState,
): boolean {
  const qf = state.quickFilters;
  const checks: boolean[] = [];
  if (qf.upstreamUrlEmpty) checks.push(entry.upstream_url == null);
  if (qf.aurUpdateFailed) checks.push(entry.aur_version == null);
  if (qf.upstreamUpdateFailed) checks.push(entry.upstream_version == null);
  if (qf.upstreamUrlAbnormal)
    checks.push(!!entry.upstream_url_status && entry.upstream_url_status !== "ok");
  if (qf.licenseMissing) checks.push(entry.upstream_license_id == null);
  const single = state.quickFilter;
  if (single) checks.push(matchesSingleQuickFilter(entry, single));
  // 没有任何激活的快筛 → 通过；有激活的 → 任一命中即通过
  return checks.length === 0 || checks.some((v) => v);
}

/** 检查是否满足条件筛选（AND 逻辑） */
export function matchesConditionFilters(
  entry: SoftwareListEntry,
  state: FilterState,
): boolean {
  const cf = state.conditionFilters;
  if (cf.packageType !== null && entry.package_type_id !== cf.packageType) return false;
  if (cf.checkerType !== null && entry.checker_type_id !== cf.checkerType) return false;
  return true;
}
