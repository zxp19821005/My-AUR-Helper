/**
 * cache.ts - 缓存相关类型定义
 *
 * 功能：
 * - 定义缓存包文件信息、缓存软件包列表条目等类型
 * - 与后端 Rust cache_software 模型保持一致
 */

/** 缓存包文件信息（来自扫描结果 PkgFileInfo） */
export interface CachePackage {
  filename: string;
  name: string;
  epoch: string | null;
  pkgver: string;
  pkgrel: string;
  arch: string;
}

/** 缓存软件包列表展示条目（从数据库 cache_software 读取，后端 CacheSoftwareEntry 映射） */
export interface CacheSoftwareEntry {
  /** 记录 ID */
  id: number;
  /** 包名（优先用 name，否则从文件名解析） */
  pkgname: string;
  /** 缓存文件名 */
  filename: string;
  /** 版本 epoch */
  epoch: number;
  /** 版本号 */
  pkgver: string;
  /** 包发布号 */
  pkgrel: string;
  /** 目标架构 */
  arch: string;
  /** 缓存目录完整路径 */
  cache_directory: string;
  /** 完整文件路径（cache_directory/filename） */
  full_path: string;
}
