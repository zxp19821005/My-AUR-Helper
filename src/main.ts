/**
 * main.ts - 应用入口文件
 *
 * 功能：
 * - 创建 Vue 应用实例
 * - 注册 Pinia 状态管理
 * - 注册 Vue Router
 * - 应用保存的客户端设置（主题、字体大小）
 */
import { createApp } from "vue";          // Vue 核心库，用于创建应用实例
import { createPinia } from "pinia";      // Pinia 状态管理库，集中管理应用全局状态
import App from "./App.vue";              // 根组件，应用的顶层入口
import router from "./router";            // Vue Router 路由配置，管理页面导航
import "./assets/variables.css";
import "./assets/base.css";
import "./assets/components.css";
import "./assets/modal.css";
import "./assets/forms.css";

// 应用保存的客户端设置
// 这些设置存储在 localStorage 中，在应用启动时应用到文档
const savedTheme = localStorage.getItem("app-theme") || "dark";          // 从 localStorage 读取保存的主题，默认为深色
const savedFontSize = localStorage.getItem("app-font-size") || "14";    // 从 localStorage 读取保存的字体大小，默认为 14px
document.documentElement.setAttribute("data-theme", savedTheme);        // 设置 HTML 根元素的 data-theme 属性以应用主题
document.documentElement.style.fontSize = savedFontSize + "px";         // 设置 HTML 根元素的字体大小

// 创建 Vue 应用实例
const app = createApp(App);

// 注册插件
app.use(createPinia()); // 状态管理 - 注册 Pinia 用于全局状态管理
app.use(router);        // 路由 - 注册 Vue Router 用于页面导航

// 挂载应用 - 将 Vue 应用挂载到 DOM 中的 #app 元素
app.mount("#app");
