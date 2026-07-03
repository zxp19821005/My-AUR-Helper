<!--
  CacheManager.vue - 缓存管理页面

  功能：
  - 扫描系统缓存
  - 显示缓存类型、路径、包数量和大小

  数据来源：
  - scan_caches: 扫描系统缓存
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";                // Vue 核心 API
import { invoke } from "@tauri-apps/api/core";        // Tauri IPC 调用
import type { DetectedCache } from "../types";        // 缓存类型定义

/** 缓存列表 - 存储扫描到的所有系统缓存信息 */
const caches = ref<DetectedCache[]>([]);

/** 扫描中状态 - 标识是否正在扫描系统缓存 */
const scanning = ref(false);

/** 组件挂载时扫描缓存 */
onMounted(() => scanCaches());

/**
 * 扫描系统缓存
 * 调用后端 scan_caches 命令检测各包管理器的缓存目录和大小
 */
async function scanCaches() {
  scanning.value = true;
  try {
    caches.value = await invoke<DetectedCache[]>("scan_caches");
  } catch { /* 忽略扫描错误 */ }
  scanning.value = false;
}

/**
 * 格式化文件大小
 * 将字节数转换为人类可读的格式（B/KB/MB/GB）
 * @param bytes - 字节数
 * @returns 格式化后的字符串（如 "1.5 GB"）
 */
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(1) + " " + units[i];
}
</script>

<template>
  <div>
    <!-- 操作按钮区域 -->
    <div style="margin-bottom: 1rem">
      <!-- 扫描系统缓存按钮 -->
      <button class="btn btn-outline" @click="scanCaches" :disabled="scanning">
        {{ scanning ? "扫描中..." : "扫描系统缓存" }}
      </button>
    </div>

    <!-- 无数据提示 - 首次加载或无缓存时显示 -->
    <div v-if="caches.length === 0" class="card">
      <p style="color: var(--text-secondary)">暂无缓存数据，点击上方按钮扫描。</p>
    </div>

    <!-- 缓存卡片网格 - 每种包管理器一个卡片 -->
    <div v-else class="cache-grid">
      <div v-for="cache in caches" :key="cache.cache_type" class="card cache-card">
        <div class="cache-header">
          <strong>{{ cache.cache_type.toUpperCase() }}</strong>
        </div>
        <div class="cache-path">{{ cache.cache_path }}</div>
        <div class="cache-stats">
          <span>{{ cache.package_count }} 个包</span>
          <span>{{ formatBytes(cache.total_size_bytes) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 缓存卡片网格 - 响应式网格布局 */
.cache-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
}

/* 缓存卡片 - 悬停上浮效果 */
.cache-card {
  transition: transform 0.2s;
}

.cache-card:hover {
  transform: translateY(-2px);
}

/* 缓存类型头部 */
.cache-header {
  margin-bottom: 0.5rem;
}

/* 缓存路径 - 灰色小字，允许换行 */
.cache-path {
  font-size: 0.8125rem;
  color: var(--text-secondary);
  margin-bottom: 0.75rem;
  word-break: break-all;
}

/* 缓存统计信息 - 两端对齐 */
.cache-stats {
  display: flex;
  justify-content: space-between;
  font-size: 0.875rem;
  color: var(--accent);
}
</style>
