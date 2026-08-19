/**
 * package.ts - 软件包相关类型定义
 *
 * 功能：
 * - 定义软件包基础信息、完整详情、上游版本、列表展示等类型
 * - 与后端 Rust SoftwareInfo / UpstreamInfo 模型保持一致
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
 * 8: 浏览器(JS渲染) - 调用本机 Chromium/Chrome 执行 JS 渲染后提取版本
 */
export type CheckerType = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;

/**
 * 软件包信息
 * 存储软件包的基本信息和配置，对应后端 SoftwareInfo 结构体
 */
export interface SoftwareInfo {
  software_id: number | null;
  pkgname: string;
  upstream_url: string | null;
  package_type_id: PackageType;
  checker_type_id: CheckerType;
  is_outdated: boolean;
  check_test_versions: boolean;
  check_binary_files: boolean;
  auto_check_enabled: boolean;
  language_ids: number[];
  version_extract_regex: string | null;
}

/**
 * 软件包完整详情
 * 包含基本信息 + AUR 信息 + 上游版本信息
 */
export interface SoftwareDetail {
  software_id: number | null;
  pkgname: string;
  upstream_url: string | null;
  package_type_id: PackageType;
  checker_type_id: CheckerType;
  is_outdated: boolean;
  check_test_versions: boolean;
  check_binary_files: boolean;
  auto_check_enabled: boolean;
  language_ids: number[];
  version_extract_regex: string | null;
  aur_version: string | null;
  aur_last_updated: number | null;
  aur_pkgdesc: string | null;
  depends: string | null;
  makedepends: string | null;
  optdepends: string | null;
  aur_license_name: string | null;
  upstream_version: string | null;
  upstream_last_checked: number | null;
  upstream_license_name: string | null;
}

/**
 * 上游版本信息
 * 存储软件包的上游版本检查结果，对应后端 UpstreamInfo 结构体
 */
export interface UpstreamInfo {
  /** 软件包 ID */
  software_id: number;
  /** 上游版本号 - 从上游检查到的版本字符串 */
  upstream_version: string | null;
  /** 上游 License 列表 - JSON 数组字符串，如 `["MIT", "GPL-3.0"]` */
  upstream_license_id: string | null;
  /** 上次检查时间 - Unix 时间戳（秒） */
  last_checked: number | null;
}

/** 上游 URL 验证状态枚举 */
export type UpstreamUrlStatus =
  | "ok"
  | "not_found"
  | "forbidden"
  | "redirected"
  | "server_error"
  | "timeout"
  | "connection_error"
  | "other_error";

/** 软件包列表展示条目（含 AUR + Upstream 信息） */
export interface SoftwareListEntry {
  software_id: number;
  pkgname: string;
  package_type_id: PackageType;
  checker_type_id: CheckerType;
  is_outdated: boolean;
  aur_version: string | null;
  aur_last_updated: number | null;
  upstream_version: string | null;
  upstream_last_checked: number | null;
  upstream_url: string | null;
  upstream_url_status: UpstreamUrlStatus | null;
  upstream_license_id: string | null;
}

/** 上游 URL 验证结果 */
export interface ValidateResult {
  software_id: number;
  pkgname: string;
  upstream_url: string | null;
  status: UpstreamUrlStatus;
}
