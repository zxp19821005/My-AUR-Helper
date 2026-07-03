<!--
  ProxySettings.vue - 代理设置页面

  功能：
  - 显示代理源列表
  - 支持从 Greasyfork 获取代理源
  - 支持启用/禁用代理

  数据来源：
  - get_proxies: 获取代理列表
  - fetch_proxy_sources: 从 Greasyfork 获取代理源
  - set_proxy_active: 设置代理启用状态
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";                // Vue 核心 API
import { invoke } from "@tauri-apps/api/core";        // Tauri IPC 调用
import type { ProxyInfo } from "../types";            // 代理类型定义
import PageToolbar from "../components/PageToolbar.vue";

/** 代理列表 - 存储所有配置的代理源 */
const proxies = ref<ProxyInfo[]>([]);

/** 获取中状态 - 标识是否正在从 Greasyfork 获取代理源 */
const fetching = ref(false);

/** 组件挂载时获取代理列表 */
onMounted(async () => {
  await loadProxies();
});

/** 加载代理列表 - 调用后端获取所有代理源 */
async function loadProxies() {
  try {
    proxies.value = await invoke<ProxyInfo[]>("get_proxies");
  } catch { /* 忽略加载错误 */ }
}

/**
 * 从 Greasyfork 获取代理源
 * 调用后端 fetch_proxy_sources 命令从 Greasyfork 社区同步代理源列表
 * 获取成功后自动刷新代理列表
 */
async function fetchSources() {
  fetching.value = true;
  try {
    const count = await invoke<number>("fetch_proxy_sources");
    await loadProxies();
    alert(`获取到 ${count} 个代理源`);
  } catch (e) {
    alert("获取失败: " + String(e));
  } finally {
    fetching.value = false;
  }
}

/**
 * 切换代理启用状态
 * 调用后端更新代理激活状态，并在本地同步更新
 * @param proxy - 需要切换状态的代理对象
 */
async function toggleProxy(proxy: ProxyInfo) {
  try {
    await invoke("set_proxy_active", {
      proxyId: proxy.proxy_id,
      isActive: !proxy.is_active,
    });
    proxy.is_active = !proxy.is_active;  // 本地同步更新状态
  } catch (e) {
    alert("操作失败: " + String(e));
  }
}
</script>

<template>
  <div>
    <PageToolbar>
      <button class="btn btn-primary" @click="fetchSources" :disabled="fetching">
        {{ fetching ? "获取中..." : "从 Greasyfork 获取代理源" }}
      </button>
    </PageToolbar>

    <!-- 无数据提示 - 代理列表为空时显示 -->
    <div v-if="proxies.length === 0" class="card">
      <p style="color: var(--text-secondary)">暂无代理源，点击上方按钮从 Greasyfork 获取。</p>
    </div>

    <!-- 代理表格 - 显示代理详细信息和操作 -->
    <div class="card" v-else>
      <table class="proxy-table">
        <thead>
          <tr>
            <th>名称</th>
            <th>URL</th>
            <th>类型</th>
            <th>状态</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(p, idx) in proxies" :key="p.proxy_id ?? idx">
            <td>{{ p.proxy_name }}</td>
            <td style="max-width: 300px; overflow: hidden; text-overflow: ellipsis">{{ p.url }}</td>
            <td>{{ p.proxy_type }}</td>
            <td>
              <!-- 代理状态徽章 - 启用/禁用 -->
              <span class="status-badge" :class="p.is_active ? 'status-up_to_date' : 'status-manual'">
                {{ p.is_active ? "启用" : "禁用" }}
              </span>
            </td>
            <td>
              <!-- 启用/禁用切换按钮 -->
              <button class="btn btn-outline" style="padding: 0.25rem 0.5rem; font-size: 0.75rem"
                @click="toggleProxy(p)">
                {{ p.is_active ? "禁用" : "启用" }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
/* 代理表格样式 */
.proxy-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

.proxy-table th {
  text-align: left;
  padding: 0.75rem;
  color: var(--text-secondary);
  font-weight: 600;
  border-bottom: 1px solid var(--border);
}

.proxy-table td {
  padding: 0.75rem;
  border-bottom: 1px solid var(--border);
}
</style>
