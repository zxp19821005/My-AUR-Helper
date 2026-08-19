/**
 * enum.ts - 枚举表相关类型定义
 *
 * 功能：
 * - 定义 License 与编程语言枚举类型
 * - 与后端 Rust EnumLicense / EnumProgrammingLanguage 模型保持一致
 */

/** License 类型别名 - 指向完整的 EnumLicense 接口 */
export type License = EnumLicense;

/** 编程语言类型别名 - 指向完整的 EnumProgrammingLanguage 接口 */
export type Language = EnumProgrammingLanguage;

export interface EnumLicense {
  id: number | null;
  spdx_id: string;
  full_name: string;
}

/**
 * 编程语言信息
 * 存储编程语言的配置，用于识别软件包语言类型
 */
export interface EnumProgrammingLanguage {
  /** 语言 ID - 数据库主键 */
  id: number | null;
  /** 语言名称 - 如 "Rust"、"Python" */
  name: string;
  /** 简称 - 如 "rs"、"py" */
  short_name: string | null;
}
