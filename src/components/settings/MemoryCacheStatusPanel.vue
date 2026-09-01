<!--
  MemoryCacheStatusPanel.vue - 内存缓存状态展示子组件

  功能：
  - 展示缓存运行状态（各域加载情况 / 条目数 / 过期时间 / 磁盘文件大小）
  - 提供「立即写盘」「清空缓存」操作按钮

  依赖：
  - SettingsCard / StandardizedButton
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import type { MemoryCacheStats } from "@/types";
import * as cacheApi from "@/api/cache";
import SettingsCard from "./SettingsCard.vue";
import StandardizedButton from "../base/StandardizedButton.vue";

/** 格式化 Unix 秒时间戳为本地时间 */
function formatTs(ts: number | null): string {
  if (ts === null) return "-";
  if (ts === 0) return "永不过期";
  return new Date(ts * 1000).toLocaleString();
}

/** 格式化字节数为可读大小 */
function formatSize(bytes: number): string {
  if (bytes <= 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

const stats = ref<MemoryCacheStats | null>(null);
const statsLoading = ref(false);

async function loadStats() {
  statsLoading.value = true;
  try {
    stats.value = await cacheApi.getMemoryCacheStats();
  } catch (e) {
    // errors handled by parent
  } finally {
    statsLoading.value = false;
  }
}

async function handleFlush() {
  try {
    await cacheApi.flushMemoryCache();
    window.dispatchEvent(new CustomEvent("cache-stats-updated"));
    await loadStats();
  } catch (e) {
    throw e;
  }
}

async function handleClear() {
  try {
    await cacheApi.clearMemoryCache();
    window.dispatchEvent(new CustomEvent("cache-stats-updated"));
    await loadStats();
  } catch (e) {
    throw e;
  }
}

onMounted(() => {
  loadStats();
});

defineExpose({ loadStats, handleFlush, handleClear });
</script>

<template>
  <SettingsCard title="缓存运行状态" description="当前内存缓存中各域的加载情况与磁盘持久化状态。">
    <div v-if="statsLoading" class="loading-text">正在读取缓存状态...</div>
    <div v-else-if="stats" class="stats-list">
      <div class="stats-summary">
        <span class="stat-chip" :class="stats.enabled ? 'chip-on' : 'chip-off'">
          {{ stats.enabled ? "缓存已启用" : "缓存已禁用" }}
        </span>
        <span class="stat-chip">条目总数 {{ stats.total_entries }}</span>
        <span class="stat-chip">上限 {{ stats.max_entries }}</span>
        <span class="stat-chip">{{ stats.cache_dir }}</span>
      </div>

      <table class="stats-table">
        <thead>
          <tr>
            <th>缓存域</th>
            <th>状态</th>
            <th>条目数</th>
            <th>创建时间</th>
            <th>过期时间</th>
            <th>持久化</th>
            <th>磁盘文件</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="d in stats.domains" :key="d.domain">
            <td>{{ d.label }}</td>
            <td>
              <span class="state-dot" :class="d.loaded ? 'dot-on' : 'dot-off'"></span>
              {{ d.loaded ? "已加载" : "未加载" }}
            </td>
            <td>{{ d.size }}</td>
            <td>{{ formatTs(d.created_at) }}</td>
            <td>{{ formatTs(d.expires_at) }}</td>
            <td>{{ d.persistent ? "是" : "否（仅内存）" }}</td>
            <td>{{ formatSize(d.file_size) }}</td>
          </tr>
        </tbody>
      </table>

      <div class="stats-actions">
        <StandardizedButton variant="primary" size="sm" @click="handleFlush">
          立即写盘
        </StandardizedButton>
        <StandardizedButton variant="danger" size="sm" @click="handleClear">
          清空缓存
        </StandardizedButton>
      </div>
    </div>
    <div v-else class="loading-text">缓存状态不可用</div>
  </SettingsCard>
</template>

<style scoped>
.loading-text {
  color: var(--text-secondary);
  font-size: 0.875rem;
}

.stats-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.stats-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.stat-chip {
  padding: 0.25rem 0.625rem;
  border-radius: 999px;
  font-size: 0.75rem;
  background-color: var(--bg-secondary, rgba(128, 128, 128, 0.12));
  color: var(--text-secondary);
  word-break: break-all;
}

.chip-on {
  background-color: rgba(76, 175, 125, 0.15);
  color: var(--success);
}

.chip-off {
  background-color: rgba(255, 152, 0, 0.15);
  color: var(--warning, #ff9800);
}

.stats-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8125rem;
}

.stats-table th,
.stats-table td {
  text-align: left;
  padding: 0.5rem 0.625rem;
  border-bottom: 1px solid var(--border);
  color: var(--text-primary);
  white-space: nowrap;
}

.stats-table th {
  color: var(--text-secondary);
  font-weight: 500;
}

.state-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 0.375rem;
}

.dot-on {
  background-color: var(--success, #4caf7d);
}

.dot-off {
  background-color: var(--text-muted, #9a9cb8);
}

.stats-actions {
  display: flex;
  gap: 0.5rem;
}
</style>
