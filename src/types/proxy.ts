/**
 * proxy.ts - 代理相关类型定义
 *
 * 功能：
 * - 定义代理类型、代理信息等类型
 * - 与后端 Rust ProxyInfo 模型保持一致
 */

/** 代理类型 - 定义代理支持的协议类型 */
export type ProxyType = "download" | "clone" | "raw" | "ssh";

/**
 * 代理信息
 * 存储代理源的配置，用于加速 AUR 资源下载
 */
export interface ProxyInfo {
  /** 代理 ID - 数据库主键，新建时为 null */
  proxy_id: number | null;
  /** 代理名称 - 显示名称（默认从 URL 域名提取，支持手动编辑覆盖） */
  proxy_name: string;
  /** 代理类型 - download/clone/raw/ssh */
  proxy_type: ProxyType;
  /** 代理 URL - 代理服务器地址 */
  url: string;
  /** 是否启用 - 是否激活此代理 */
  is_active: boolean;
  /** 成功测试次数 - 来自最新测试记录 */
  success_count: number;
  /** 失败测试次数 - 来自最新测试记录 */
  fail_count: number;
  /** 平均延迟(ms) - 来自最新测试记录 */
  avg_latency: number | null;
  /** 最后测试状态 - 'success' | 'fail'，来自最新测试记录，null 表示未测试 */
  last_test_status?: string | null;
  /** 目标协议头约定 - true 表示测试拼接时去除目标地址协议头（isteed 类），false 保留（crashmc 类） */
  strip_target_protocol?: boolean;
}

/**
 * 代理测试结果
 * 单次代理连通性测试返回的结构，由代理测试命令产生
 */
export interface ProxyTestResult {
  /** 代理 ID */
  proxy_id: number;
  /** 是否测试成功 */
  success: boolean;
  /** 平均延迟(ms)，失败为 null */
  latency: number | null;
  /** 错误信息，成功为 null */
  error: string | null;
  /** 实际测试的 URL */
  test_url: string;
}
