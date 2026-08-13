/**
 * footer.ts - 底部工具栏状态管理
 *
 * 功能：
 * - 定义 FooterState 接口和默认状态（信息文本、分页、进度条）
 * - 提供 addMessage 方法：将操作反馈消息推送到全局 toast 队列
 *
 * 设计说明：
 * - 消息系统已拆分为「瞬时 toast（useToast）+ 统一确认框（useConfirm）」，
 *   底部工具栏右侧 1/3 仅承载不重要的状态信息（infoText / 进度条）。
 * - addMessage 保留 (footer, level, text) 签名以兼容历史调用点，
 *   但内部不再写入日志数组，而是入队 toast。
 */
import { type InjectionKey } from "vue";
import { addToast } from "./useToast";

/** 消息级别枚举 */
export type MessageLevel = "info" | "success" | "warning" | "error";

export interface FooterState {
  infoText: string;
  showPagination: boolean;
  totalRecords: number;
  currentPage: number;
  pageSize: number;
  onPageChange: ((page: number) => void) | null;
  progress: { current: number; total: number; message?: string } | null;
}

export const defaultFooterState = (): FooterState => ({
  infoText: "",
  showPagination: false,
  totalRecords: 0,
  currentPage: 1,
  pageSize: 50,
  onPageChange: null,
  progress: null,
});

/**
 * 添加操作反馈消息（转为右下角 toast）
 *
 * @param _footer - 历史兼容参数（保留以不改变调用点），当前未使用
 * @param level - 消息级别
 * @param text - 消息内容
 */
export function addMessage(
  _footer: FooterState,
  level: MessageLevel,
  text: string
): void {
  addToast(level, text);
}

export const FOOTER_KEY: InjectionKey<FooterState> = Symbol("footer");
