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
import { feDebug } from "../utils/felog"; // 前端诊断日志（仅终端）

// 页面组件全部采用动态导入（路由懒加载）：
// 首屏只加载当前路由的模块，避免启动时加载全部 10 个页面及其依赖。
// 背景：Vite dev 模式不打包，首次访问某路由需即时编译其 chunk + 加载成百上千个
// ESM 模块。WebKitGTK 加载大量零散模块远慢于 Chromium，导致首次进入页面卡顿。
// 注意：dev 模式下【任何预加载】都会与首屏渲染争抢主线程导致雪崩，故不做预取；
// release 打包版模块预合并，无此问题。

/** 主窗口页面 */
const Dashboard = () => import("../views/Dashboard.vue");
const PackageList = () => import("../views/PackageList.vue");
const PackageDetail = () => import("../views/PackageDetail.vue");
const BackupManager = () => import("../views/BackupManager.vue");
const CacheManager = () => import("../views/CacheManager.vue");
const ProxySettings = () => import("../views/ProxySettings.vue");

/** 弹出窗口布局 */
const SettingsPopup = () => import("../components/layout/SettingsPopup.vue");
const EnumLayout = () => import("../components/layout/EnumLayout.vue");
const LogsPopup = () => import("../components/layout/LogsPopup.vue");

/** 弹出窗口子页面 */
const Settings = () => import("../views/Settings.vue");
const LicenseManager = () => import("../views/LicenseManager.vue");
const LanguageManager = () => import("../views/LanguageManager.vue");
const LogViewer = () => import("../views/LogViewer.vue");

/** 主窗口路由 */
const routes = [
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
      { path: "list", name: "SettingsList", component: Settings },    // 列表设置
      { path: "aur", name: "SettingsAur", component: Settings },     // AUR 设置
      { path: "checker", name: "SettingsChecker", component: Settings },  // 检查器设置
      { path: "backup", name: "SettingsBackup", component: Settings },    // 备份设置
      { path: "cache", name: "SettingsCache", component: Settings },      // 缓存设置
      { path: "proxy", name: "SettingsProxy", component: Settings },      // 代理设置
      { path: "log", name: "SettingsLog", component: Settings },          // 日志管理设置
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

// 前端诊断：记录路由导航时序（首屏白屏排查用）
router.beforeEach((to, from) => {
  feDebug("Router", `navigate ${from.path || "(start)"} -> ${to.path}`);
});
router.afterEach((to) => {
  feDebug("Router", `navigated ${to.path}`);
});

export default router;
