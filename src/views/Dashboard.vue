<!--
  Dashboard.vue - 仪表盘页面

  功能：
  - 显示软件包统计信息（总数、已最新、需更新）
  - 显示代理源数量
  - 显示系统缓存概览
  - 提供快速操作入口

  数据来源：
  - packages store: 软件包列表
  - get_proxies: 代理源列表
  - scan_caches: 系统缓存信息
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";                        // Vue 核心 API
import { useRouter } from "vue-router";                      // 路由 API：用于页面跳转
import { usePackageStore } from "../stores/packages";         // 软件包状态 Store
import { invoke } from "@tauri-apps/api/core";                // Tauri IPC 调用
import type { ProxyInfo, DetectedCache } from "../types";     // 类型定义

const router = useRouter();         // 路由实例，用于快速操作按钮导航
const pkgStore = usePackageStore(); // 软件包 Store 实例

/** 代理源数量 - 从后端获取的代理源列表长度 */
const proxyCount = ref(0);

/** 系统缓存列表 - 存储扫描到的所有系统缓存信息 */
const caches = ref<DetectedCache[]>([]);

/**
 * 组件挂载时加载数据
 * - 获取软件包列表（通过 Store 的 fetchPackages 方法）
 * - 获取代理源数量（调用后端 get_proxies 命令）
 * - 扫描系统缓存（调用后端 scan_caches 命令）
 */
onMounted(async () => {
  await pkgStore.fetchPackages();
  try {
    const proxies = await invoke<ProxyInfo[]>("get_proxies");
    proxyCount.value = proxies.length;
  } catch { /* 忽略代理获取错误 */ }
  try {
    caches.value = await invoke<DetectedCache[]>("scan_caches");
  } catch { /* 忽略缓存扫描错误 */ }
});

/** 统计数据计算函数 - 从 Store 中计算各项统计指标 */
const stats = {
  /** 软件包总数 */
  total: () => pkgStore.packages.length,
  /** 已最新的软件包数量 - is_outdated 为 false 的包 */
  updated: () => pkgStore.packages.filter((p) => !p.is_outdated).length,
  /** 需要更新的软件包数量 - is_outdated 为 true 的包 */
  outdated: () => pkgStore.packages.filter((p) => p.is_outdated).length,
};

/**
 * 格式化文件大小
 * 将字节数转换为人类可读的格式（B/KB/MB/GB）
 * @param bytes - 字节数
 * @returns 格式化后的字符串（如 "1.5 GB"）
 */
function formatSizeUnits(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(1) + " " + units[i];
}
</script>

<template>
  <div>
    <!-- 统计卡片网格 - 使用可点击卡片展示关键指标 -->
    <div class="dashboard-grid">
      <!-- 总包数卡片 - 点击跳转到软件管理页面 -->
      <div class="card stat-card" @click="router.push('/packages')">
        <div class="stat-number">{{ stats.total() }}</div>
        <div class="stat-label">总包数</div>
      </div>
      <!-- 已最新卡片 - 绿色数字，点击跳转到软件管理 -->
      <div class="card stat-card" @click="router.push('/packages')">
        <div class="stat-number" style="color: var(--success)">{{ stats.updated() }}</div>
        <div class="stat-label">已最新</div>
      </div>
      <!-- 有更新卡片 - 橙色数字，点击跳转到软件管理 -->
      <div class="card stat-card" @click="router.push('/packages')">
        <div class="stat-number" style="color: var(--warning)">{{ stats.outdated() }}</div>
        <div class="stat-label">有更新</div>
      </div>
      <!-- 代理源数量卡片 - 点击跳转到代理管理 -->
      <div class="card stat-card" @click="router.push('/proxy')">
        <div class="stat-number">{{ proxyCount }}</div>
        <div class="stat-label">代理源</div>
      </div>
      <!-- 系统缓存数量卡片 - 点击跳转到缓存管理 -->
      <div class="card stat-card" @click="router.push('/cache')">
        <div class="stat-number">{{ caches.length }}</div>
        <div class="stat-label">系统缓存</div>
      </div>
    </div>

    <!-- 系统缓存概览区域 - 显示各包管理器的缓存详情 -->
    <div v-if="caches.length" class="card" style="margin-top: 1.5rem">
      <h3>系统缓存概览</h3>
      <div style="margin-top: 0.75rem; display: flex; gap: 1rem; flex-wrap: wrap">
        <div v-for="c in caches" :key="c.cache_type" style="flex: 1; min-width: 180px">
          <strong>{{ c.cache_type.toUpperCase() }}</strong>
          <div style="font-size: 0.8125rem; margin-top: 0.25rem">
            {{ c.package_count }} 个包 · {{ formatSizeUnits(c.total_size_bytes) }}
          </div>
        </div>
      </div>
    </div>

    <!-- 快速操作区域 - 提供常用功能的一键跳转按钮 -->
    <div class="card" style="margin-top: 1.5rem">
      <h3>快速操作</h3>
      <div style="display: flex; gap: 1rem; margin-top: 1rem; flex-wrap: wrap">
        <button class="btn btn-primary" @click="router.push('/packages')">软件管理</button>
        <button class="btn btn-outline" @click="router.push('/backup')">备份管理</button>
        <button class="btn btn-outline" @click="router.push('/cache')">缓存管理</button>
        <button class="btn btn-outline" @click="router.push('/proxy')">代理管理</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 统计卡片网格 - 响应式网格布局 */
.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 1rem;
}

/* 统计卡片 - 可点击，悬停有上浮效果 */
.stat-card {
  cursor: pointer;
  text-align: center;
  transition: transform 0.2s;
}

.stat-card:hover {
  transform: translateY(-2px);
}

/* 统计数字 - 大号加粗显示 */
.stat-number {
  font-size: 2.5rem;
  font-weight: 700;
}

/* 统计标签 - 灰色小字说明 */
.stat-label {
  color: var(--text-secondary);
  margin-top: 0.25rem;
  font-size: 0.875rem;
}
</style>
