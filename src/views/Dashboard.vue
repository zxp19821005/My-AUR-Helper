<!--
  Dashboard.vue - 仪表盘页面

  功能：
  - 显示软件包统计信息（总数、已最新、需更新）
  - 显示代理源数量
  - 提供快速操作入口

  数据来源：
  - packages store: 软件包列表
  - get_proxies: 代理源列表
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { usePackageStore } from "../stores/packages";
import { invoke } from "@tauri-apps/api/core";
import type { ProxyInfo } from "../types";
import { inject } from "vue";
import { FOOTER_KEY } from "../composables/footer";
import PageToolbar from "../components/PageToolbar.vue";

const router = useRouter();
const pkgStore = usePackageStore();
const footer = inject(FOOTER_KEY)!;

const proxyCount = ref(0);

onMounted(async () => {
  await pkgStore.fetchPackages();
  const total = pkgStore.packages.length;
  const outdated = pkgStore.packages.filter(p => p.is_outdated).length;
  footer.infoText = `总计: ${total}  |  已最新: ${total - outdated}  |  需更新: ${outdated}`;
  try {
    const proxies = await invoke<ProxyInfo[]>("get_proxies");
    proxyCount.value = proxies.length;
  } catch { /* 忽略代理获取错误 */ }
});

const stats = {
  total: () => pkgStore.packages.length,
  updated: () => pkgStore.packages.filter((p) => !p.is_outdated).length,
  outdated: () => pkgStore.packages.filter((p) => p.is_outdated).length,
};
</script>

<template>
  <div>
    <PageToolbar />

    <div class="dashboard-grid">
      <div class="card stat-card" @click="router.push('/packages')">
        <div class="stat-number">{{ stats.total() }}</div>
        <div class="stat-label">总包数</div>
      </div>
      <div class="card stat-card" @click="router.push('/packages')">
        <div class="stat-number" style="color: var(--success)">{{ stats.updated() }}</div>
        <div class="stat-label">已最新</div>
      </div>
      <div class="card stat-card" @click="router.push('/packages')">
        <div class="stat-number" style="color: var(--warning)">{{ stats.outdated() }}</div>
        <div class="stat-label">有更新</div>
      </div>
      <div class="card stat-card" @click="router.push('/proxy')">
        <div class="stat-number">{{ proxyCount }}</div>
        <div class="stat-label">代理源</div>
      </div>
    </div>

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
.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 1rem;
}

.stat-card {
  cursor: pointer;
  text-align: center;
  transition: transform 0.2s;
}

.stat-card:hover {
  transform: translateY(-2px);
}

.stat-number {
  font-size: 2.5rem;
  font-weight: 700;
}

.stat-label {
  color: var(--text-secondary);
  margin-top: 0.25rem;
  font-size: 0.875rem;
}
</style>
