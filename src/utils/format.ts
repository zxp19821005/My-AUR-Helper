/**
 * format.ts - 通用格式化工具函数
 *
 * 功能：
 * - 格式化时间戳
 * - 格式化 License JSON 字符串
 * - 解析 JSON 数组字符串为逗号分隔的列表
 * - 根据语言 ID 列表获取语言名称列表
 */

/**
 * 格式化 Unix 时间戳为本地化日期字符串（zh-CN，形如 "2024/01/15"）
 *
 * 与 fmtTimestamp 的区别（命名相近但语义不同，切勿混用）：
 * - 本函数：toLocaleDateString 输出本地化「完整日期」，空值返回 "—"（em dash），
 *   接受 null / undefined，适合详情类展示。
 * - fmtTimestamp：输出定长 "YYYY-MM-DD" 纯日期，空值返回 "-"，仅接受 number | null，
 *   适合表格 / 卡片中需要定长对齐的场景。
 * 另注：PackageAurInfoCard / PackageUpstreamInfoCard 内各自本地定义的 fmtTimestamp
 * 会输出「含时分的完整日期时间」，与本文件两个函数均不等价，故保留未合并。
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

/**
 * 格式化 Unix 时间戳为 ISO 风格日期字符串（YYYY-MM-DD）
 *
 * 与 formatTimestamp 不同：本函数输出定长 "YYYY-MM-DD" 形式，空值返回 "-"，
 * 用于表格/卡片中对齐显示的版本时间戳。
 *
 * @param ts Unix 时间戳（秒）
 * @returns 格式化后的日期字符串，如 "2024-01-15"
 */
export function fmtTimestamp(ts: number | null): string {
  if (ts == null) return "-";
  const d = new Date(ts * 1000);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}
