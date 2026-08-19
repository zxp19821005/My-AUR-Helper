/**
 * settings.ts - 设置与日志相关类型定义
 *
 * 功能：
 * - 定义设置项与日志条目类型
 * - 与后端 Rust settings / logs 模型保持一致
 */

/**
 * 日志条目（从文件解析）
 * 存储应用日志信息，用于调试和问题排查
 */
export interface LogEntry {
  /** 时间戳 */
  timestamp: string;
  /** 日志级别 - INFO/WARN/ERROR/DEBUG */
  level: string;
  /** 日志模块 - 产生日志的代码模块名 */
  module: string;
  /** 日志消息 - 具体的日志内容 */
  message: string;
}

/**
 * 设置项
 * 存储应用配置，支持按分类管理
 */
export interface Setting {
  /** 设置 ID - 数据库主键 */
  id: number | null;
  /** 设置键 - 配置项的唯一标识 */
  key: string;
  /** 设置值 - 配置项的值 */
  value: string;
  /** 设置描述 - 配置项的说明文字 */
  description: string | null;
  /** 设置分类 - general/aur/backup/checker 等分组标签 */
  category: string;
  /** 创建时间 - ISO 格式时间字符串 */
  created_at: string | null;
}
