<!--
  Settings.vue - 设置页面主视图

  功能：
  - 根据路由参数显示不同分类的设置
  - 分发到各自的设置分区组件（各分区内部实现草稿 + 保存/重置）
  - 分类：通用(外观) / 列表 / AUR / 上游检查器 / 备份 / 缓存 / 代理 / 日志

  说明：
  - 列表/AUR/检查器/备份 为动态键值设置，由 SettingsDynamicSection 统一渲染
  - 其余为结构化的独立分区组件
-->
<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import SettingsDynamicSection from "../components/settings/SettingsDynamicSection.vue";
import SettingsLogSection from "../components/settings/SettingsLogSection.vue";
import SettingsCacheSection from "../components/settings/SettingsCacheSection.vue";
import SettingsMemoryCacheSection from "../components/settings/SettingsMemoryCacheSection.vue";
import SettingsProxySection from "../components/settings/SettingsProxySection.vue";
import AppearanceSettings from "../components/settings/AppearanceSettings.vue";

const route = useRoute();

const categoryMap: Record<string, string> = {
  "/settings": "general",
  "/settings/list": "list",
  "/settings/aur": "aur",
  "/settings/checker": "checker",
  "/settings/backup": "backup",
  "/settings/cache": "cache",
  "/settings/memory-cache": "memory_cache",
  "/settings/proxy": "proxy",
  "/settings/log": "log",
};

const category = computed(() => categoryMap[route.path] || "general");
</script>

<template>
  <div class="settings-container">
    <!-- 通用设置（外观） -->
    <AppearanceSettings v-if="category === 'general'" />

    <!-- 动态键值设置：列表 / AUR / 上游检查器 / 备份 -->
    <SettingsDynamicSection
      v-else-if="['list', 'aur', 'checker', 'backup'].includes(category)"
      :key="category"
      :category="category"
    />

    <!-- 日志管理设置 -->
    <SettingsLogSection v-else-if="category === 'log'" />

    <!-- 缓存目录设置 -->
    <SettingsCacheSection v-else-if="category === 'cache'" />

    <!-- 内存缓存设置 -->
    <SettingsMemoryCacheSection v-else-if="category === 'memory_cache'" />

    <!-- 代理管理设置 -->
    <SettingsProxySection v-else-if="category === 'proxy'" />
  </div>
</template>

<style scoped>
.settings-container {
  width: 100%;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}
</style>
