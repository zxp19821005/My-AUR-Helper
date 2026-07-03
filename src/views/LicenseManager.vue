<!--
  LicenseManager.vue - License 管理页面

  功能：
  - 显示 License 列表
  - 支持从 SPDX 同步 License
  - 支持搜索 License（按 SPDX ID 或完整名称）
  - 显示 OSI 批准和弃用状态

  数据来源：
  - get_licenses: 获取 License 列表
  - sync_licenses_from_spdx: 从 SPDX 同步
-->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";         // Vue 核心 API
import { invoke } from "@tauri-apps/api/core";           // Tauri IPC 调用
import type { License } from "../types";                 // License 类型定义

/** License 列表 - 存储从后端获取的所有 SPDX License */
const licenses = ref<License[]>([]);

/** 同步中状态 - 标识是否正在从 SPDX 同步数据 */
const syncing = ref(false);

/** 消息提示 - 操作结果反馈信息 */
const message = ref("");

/** 搜索关键词 - 用于过滤 License 列表的输入文本 */
const searchQuery = ref("");

/** 组件挂载时加载 License 列表 */
onMounted(() => loadLicenses());

/** 加载 License 列表 - 调用后端获取所有 License */
async function loadLicenses() {
  try {
    licenses.value = await invoke<License[]>("get_licenses");
  } catch (e) {
    message.value = "加载失败: " + String(e);
  }
}

/**
 * 从 SPDX 同步 License
 * 调用后端 sync_licenses_from_spdx 命令从 SPDX 官方源获取最新的 License 数据
 * 同步完成后自动刷新列表并显示同步结果
 */
async function syncFromSPDX() {
  syncing.value = true;
  message.value = "";
  try {
    const count = await invoke<number>("sync_licenses_from_spdx");
    message.value = `已同步 ${count} 个 SPDX License`;
    await loadLicenses();
  } catch (e) {
    message.value = "同步失败: " + String(e);
  } finally {
    syncing.value = false;
  }
}

/** 过滤后的 License 列表 - 根据搜索关键词实时过滤 */
const filtered = computed(() => {
  if (!searchQuery.value) return licenses.value;
  const q = searchQuery.value.toLowerCase();
  return licenses.value.filter(
    (l) =>
      l.spdx_id.toLowerCase().includes(q) ||       // 按 SPDX ID 搜索
      l.full_name.toLowerCase().includes(q)        // 按完整名称搜索
  );
});

/** OSI 批准的 License 数量 - 计算属性 */
const osiCount = computed(() => licenses.value.filter((l) => l.is_osi_approved).length);

/** 已弃用的 License 数量 - 计算属性 */
const deprecatedCount = computed(() => licenses.value.filter((l) => l.is_deprecated).length);
</script>

<template>
  <div>
    <!-- 消息提示区域 -->
    <div v-if="message" class="card" style="margin-bottom: 1rem; border-color: var(--accent)">
      {{ message }}
    </div>

    <!-- 操作按钮和统计信息区域 -->
    <div style="display: flex; gap: 0.5rem; margin-bottom: 1rem; align-items: center">
      <!-- 统计信息 -->
      <span style="color: var(--text-secondary); font-size: 0.875rem">
        总计: {{ licenses.length }} | OSI 批准: {{ osiCount }} | 已弃用: {{ deprecatedCount }}
      </span>
      <!-- 从 SPDX 同步按钮 -->
      <button class="btn btn-primary" @click="syncFromSPDX" :disabled="syncing">
        {{ syncing ? "同步中..." : "从 SPDX 同步" }}
      </button>
    </div>

    <!-- 搜索和列表区域 -->
    <div class="card">
      <!-- 搜索框 - 用于实时搜索过滤 License -->
      <div style="margin-bottom: 1rem">
        <input
          type="text"
          v-model="searchQuery"
          placeholder="搜索 License (SPDX ID / 名称)..."
          class="search-input"
        />
      </div>

      <!-- 无数据提示 -->
      <div v-if="licenses.length === 0" style="color: var(--text-secondary)">
        暂无 License 数据，请从 SPDX 同步。
      </div>

      <!-- License 卡片网格 - 显示每个 License 的详细信息 -->
      <div v-else class="license-grid">
        <div v-for="(lic, i) in filtered" :key="i" class="license-card">
          <div class="license-id">
            <strong>{{ lic.spdx_id }}</strong>
            <!-- 已弃用标签 -->
            <span v-if="lic.is_deprecated" class="badge-deprecated">已弃用</span>
            <!-- OSI 批准标签 -->
            <span v-if="lic.is_osi_approved" class="badge-osi">OSI</span>
          </div>
          <div class="license-name">{{ lic.full_name }}</div>
          <!-- SPDX URL 链接 -->
          <div v-if="lic.url" class="license-url">
            <a :href="lic.url" target="_blank" rel="noopener">{{ lic.url }}</a>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 搜索框样式 */
.search-input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 8px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
}

/* License 卡片网格 - 响应式网格布局 */
.license-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 0.75rem;
}

/* License 卡片 */
.license-card {
  padding: 0.75rem;
  border: 1px solid var(--border);
  border-radius: 8px;
}

/* License ID 行 - 水平排列 SPDX ID 和标签 */
.license-id {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

/* License 完整名称 */
.license-name {
  font-size: 0.8125rem;
  color: var(--text-secondary);
  margin-bottom: 0.25rem;
}

/* License URL - 单行省略 */
.license-url {
  font-size: 0.75rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.license-url a {
  color: var(--accent);
  text-decoration: none;
}

/* 已弃用标签 - 红色半透明背景 */
.badge-deprecated {
  font-size: 0.625rem;
  background-color: rgba(239, 83, 80, 0.15);
  color: var(--error);
  padding: 0.125rem 0.375rem;
  border-radius: 4px;
}

/* OSI 批准标签 - 绿色半透明背景 */
.badge-osi {
  font-size: 0.625rem;
  background-color: rgba(76, 175, 125, 0.15);
  color: var(--success);
  padding: 0.125rem 0.375rem;
  border-radius: 4px;
}
</style>
