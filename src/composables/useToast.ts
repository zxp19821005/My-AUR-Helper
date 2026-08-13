/**
 * useToast.ts - 全局瞬时消息（toast）队列
 *
 * 功能：
 * - 维护模块级 reactive 的 toast 队列，跨页面/弹窗共享同一出口
 * - addToast 入队一条消息，按 duration 自动消失（默认 3000ms，可自定义）
 * - 多条消息按时间顺序堆叠，互不覆盖（由 ToastContainer 渲染）
 *
 * 设计说明：
 * - 原 footer.addMessage 已改为调用本模块的 addToast，因此所有历史调用点
 *   无需改动即可获得右下角 toast 能力。
 * - 队列上限 MAX_TOASTS，超出时丢弃最早的一条，避免堆叠过多。
 */
import { reactive } from "vue";
import type { MessageLevel } from "./footer";

/** 单条 toast 数据 */
export interface ToastItem {
  id: number;
  level: MessageLevel;
  text: string;
  /** 自动消失时间（毫秒），0 表示不自动消失 */
  duration: number;
}

/** 同时存在的最大 toast 条数 */
const MAX_TOASTS = 5;

let toastIdCounter = 0;

/** 全局 toast 队列（模块级，单例） */
const toasts = reactive<ToastItem[]>([]);

/**
 * 添加一条 toast 消息
 *
 * @param level - 消息级别（info/success/warning/error）
 * @param text - 消息内容
 * @param duration - 自动消失时间（毫秒），默认 3000，传 0 表示不自动消失
 * @returns toast 的唯一 id
 */
export function addToast(
  level: MessageLevel,
  text: string,
  duration = 3000
): number {
  const id = ++toastIdCounter;
  toasts.push({ id, level, text, duration });
  // 超出上限时移除最早的 toast
  if (toasts.length > MAX_TOASTS) {
    toasts.splice(0, toasts.length - MAX_TOASTS);
  }
  return id;
}

/**
 * 按 id 移除一条 toast（手动关闭或自动消失时调用）
 *
 * @param id - toast 的唯一 id
 */
export function removeToast(id: number): void {
  const idx = toasts.findIndex((t) => t.id === id);
  if (idx !== -1) {
    toasts.splice(idx, 1);
  }
}

/**
 * 清空所有 toast
 */
export function clearToasts(): void {
  toasts.splice(0, toasts.length);
}

/**
 * 获取当前 toast 队列（只读引用）
 */
export function useToast() {
  return { toasts, addToast, removeToast, clearToasts };
}
