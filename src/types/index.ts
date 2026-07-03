/**
 * index.ts - TypeScript 类型定义
 *
 * 功能：
 * - 定义应用中使用的所有数据类型和接口
 * - 与后端 Rust 模型保持一致，确保前后端数据一致性
 * - 提供类型安全的开发体验
 */

/**
 * 软件包类型枚举
 * 1: 编译安装 (compiled) - 从 AUR 源码编译
 * 2: 二进制包 (binary) - 预编译二进制包
 * 3: Git 仓库 (git) - 从 Git 仓库直接安装
 * 4: AppImage - AppImage 格式应用
 */
export type PackageType = 1 | 2 | 3 | 4;

/**
 * 检查器类型枚举
 * 1: GitHub Release - 检查 GitHub Release 版本
 * 2: GitHub Tag - 检查 GitHub Tag 版本
 * 3: Gitee - 通过 Gitee API 检查
 * 4: GitLab - 通过 GitLab API 检查
 * 5: 重定向 (redirect) - 通过 HTTP 重定向获取版本
 * 6: HTTP 页面解析 - 从 HTML 页面解析版本号
 * 7: 手动检查 (manual) - 手动更新，无自动检查
 */
export type CheckerType = 1 | 2 | 3 | 4 | 5 | 6 | 7;

/** 代理类型 - 定义代理支持的协议类型 */
export type ProxyType = "download" | "clone" | "raw" | "ssh";

/**
 * 软件包信息
 * 存储软件包的基本信息和配置，对应后端 SoftwareInfo 结构体
 */
export interface SoftwareInfo {
  /** 软件包 ID - 数据库主键，新建时为 null */
  software_id: number | null;
  /** 软件包名称 - AUR 中的包名 */
  pkgname: string;
  /** 上游项目地址 - 源码仓库或项目主页 URL */
  upstream_url: string | null;
  /** 软件包类型 - 参见 PackageType 枚举 */
  package_type: PackageType;
  /** 检查器类型 - 参见 CheckerType 枚举 */
  checker_type: CheckerType;
  /** 是否有更新 - true 表示上游有更新可用 */
  is_outdated: boolean;
  /** 是否检查测试版本 - 是否将预览版/测试版纳入版本检查 */
  check_test_versions: boolean;
  /** 是否检查二进制文件 - 是否检查二进制包更新 */
  check_binary_files: boolean;
  /** 是否启用自动检查 - 是否定期自动检查上游版本 */
  auto_check_enabled: boolean;
  /** License ID - 关联的许可证 ID */
  license_id: number | null;
  /** 编程语言 ID - 关联的编程语言 ID */
  language_id: number | null;
  /** 创建时间 - Unix 时间戳（毫秒） */
  created_at: number;
}

/**
 * 上游版本信息
 * 存储软件包的上游版本检查结果，对应后端 UpstreamInfo 结构体
 */
export interface UpstreamInfo {
  /** 软件包 ID */
  software_id: number;
  /** 上游项目地址 */
  upstream_url: string | null;
  /** 上游版本号 - 从上游检查到的版本字符串 */
  upstream_version: string | null;
  /** 上游 License - 上游项目声明的许可证 */
  upstream_license: string | null;
  /** 上次检查时间 - ISO 格式时间字符串 */
  last_checked: string | null;
}

/**
 * 代理信息
 * 存储代理源的配置，用于加速 AUR 资源下载
 */
export interface ProxyInfo {
  /** 代理 ID - 数据库主键，新建时为 null */
  proxy_id: number | null;
  /** 代理名称 - 显示名称 */
  proxy_name: string;
  /** 代理类型 - download/clone/raw/ssh */
  proxy_type: ProxyType;
  /** 代理 URL - 代理服务器地址 */
  url: string;
  /** 是否启用 - 是否激活此代理 */
  is_active: boolean;
}

/** License 类型别名 - 指向完整的 EnumLicense 接口 */
export type License = EnumLicense;

/** 编程语言类型别名 - 指向完整的 EnumProgrammingLanguage 接口 */
export type Language = EnumProgrammingLanguage;

/**
 * License 信息
 * 存储 SPDX License 数据，用于软件包许可证管理
 */
export interface EnumLicense {
  /** License ID - 数据库主键 */
  id: number | null;
  /** SPDX ID - SPDX 标准标识符，如 "MIT"、"GPL-3.0" */
  spdx_id: string;
  /** 完整名称 - License 的完整名称 */
  full_name: string;
  /** License URL - License 的官方文档链接 */
  url: string | null;
  /** 是否已弃用 - 是否被 SPDX 标记为已弃用 */
  is_deprecated: boolean;
  /** 是否被 OSI 批准 - 是否符合开源促进会标准 */
  is_osi_approved: boolean;
  /** 描述 - License 的简要说明 */
  description: string | null;
  /** 分类 - License 的分类标签 */
  category: string | null;
  /** 创建时间 - ISO 格式时间字符串 */
  created_at: string | null;
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
  /** 描述 - 语言的简要说明 */
  description: string | null;
  /** 文件扩展名 - 逗号分隔的字符串，如 ".rs,.toml" */
  file_extensions: string | null;
  /** 构建系统 - 如 "cargo"、"pip" */
  build_system: string | null;
  /** 构建命令 - 如 "cargo build"、"pip install" */
  build_command: string | null;
}

/**
 * 日志条目
 * 存储应用日志信息，用于调试和问题排查
 */
export interface LogEntry {
  /** 日志 ID - 数据库主键 */
  id: number | null;
  /** 日志级别 - INFO/WARN/ERROR/DEBUG */
  level: string;
  /** 日志消息 - 具体的日志内容 */
  message: string;
  /** 日志模块 - 产生日志的代码模块名 */
  module: string | null;
  /** 创建时间 - ISO 格式时间字符串 */
  created_at: string;
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

/**
 * 检测到的缓存信息
 * 存储系统缓存扫描结果，用于缓存管理
 */
export interface DetectedCache {
  /** 缓存类型 - pacman/paru/yay 等包管理器标识 */
  cache_type: string;
  /** 缓存路径 - 缓存文件在磁盘上的路径 */
  cache_path: string;
  /** 包数量 - 缓存中包含的软件包数量 */
  package_count: number;
  /** 总大小 - 缓存占用的磁盘空间（字节） */
  total_size_bytes: number;
}

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
