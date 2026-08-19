/**
 * dashboard.ts - 仪表盘与页面状态类型定义
 *
 * 功能：
 * - 定义仪表盘统计、底部工具栏状态等类型
 * - 与后端 Rust get_dashboard_stats 返回模型保持一致
 */

/** 仪表盘统计（后端 get_dashboard_stats 返回，各模块计数） */
export interface DashboardStats {
  /** 软件包总数 */
  pkg_total: number;
  /** 已是最新的软件包数 */
  pkg_updated: number;
  /** 有更新的软件包数 */
  pkg_outdated: number;
  /** 备份记录总数 */
  backup_total: number;
  /** 缓存记录总数 */
  cache_total: number;
  /** 代理源总数 */
  proxy_total: number;
  /** 可用代理源数 */
  proxy_active: number;
  /** License 枚举总数 */
  license_total: number;
  /** 编程语言枚举总数 */
  language_total: number;
}

/** Footer 状态接口 - 管理底部工具栏状态 */
export interface FooterState {
  infoText: string;
  showPagination: boolean;
  totalRecords: number;
  currentPage: number;
  pageSize: number;
  onPageChange: ((page: number) => void) | null;
  progress: { current: number; total: number; message?: string } | null;
}
