/**
 * useConfirm.ts - 全局统一的确认对话框服务
 *
 * 功能：
 * - 以 Promise 形式替代浏览器原生 confirm()，返回用户选择（true=确认 / false=取消）
 * - 维护模块级 confirmState，由 ConfirmDialog.vue 渲染并调用 resolveConfirm 回写结果
 * - 所有需要二次确认的操作统一走此服务，对话框风格一致
 *
 * 设计说明：
 * - openConfirm 是唯一导出函数，调用点通过 `if (!(await confirm({ message }))) return;`
 *   即可获得与原生 confirm 一致的控制流，无需各自维护弹窗状态。
 * - variant 用于语义化按钮色调（如删除类操作传 "danger"）。
 */
import { reactive } from "vue";

/** 确认对话框按钮色调 */
export type ConfirmVariant = "primary" | "danger" | "warning";

/** openConfirm 入参 */
export interface ConfirmOptions {
  /** 对话框标题，默认“确认操作” */
  title?: string;
  /** 确认提示正文（必填） */
  message: string;
  /** 确认按钮文字，默认“确认” */
  confirmText?: string;
  /** 取消按钮文字，默认“取消” */
  cancelText?: string;
  /** 确认按钮语义色调，默认 "primary" */
  variant?: ConfirmVariant;
}

/** 模块级确认状态（单例），供 ConfirmDialog.vue 绑定 */
export const confirmState = reactive({
  visible: false,
  title: "确认操作",
  message: "",
  confirmText: "确认",
  cancelText: "取消",
  variant: "primary" as ConfirmVariant,
});

/** 当前待回写的 Promise resolve（不放入 reactive，避免被代理） */
let pendingResolve: ((result: boolean) => void) | null = null;

/**
 * 弹出统一确认对话框
 *
 * @param options - 标题/正文/按钮文案/色调
 * @returns Promise<boolean>：用户点击确认返回 true，取消或关闭返回 false
 */
export function openConfirm(options: ConfirmOptions): Promise<boolean> {
  confirmState.visible = true;
  confirmState.title = options.title ?? "确认操作";
  confirmState.message = options.message;
  confirmState.confirmText = options.confirmText ?? "确认";
  confirmState.cancelText = options.cancelText ?? "取消";
  confirmState.variant = options.variant ?? "primary";

  return new Promise<boolean>((resolve) => {
    pendingResolve = resolve;
  });
}

/**
 * 由 ConfirmDialog.vue 在用户点击确认/取消/关闭时调用，回写结果并关闭
 *
 * @param result - true=确认，false=取消
 */
export function resolveConfirm(result: boolean): void {
  confirmState.visible = false;
  if (pendingResolve) {
    pendingResolve(result);
    pendingResolve = null;
  }
}
