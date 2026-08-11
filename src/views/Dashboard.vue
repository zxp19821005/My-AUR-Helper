<!--
  Dashboard.vue - 仪表盘页面

  功能：
  - 显示软件包统计信息（总数、已最新、需更新）
  - 显示代理源数量
  - 提供快速操作入口

  数据来源：
  - packages store: 软件包列表
  - get_proxies: 代理源列表

  使用组件：
  - StandardizedStatCard: 统计卡片
  - StandardizedButton: 操作按钮
  - PageToolbar: 页面工具栏
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { usePackageStore } from "../stores/packages";
import { invoke } from "@tauri-apps/api/core";
import type { ProxyInfo } from "../types";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedStatCard from "../components/base/StandardizedStatCard.vue";
import StandardizedButton from "../components/base/StandardizedButton.vue";
import { Icon } from "../icons";

const router = useRouter();
const pkgStore = usePackageStore();

const proxyCount = ref(0);

onMounted(async () => {
  await pkgStore.fetchPackages();
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

    <!-- 统计卡片网格 -->
    <div class="dashboard-grid">
      <StandardizedStatCard
        title="总包数"
        :value="stats.total()"
        :icon="Icon.statTotal"
        color="var(--accent)"
        clickable
        @click="router.push('/packages')"
      />

      <StandardizedStatCard
        title="已最新"
        :value="stats.updated()"
        :icon="Icon.statUpdated"
        color="var(--success)"
        clickable
        @click="router.push('/packages')"
      />

      <StandardizedStatCard
        title="有更新"
        :value="stats.outdated()"
        :icon="Icon.statOutdated"
        color="var(--warning)"
        clickable
        @click="router.push('/packages')"
      />

      <StandardizedStatCard
        title="代理源"
        :value="proxyCount"
        :icon="Icon.statProxy"
        color="var(--info)"
        clickable
        @click="router.push('/proxy')"
      />
    </div>

    <!-- 快速操作区域 -->
    <div class="card quick-actions-card">
      <h3 class="quick-actions-title">
        <component :is="Icon.navSettings" :size="18" />
        快速操作
      </h3>
      <div class="quick-actions-buttons">
        <StandardizedButton
          variant="primary"
          size="md"
          @click="router.push('/packages')"
        >
          <component :is="Icon.navPackages" :size="16" />
          软件管理
        </StandardizedButton>

        <StandardizedButton
          variant="outline"
          size="md"
          @click="router.push('/backup')"
        >
          <component :is="Icon.navBackup" :size="16" />
          备份管理
        </StandardizedButton>

        <StandardizedButton
          variant="outline"
          size="md"
          @click="router.push('/cache')"
        >
          <component :is="Icon.navCache" :size="16" />
          缓存管理
        </StandardizedButton>

        <StandardizedButton
          variant="outline"
          size="md"
          @click="router.push('/proxy')"
        >
          <component :is="Icon.navProxy" :size="16" />
          代理管理
        </StandardizedButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 1rem;
}

.quick-actions-card {
  margin-top: 1.5rem;
  padding: 1.25rem;
}

.quick-actions-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.quick-actions-buttons {
  display: flex;
  gap: 0.75rem;
  margin-top: 1rem;
  flex-wrap: wrap;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .dashboard-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 0.75rem;
  }

  .quick-actions-buttons {
    flex-direction: column;
  }

  .quick-actions-buttons > * {
    width: 100%;
  }
}
</style>