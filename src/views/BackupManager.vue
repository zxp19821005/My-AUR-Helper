<!--
  BackupManager.vue - 备份管理页面

  功能：
  - 扫描系统缓存
  - 显示检测到的缓存信息
  - 执行备份操作
  - 显示备份结果

  数据来源：
  - scan_caches: 扫描系统缓存
  - run_backup: 执行备份
  - get_setting: 获取备份目录设置
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";                    // Vue 核心 API
import { invoke } from "@tauri-apps/api/core";            // Tauri IPC 调用
import type { DetectedCache, BackupResult, Setting } from "../types";  // 类型定义

/** 备份结果 - 存储最近一次备份操作的结果 */
const result = ref<BackupResult | null>(null);

/** 运行中状态 - 标识是否正在执行备份 */
const loading = ref(false);

/** 扫描中状态 - 标识是否正在扫描缓存 */
const scanning = ref(false);

/** 备份目录 - 从设置中读取的备份目标路径 */
const backupDir = ref("/run/media/zxp/Backup/Linux/ZST");

/** 检测到的缓存列表 - 系统各包管理器的缓存信息 */
const detectedCaches = ref<DetectedCache[]>([]);

/**
 * 组件挂载时加载数据
 * - 从后端获取备份目录设置
 * - 扫描系统缓存
 */
onMounted(async () => {
  try {
    const setting = await invoke<Setting | null>("get_setting", { key: "backup_dir" });
    if (setting) backupDir.value = setting.value;
  } catch { /* 忽略获取设置错误 */ }
  await scanCaches();
});

/**
 * 扫描系统缓存
 * 调用后端 scan_caches 命令检测各包管理器的缓存信息
 */
async function scanCaches() {
  scanning.value = true;
  try {
    detectedCaches.value = await invoke<DetectedCache[]>("scan_caches");
  } catch { /* 忽略扫描错误 */ }
  scanning.value = false;
}

/**
 * 执行备份
 * 调用后端 run_backup 命令，将系统缓存备份到指定目录
 * 备份完成后显示结果信息
 */
async function runBackup() {
  loading.value = true;
  result.value = null;
  try {
    result.value = await invoke<BackupResult>("run_backup", { backupPath: backupDir.value });
  } catch (e) {
    result.value = { copied: 0, removed: 0, errors: [String(e)] };
  } finally {
    loading.value = false;
  }
}

/**
 * 格式化文件大小
 * 将字节数转换为人类可读的格式
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
    <div style="display: flex; gap: 0.5rem; margin-bottom: 1rem">
      <!-- 扫描系统缓存按钮 -->
      <button class="btn btn-outline" @click="scanCaches" :disabled="scanning">
        {{ scanning ? "扫描中..." : "扫描系统缓存" }}
      </button>
      <!-- 执行备份按钮 -->
      <button class="btn btn-primary" @click="runBackup" :disabled="loading">
        {{ loading ? "运行中..." : "执行备份" }}
      </button>
    </div>

    <!-- 备份目录信息卡片 -->
    <div class="card" style="margin-bottom: 1rem">
      <p style="color: var(--text-secondary); font-size: 0.875rem">
        备份目录: <strong>{{ backupDir }}</strong>
      </p>
    </div>

    <!-- 检测到的系统缓存区域 - 展示各包管理器缓存详情 -->
    <div v-if="detectedCaches.length" class="card" style="margin-bottom: 1rem">
      <h3>检测到的系统缓存</h3>
      <div style="margin-top: 0.5rem; display: flex; gap: 0.75rem; flex-wrap: wrap">
        <div v-for="cache in detectedCaches" :key="cache.cache_type"
          style="padding: 0.75rem; border: 1px solid var(--border); border-radius: 8px; flex: 1; min-width: 200px">
          <strong>{{ cache.cache_type.toUpperCase() }}</strong>
          <div style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.25rem">
            {{ cache.cache_path }}
          </div>
          <div style="font-size: 0.75rem; margin-top: 0.25rem">
            包数: {{ cache.package_count }} | 大小: {{ formatBytes(cache.total_size_bytes) }}
          </div>
        </div>
      </div>
    </div>

    <!-- 备份结果展示区域 -->
    <div v-if="result" class="card">
      <h3>备份结果</h3>
      <div style="margin-top: 0.5rem; font-size: 0.875rem">
        <span style="color: var(--success)">已复制: {{ result.copied }}</span>
        &nbsp;
        <span style="color: var(--warning)">已清理: {{ result.removed }}</span>
      </div>
      <!-- 错误信息列表 -->
      <div v-if="result.errors.length" style="color: var(--error); margin-top: 0.5rem">
        <div v-for="(err, i) in result.errors" :key="i">{{ err }}</div>
      </div>
    </div>
  </div>
</template>
