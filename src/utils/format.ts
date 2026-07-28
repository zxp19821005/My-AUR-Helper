/**
 * format.ts - 通用格式化工具函数
 *
 * 功能：
 * - 格式化时间戳
 * - 格式化 License JSON 字符串
 * - 解析 JSON 数组字符串为逗号分隔的列表
 * - 获取包类型、检查器类型的显示名称
 */
import { pkgTypeOptions, checkerTypeOptions } from "./enums";

/**
 * 格式化 Unix 时间戳为日期字符串
 *
 * @param ts Unix 时间戳（秒）
 * @returns 格式化后的日期字符串，空值返回 "—"
 */
export function formatTimestamp(ts: number | null | undefined): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}

/**
 * 格式化 License JSON 字符串为显示文本
 *
 * @param licenseJson License 数组的 JSON 字符串，或普通字符串
 * @returns 逗号分隔的 License 列表，空值返回 "—"
 */
export function formatLicense(licenseJson: string | null | undefined): string {
  if (!licenseJson) return "—";
  try {
    const parsed = JSON.parse(licenseJson);
    if (Array.isArray(parsed)) {
      return parsed.length > 0 ? parsed.join(", ") : "—";
    }
  } catch {
    // Not JSON, return as is
  }
  return licenseJson;
}

/**
 * 解析 JSON 数组字符串为逗号分隔的列表
 *
 * @param val JSON 数组字符串，或普通字符串
 * @returns 逗号分隔的列表，空值返回 "—"
 */
export function parseJsonList(val: string | null | undefined): string {
  if (!val) return "—";
  try {
    const arr = JSON.parse(val);
    return Array.isArray(arr) ? arr.join(", ") : val;
  } catch {
    return val;
  }
}

/**
 * 获取包类型的显示名称
 *
 * @param id 包类型 ID
 * @returns 显示名称，未知 ID 返回 "未知"
 */
export function getPkgTypeName(id: number | null | undefined): string {
  if (id == null) return "未知";
  return pkgTypeOptions.find((t) => t.id === id)?.label || "未知";
}

/**
 * 获取检查器类型的显示名称
 *
 * @param id 检查器类型 ID
 * @returns 显示名称，未知 ID 返回 "未知"
 */
export function getCheckerTypeName(id: number | null | undefined): string {
  if (id == null) return "未知";
  return checkerTypeOptions.find((c) => c.id === id)?.label || "未知";
}

/**
 * 根据语言 ID 列表获取语言名称列表
 *
 * @param ids 语言 ID 列表
 * @param languages 所有语言的映射
 * @returns 逗号分隔的语言名称列表
 */
export function getLanguageNames(
  ids: number[] | null | undefined,
  languages: { id: number | null; name: string }[]
): string {
  if (!ids || ids.length === 0) return "—";
  return (
    ids
      .map((id) => languages.find((l) => l.id === id)?.name)
      .filter(Boolean)
      .join(", ") || "—"
  );
}
