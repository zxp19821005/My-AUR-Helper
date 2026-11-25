/**
 * errors.ts - 前端错误判断与格式化工具
 *
 * 功能：
 * - isNetworkError: 判断错误是否属于网络类（关键词表统一维护）
 * - formatErrorMessage: 把错误格式化为用户可见的提示文案
 *
 * 工作流程：
 * 1. String(e) 统一转成文本，网络判定在 lowercase 文本上做关键词匹配；
 * 2. 网络异常返回 `${context}：网络异常，请检查网络连接后重试。`，并在原始文本
 *    超过 60 字时追加「（详情见日志）」；其余返回 `${context}失败：${msg}`。
 *
 * 适用场景：软件包检查 / 同步 / 删除等 composable 的 catch 分支错误提示，
 * 与 composables/useToast.ts 的 showActionError 配合使用。文案与网络判定关键词
 * 与抽取前三个 composable 的本地实现逐字等价。
 */

/** 网络错误关键词表（一处维护，顺序与原三份实现一致） */
const NETWORK_ERROR_KEYWORDS = [
  "网络",
  "timeout",
  "连接",
  "connect",
  "fetch",
  "request failed",
];

/**
 * 判断错误是否为网络相关错误
 *
 * @param e - catch 到的任意错误对象
 * @returns 命中任一网络关键词返回 true
 */
export function isNetworkError(e: unknown): boolean {
  const msg = String(e).toLowerCase();
  return NETWORK_ERROR_KEYWORDS.some((keyword) => msg.includes(keyword));
}

/**
 * 格式化错误提示信息
 *
 * @param e - catch 到的任意错误对象
 * @param context - 操作上下文（如「AUR同步」「批量删除」）
 * @returns 可直接展示给用户的提示文案
 */
export function formatErrorMessage(e: unknown, context: string): string {
  const msg = String(e);
  if (isNetworkError(e)) {
    return `${context}：网络异常，请检查网络连接后重试。${msg.length > 60 ? "（详情见日志）" : ""}`;
  }
  return `${context}失败：${msg}`;
}
