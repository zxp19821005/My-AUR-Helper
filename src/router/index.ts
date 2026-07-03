/**
 * index.ts - 路由配置
 *
 * 功能：
 * - 定义应用所有页面的路由规则
 * - 管理路由路径到组件的映射
 * - 主窗口路由：仪表盘、软件管理、备份管理、缓存管理、代理管理
 * - 弹出窗口路由：设置、枚举值管理、日志
 *
 * 路由结构：
 * - 主窗口使用一级路由
 * - 弹出窗口使用嵌套路由，父级为布局组件
 */
import { createRouter, createWebHistory } from "vue-router";  // Vue Router 核心 API

// 主窗口页面组件
import Dashboard from "../views/Dashboard.vue";          // 仪表盘页面 - 显示统计概览和快速操作
import PackageList from "../views/PackageList.vue";      // 软件包列表页面 - 显示所有 AUR 软件包
import PackageDetail from "../views/PackageDetail.vue";  // 软件包详情页面 - 显示单个软件包详细信息
import BackupManager from "../views/BackupManager.vue";  // 备份管理页面 - 管理系统备份
import CacheManager from "../views/CacheManager.vue";    // 缓存管理页面 - 管理本地缓存包
import ProxySettings from "../views/ProxySettings.vue";  // 代理设置页面 - 管理代理源

// 弹出窗口布局组件
import SettingsPopup from "../components/SettingsPopup.vue";  // 设置窗口布局 - 左侧菜单 + 右侧内容
import EnumLayout from "../components/EnumLayout.vue";        // 枚举值管理窗口布局
import LogsPopup from "../components/LogsPopup.vue";          // 日志窗口布局

// 弹出窗口子页面组件
import Settings from "../views/Settings.vue";            // 设置页面 - 应用配置管理
import LicenseManager from "../views/LicenseManager.vue"; // License 管理页面
import LanguageManager from "../views/LanguageManager.vue"; // 编程语言管理页面
import LogViewer from "../views/LogViewer.vue";           // 日志查看页面

/**
 * 路由配置
 * 主窗口路由和弹出窗口路由分开配置
 */
const routes = [
  // ===== 主窗口路由 =====

  /** 仪表盘 - 默认首页 */
  { path: "/", name: "Dashboard", component: Dashboard },

  /** 软件包列表 */
  { path: "/packages", name: "PackageList", component: PackageList },

  /** 软件包详情 - :pkgname 为动态路由参数，表示软件包名称 */
  { path: "/packages/:pkgname", name: "PackageDetail", component: PackageDetail },

  /** 备份管理 */
  { path: "/backup", name: "BackupManager", component: BackupManager },

  /** 缓存管理 */
  { path: "/cache", name: "CacheManager", component: CacheManager },

  /** 代理管理 */
  { path: "/proxy", name: "ProxySettings", component: ProxySettings },

  // ===== 弹出窗口路由 =====

  /** 设置弹出窗口 - 使用嵌套路由，子路由对应不同设置分类 */
  {
    path: "/settings",
    component: SettingsPopup,
    children: [
      { path: "", name: "SettingsGeneral", component: Settings },    // 通用设置
      { path: "aur", name: "SettingsAur", component: Settings },     // AUR 设置
      { path: "checker", name: "SettingsChecker", component: Settings },  // 检查器设置
      { path: "backup", name: "SettingsBackup", component: Settings },    // 备份设置
      { path: "cache", name: "SettingsCache", component: Settings },      // 缓存设置
      { path: "proxy", name: "SettingsProxy", component: Settings },      // 代理设置
    ],
  },

  /** 枚举值管理弹出窗口 */
  {
    path: "/enums",
    component: EnumLayout,
    children: [
      { path: "", redirect: "/enums/licenses" },  // 默认重定向到 License 管理
      { path: "licenses", name: "EnumLicenses", component: LicenseManager },      // License 管理
      { path: "languages", name: "EnumLanguages", component: LanguageManager },    // 编程语言管理
    ],
  },

  /** 日志弹出窗口 */
  {
    path: "/logs",
    component: LogsPopup,
    children: [
      { path: "", name: "Logs", component: LogViewer },  // 日志查看
    ],
  },
];

/** 创建路由实例 - 使用 HTML5 History 模式 */
const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
