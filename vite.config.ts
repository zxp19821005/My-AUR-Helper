import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
    // Vite 服务端预热：启动时预编译高频路由的模块依赖图。
    // 与浏览器端预加载有本质区别——server.warmup 在 Vite 服务端预编译模块，
    // 不争抢 WebKitGTK 浏览器主线程，不会导致雪崩。
    // 浏览器请求时直接返回已编译缓存的模块，消除首次路由切换的即时编译延迟。
    warmup: {
      clientFiles: [
        "./src/views/PackageList.vue",
        "./src/views/PackageDetail.vue",
        "./src/views/BackupManager.vue",
        "./src/views/CacheManager.vue",
        "./src/views/ProxySettings.vue",
      ],
    },
  },
}));