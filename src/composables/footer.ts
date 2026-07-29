/**
 * footer.ts - 底部工具栏状态管理
 *
 * 功能：
 * - 定义 FooterState 接口和默认状态
 * - 支持信息文本、分页、进度条
 * - 支持消息日志系统（info/success/warning/error 级别）
 * - 提供 addMessage/clearMessages 方法管理日志
 */
import { type InjectionKey } from "vue";

/** 消息级别枚举 */
export type MessageLevel = "info" | "success" | "warning" | "error";

/** 日志消息条目 */
export interface LogMessage {
  id: number;
  level: MessageLevel;
  text: string;
  timestamp: number;
}

/** 消息日志最大保留条数 */
const MAX_MESSAGES = 100;

let messageIdCounter = 0;

export interface FooterState {
  infoText: string;
  showPagination: boolean;
  totalRecords: number;
  currentPage: number;
  pageSize: number;
  onPageChange: ((page: number) => void) | null;
  progress: { current: number; total: number; message?: string } | null;
  /** 消息日志列表 */
  messages: LogMessage[];
  /** 日志面板是否展开 */
  logPanelExpanded: boolean;
}

export const defaultFooterState = (): FooterState => ({
  infoText: "",
  showPagination: false,
  totalRecords: 0,
  currentPage: 1,
  pageSize: 50,
  onPageChange: null,
  progress: null,
  messages: [],
  logPanelExpanded: false,
});

/**
 * 添加日志消息
 * @param footer - FooterState 实例
 * @param level - 消息级别
 * @param text - 消息内容
 */
export function addMessage(
  footer: FooterState,
  level: MessageLevel,
  text: string
): void {
  const msg: LogMessage = {
    id: ++messageIdCounter,
    level,
    text,
    timestamp: Date.now(),
  };
  footer.messages.push(msg);
  // 超过最大条数时移除最早的消息
  if (footer.messages.length > MAX_MESSAGES) {
    footer.messages.splice(0, footer.messages.length - MAX_MESSAGES);
  }
}

/**
 * 清空所有日志消息
 */
export function clearMessages(footer: FooterState): void {
  footer.messages = [];
}

export const FOOTER_KEY: InjectionKey<FooterState> = Symbol("footer");
