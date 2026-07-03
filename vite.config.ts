// Vite 构建配置
// 导入 Vite 的 defineConfig 工具函数，用于类型安全的配置
import { defineConfig } from "vite";
// 导入 Vite Vue 插件，用于编译 .vue 单文件组件
import vue from "@vitejs/plugin-vue";

// 从环境变量中获取 Tauri 开发主机地址
// 此变量由 Tauri CLI 在开发模式下设置，用于移动端或局域网访问
const host = process.env.TAURI_DEV_HOST;

// 使用异步工厂函数导出配置（支持异步插件初始化）
export default defineConfig(async () => ({
  // 插件列表：使用 Vue 3 插件
  plugins: [vue()],
  // 清除屏幕：设为 false 保留 Vite 启动信息
  clearScreen: false,
  // 开发服务器配置
  server: {
    port: 1420,             // Tauri 开发默认端口
    strictPort: true,       // 端口被占用时直接报错而非自动递增
    host: host || false,    // 指定主机地址（TAURI_DEV_HOST 时监听所有网卡）
    // HMR (热模块替换) 配置：仅在指定主机时启用 WebSocket
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,       // HMR WebSocket 端口，与 dev 服务器端口分开
        }
      : undefined,
    // 文件监听配置：排除 src-tauri 目录以减少不必要的重新编译
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
