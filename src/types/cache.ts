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

/** 单个缓存域的运行状态（后端 CacheDomainStats 映射） */
export interface CacheDomainStats {
  /** 域标识 */
  domain: string;
  /** 中文展示名 */
  label: string;
  /** 该域是否已加载到内存 */
  loaded: boolean;
  /** 数据条目数 */
  size: number;
  /** 缓存创建时间（Unix 秒） */
  created_at: number | null;
  /** 过期时间（Unix 秒），0 表示永不过期 */
  expires_at: number | null;
  /** 是否支持磁盘持久化 */
  persistent: boolean;
  /** 磁盘缓存文件大小（字节） */
  file_size: number;
}

/** 内存缓存整体统计（后端 MemoryCacheStats 映射） */
export interface MemoryCacheStats {
  /** 是否启用内存缓存 */
  enabled: boolean;
  /** 缓存条目上限 */
  max_entries: number;
  /** 缓存有效期（秒），0 表示永不过期 */
  ttl_secs: number;
  /** 自动写盘周期（秒），0 表示关闭定时写 */
  write_interval_secs: number;
  /** 缓存写入目录 */
  cache_dir: string;
  /** 各缓存域状态 */
  domains: CacheDomainStats[];
  /** 全部域条目总数 */
  total_entries: number;
}
