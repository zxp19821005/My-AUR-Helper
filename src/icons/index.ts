/**
 * src/icons/index.ts — 全局图标管理中心（单一来源）
 *
 * 工作流程：
 * 1. 本项目所有图标统一从这里导入，禁止在组件中直接 `from "@lucide/vue"` 取图标，
 *    也禁止在模板里手写内联 <svg>（避免"同功能多图标 / 异功能同图标"的混乱）。
 * 2. 每个「语义令牌」对应一个确定的 Lucide 图标，修改此处即可全局生效。
 * 3. 命名约定：按"功能语义"取名（如 actionEdit / navProxy / statusError），
 *    而非按图标外形取名，确保「同一功能永远同一图标，不同功能用不同图标」。
 *
 * 使用方式：
 *   import { Icon } from "@/icons";
 *   <component :is="Icon.navProxy" :size="20" />
 *   <StandardizedStatCard :icon="Icon.statTotal" />
 *
 * 注：本项目已安装 @lucide/vue（Lucide 图标库，含 1000+ 图标）。
 */

import type { Component } from "vue";
import {
  // 导航 / 页面
  LayoutDashboard,
  Package,
  HardDrive,
  Database,
  Globe,
  Settings,
  FileText,
  Code,
  // 设置子页面
  List,
  Wifi,
  ScrollText,
  Search,
  // 通用操作
  Plus,
  Pencil,
  Trash2,
  Download,
  Upload,
  RefreshCw,
  RefreshCcw,
  Filter,
  Info,
  X,
  // 业务操作
  FolderSearch, // 扫描目录（在文件夹中查找）
  Copy,
  FolderDown,
  GitMerge,
  Zap,
  TestTube,
  FileCode,
  PackagePlus,
  Eraser, // 清空 / 擦除
  FileX2, // 删除文件（缓存清理）
  // 面板 / 工具栏
  PanelLeftClose,
  PanelLeftOpen,
  // 分页
  Home,
  ChevronLeft,
  ChevronRight,
  SkipForward,
  // 状态
  CheckCircle,
  XCircle,
  AlertTriangle,
  AlertCircle,
  // 排序 / 趋势 / 方向
  ChevronUp,
  ChevronDown,
  ChevronsUpDown,
  ArrowUp,
  ArrowDown,
  Minus,
  ArrowLeft,
  ArrowRight,
  // 可见性 / 加载
  Eye,
  EyeOff,
  Loader2,
  // 时间 / 来源
  Clock,
  // 设置动作
  Save,
  RotateCcw,
} from "@lucide/vue";

/**
 * 语义化图标令牌表 —— 全局图标唯一来源。
 * 每个 key 是一个功能语义，value 是该功能唯一对应的 Lucide 图标组件。
 */
export const Icon = {
  // ===================== 导航 / 页面（彼此必须互不相同） =====================
  navDashboard: LayoutDashboard, // 仪表盘
  navPackages: Package, // 软件管理
  navBackup: HardDrive, // 备份管理
  navCache: Database, // 缓存管理
  navProxy: Globe, // 代理管理
  navSettings: Settings, // 设置
  navLanguages: Code, // 编程语言管理
  navLicenses: FileText, // License 管理

  // ===================== 设置子页面 =====================
  settingsGeneral: Settings, // 通用设置
  settingsList: List, // 列表设置
  settingsAur: Globe, // AUR 设置
  settingsChecker: Search, // 上游检查器设置
  settingsBackup: HardDrive, // 备份管理设置
  settingsCache: Database, // 缓存软件设置
  settingsProxy: Wifi, // 代理管理设置
  settingsLog: ScrollText, // 日志管理设置

  // ===================== 通用操作（同一语义全局统一） =====================
  actionAdd: Plus, // 新增
  actionEdit: Pencil, // 编辑（全局唯一编辑图标）
  actionDelete: Trash2, // 删除 / 移除（全局唯一删除图标）
  actionSearch: Search, // 搜索
  actionClear: X, // 清除 / 关闭
  actionRefresh: RefreshCw, // 刷新数据
  actionFilter: Filter, // 筛选
  actionDownload: Download, // 下载
  actionUpload: Upload, // 上传
  actionCopy: Copy, // 复制
  actionInfo: Info, // 详情 / 信息

  // ===================== 业务操作（不同功能用不同图标） =====================
  syncAur: RefreshCcw, // 从 AUR 同步
  syncPkgbuild: Download, // 从 PKGBUILD 同步（拉取文件）
  scan: FolderSearch, // 扫描目录（在文件夹中查找文件）
  dedup: GitMerge, // 去重（合并重复）
  backupNewVersion: Copy, // 备份新版（复制到已有位置）
  backupTo: FolderDown, // 备份到（选择子目录）
  install: PackagePlus, // 安装
  testBatch: Zap, // 批量测试
  testSingle: TestTube, // 单个测试
  parseProxy: FileCode, // 解析代理文件
  clearTable: Eraser, // 清空表（擦除全部记录，与垃圾桶区分）
  deleteSelected: Trash2, // 删除选中（丢进垃圾桶）
  fullCleanup: FileX2, // 缓存清理（删除磁盘缓存文件）

  // ===================== 面板 / 工具栏 =====================
  panelLeftClose: PanelLeftClose, // 收起侧边栏
  panelLeftOpen: PanelLeftOpen, // 展开侧边栏
  menuEnums: List, // 枚举值管理
  menuLogs: ScrollText, // 日志

  // ===================== 分页 =====================
  pageFirst: Home, // 首页
  pagePrev: ChevronLeft, // 上一页
  pageNext: ChevronRight, // 下一页
  pageLast: SkipForward, // 末页

  // ===================== 状态 =====================
  statusSuccess: CheckCircle, // 成功
  statusError: XCircle, // 错误
  statusWarning: AlertTriangle, // 警告
  statusInfo: Info, // 信息
  statusAlert: AlertCircle, // 提醒 / 有更新

  // ===================== 统计卡片 =====================
  statTotal: Package, // 总包数
  statUpdated: CheckCircle, // 已最新
  statOutdated: AlertCircle, // 有更新
  statProxy: Globe, // 代理源

  // ===================== 排序 / 趋势 / 方向 =====================
  sortAsc: ChevronUp, // 升序
  sortDesc: ChevronDown, // 降序
  sortNone: ChevronsUpDown, // 未排序
  trendUp: ArrowUp, // 趋势上升
  trendDown: ArrowDown, // 趋势下降
  trendNeutral: Minus, // 趋势平稳
  arrowLeft: ArrowLeft, // 上一页 / 后退
  arrowRight: ArrowRight, // 下一页 / 前进

  // ===================== 可见性 / 加载 =====================
  show: Eye, // 显示密码
  hide: EyeOff, // 隐藏密码
  loading: Loader2, // 加载中（旋转）

  // ===================== 时间 / 来源 =====================
  clock: Clock, // 时间 / 更新时间
  sourceAur: Globe, // AUR / 网络来源指示

  // ===================== 设置动作 =====================
  save: Save, // 保存
  reset: RotateCcw, // 重置 / 放弃更改

  // ===================== 折叠 / 展开 =====================
  collapse: ChevronDown, // 收起（指示可向下展开）
  expand: ChevronUp, // 展开（指示可向上收起）
} as const;

/** 图标令牌的类型（所有合法 key 的联合类型） */
export type IconToken = keyof typeof Icon;

/** 图标组件类型 */
export type IconComponent = Component;
