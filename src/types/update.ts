/**
 * update.ts - 软件更新模块类型定义
 *
 * 功能：定义软件更新工作台的列表行、状态枚举、dry-run 预览与写回结果。
 * 约定：字段保持 snake_case，与 Rust serde 默认序列化一致；
 *      invoke 入参使用 camelCase，由 Tauri 2 自动映射。
 *
 * 说明：列表只展示 software_info.is_outdated = 1 的包；状态仅描述本地
 *      PKGBUILD 是否已写回，由磁盘实际 pkgver 推导，数据库不新增表。
 */

/** 写回状态（与 Rust SoftwareUpdateStatus 对应） */
export type SoftwareUpdateStatus =
  | "pending"
  | "local_updated"
  | "missing"
  | "no_upstream";

/** 软件更新工作台列表行 */
export interface SoftwareUpdateItem {
  /** 软件包主键，同时作为表格 rowKey */
  software_id: number;
  /** 包名 */
  pkgname: string;
  /** AUR 现网版本 */
  aur_version: string | null;
  /** 上游最新版本 */
  upstream_version: string | null;
  /** 归一化后可写入 PKGBUILD 的目标版本 */
  target_version: string;
  /** 本地 PKGBUILD 当前 pkgver */
  local_pkgver: string | null;
  /** 本地 PKGBUILD 当前 pkgrel */
  local_pkgrel: string | null;
  /** 本地 PKGBUILD 绝对路径 */
  pkgbuild_path: string | null;
  /** AUR 是否已标记 out-of-date */
  aur_out_of_date: boolean;
  /** 推导出的状态 */
  status: SoftwareUpdateStatus;
  /** 上游最近检查时间（Unix 秒） */
  last_checked: number | null;
}

/** dry-run 预览条目（只描述改动，不写文件） */
export interface SoftwareUpdatePreview {
  pkgname: string;
  pkgbuild_path: string;
  old_pkgver: string;
  new_pkgver: string;
  old_pkgrel: string;
  new_pkgrel: string;
  /** 仅列出变化行的文本 diff */
  diff: string;
  /** 不可预览的原因；为 null 时可安全执行 */
  error: string | null;
}

/** 单个软件包的写回结果 */
export interface SoftwareUpdateApplyResult {
  pkgname: string;
  success: boolean;
  message: string;
  /** 写回前的备份文件路径 */
  backup_path: string | null;
}

/** 状态中文标签（表格与筛选下拉共用，避免文案分散） */
export const UPDATE_STATUS_LABEL: Record<SoftwareUpdateStatus, string> = {
  pending: "待写回",
  local_updated: "已写回 · 待推送 AUR",
  missing: "PKGBUILD 缺失",
  no_upstream: "无上游版本",
};
