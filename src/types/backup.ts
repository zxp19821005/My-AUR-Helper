/**
 * backup.ts - 备份相关类型定义
 *
 * 功能：
 * - 定义备份结果、备份软件包列表条目、去重结果等类型
 * - 与后端 Rust 备份模块模型保持一致
 */

/**
 * 备份结果
 * 存储备份操作的执行结果，用于显示备份状态
 */
export interface BackupResult {
  /** 已复制的文件数 - 成功备份的文件数量 */
  copied: number;
  /** 已清理的文件数 - 备份后清理的旧文件数量 */
  removed: number;
  /** 错误信息列表 - 备份过程中遇到的错误 */
  errors: string[];
}

/** 备份软件包列表展示条目（含软件包名称） */
export interface BackupSoftwareEntry {
  id: number;
  pkgname: string;
  name: string;
  filename: string;
  epoch: number;
  pkgver: string;
  pkgrel: string;
  arch: string;
  subdirectory: string | null;
  full_path: string;
}

/** 备份去重结果 */
export interface DeduplicateResult {
  removed_files: number;
  removed_records: number;
  errors: string[];
}
