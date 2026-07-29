<!--
  Settings.vue - 设置页面主视图

  功能：
  - 根据路由参数显示不同分类的设置
  - 支持通用设置、列表设置、AUR设置、检查器设置、备份设置、缓存设置、代理设置、日志设置
  - 提供保存设置和重置功能

  使用组件：
  - StandardizedCard: 设置卡片容器
  - StandardizedInput: 输入框（支持密码显示/隐藏）
  - StandardizedSelect: 下拉选择框
  - StandardizedMessage: 消息提示
-->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { Eye, EyeOff } from "@lucide/vue";
import type { Setting } from "../types";
import SettingsLogSection from "../components/settings/SettingsLogSection.vue";
import SettingsCacheSection from "../components/settings/SettingsCacheSection.vue";
import SettingsProxySection from "../components/settings/SettingsProxySection.vue";
import AppearanceSettings from "../components/settings/AppearanceSettings.vue";
import { useSettingsStore } from "../stores/settings";
import StandardizedCard from "../components/base/StandardizedCard.vue";
import StandardizedInput from "../components/base/StandardizedInput.vue";
import StandardizedSelect from "../components/base/StandardizedSelect.vue";
import StandardizedMessage from "../components/base/StandardizedMessage.vue";

const route = useRoute();
const settingsStore = useSettingsStore();

const settings = ref<Setting[]>([]);
const loading = ref(false);
const message = ref("");
const passwordVisible = ref<Record<string, boolean>>({});

const theme = ref(localStorage.getItem("app-theme") || "dark");
const fontSize = ref(localStorage.getItem("app-font-size") || "14");

const categoryMap: Record<string, string> = {
  "/settings": "general",
  "/settings/list": "list",
  "/settings/aur": "aur",
  "/settings/checker": "checker",
  "/settings/backup": "backup",
  "/settings/cache": "cache",
  "/settings/proxy": "proxy",
  "/settings/log": "log",
};

const category = computed(() => categoryMap[route.path] || "general");

const categoryLabels: Record<string, string> = {
  general: "通用设置",
  list: "列表设置",
  aur: "AUR 设置",
  checker: "上游检查器设置",
  backup: "备份管理设置",
  cache: "缓存软件设置",
  proxy: "代理管理设置",
  log: "日志管理设置",
};

const filteredSettings = computed(() =>
  settings.value.filter((s) => s.category === category.value)
);

onMounted(async () => {
  await loadAll();
  applySettings();
});

async function loadAll() {
  loading.value = true;
  try {
    settings.value = await invoke<Setting[]>("get_settings");
  } catch (e) {
    message.value = "加载失败: " + String(e);
  } finally {
    loading.value = false;
  }
}

async function saveSetting(key: string, value: string) {
  try {
    await invoke("set_setting", { key, value });
    const idx = settings.value.findIndex((s) => s.key === key);
    if (idx >= 0) {
      settings.value[idx] = { ...settings.value[idx], value };
    }
    await settingsStore.refreshSetting(key);
    message.value = "已保存";
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "保存失败: " + String(e);
  }
}

function togglePassword(key: string) {
  passwordVisible.value[key] = !passwordVisible.value[key];
}

function isTokenKey(key: string): boolean {
  return key.includes("token");
}

function inputType(s: Setting): string {
  if (!isTokenKey(s.key)) return "text";
  return passwordVisible.value[s.key] ? "text" : "password";
}
</script>

<template>
  <div class="settings-container">
    <!-- 消息提示 -->
    <StandardizedMessage
      v-if="message"
      type="success"
      :message="message"
      :duration="2000"
      @close="message = ''"
    />

    <!-- 页面标题 -->
    <h2 class="settings-title">{{ categoryLabels[category] || category }}</h2>

    <!-- 加载中 -->
    <StandardizedCard v-if="loading" title="加载中">
      <p class="loading-text">正在加载设置...</p>
    </StandardizedCard>

    <!-- 通用设置 -->
    <AppearanceSettings
      v-else-if="category === 'general'"
      v-model:theme="theme"
      v-model:font-size="fontSize"
    />

    <!-- 日志设置 -->
    <SettingsLogSection v-else-if="category === 'log'" />

    <!-- 缓存设置 -->
    <SettingsCacheSection v-else-if="category === 'cache'" />

    <!-- 代理设置 -->
    <SettingsProxySection v-else-if="category === 'proxy'" />

    <!-- 暂无设置 -->
    <StandardizedCard
      v-else-if="filteredSettings.length === 0 && category !== 'general'"
      title="暂无设置"
    >
      <p class="empty-text">当前分类下暂无设置项</p>
    </StandardizedCard>

    <!-- 动态设置 -->
    <StandardizedCard
      v-else-if="filteredSettings.length > 0"
      :title="categoryLabels[category] || category"
    >
      <div
        v-for="s in filteredSettings"
        :key="s.key"
        class="setting-row"
      >
        <div class="setting-info">
          <h4>{{ s.description || s.key }}</h4>
          <p>{{ s.key }}</p>
        </div>
        <div class="setting-control">
          <!-- Token 类型输入框（带密码显示/隐藏） -->
          <template v-if="isTokenKey(s.key)">
            <div class="password-wrapper">
              <StandardizedInput
                :type="inputType(s) as any"
                :modelValue="s.value"
                size="md"
                @update:modelValue="(val) => saveSetting(s.key, val)"
              />
              <button
                class="toggle-password"
                @click="togglePassword(s.key)"
                type="button"
                :title="passwordVisible[s.key] ? '隐藏' : '显示'"
              >
                <Eye v-if="passwordVisible[s.key]" :size="18" />
                <EyeOff v-else :size="18" />
              </button>
            </div>
          </template>
          <!-- 普通输入框 -->
          <template v-else>
            <StandardizedInput
              type="text"
              :modelValue="s.value"
              size="md"
              @update:modelValue="(val) => saveSetting(s.key, val)"
            />
          </template>
        </div>
      </div>
    </StandardizedCard>
  </div>
</template>

<style scoped>
.settings-container {
  width: 100%;
  min-width: 0;
}

.settings-title {
  margin-bottom: 1.5rem;
  font-size: 1.25rem;
}

.loading-text,
.empty-text {
  color: var(--text-secondary);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--border);
  gap: 1rem;
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-info h4 {
  margin: 0 0 0.25rem 0;
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--text-primary);
}

.setting-info p {
  margin: 0;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.setting-control {
  flex-shrink: 0;
  min-width: 240px;
}

.password-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.password-wrapper :deep(.standardized-input) {
  padding-right: 2.5rem;
}

.toggle-password {
  position: absolute;
  right: 0.5rem;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  padding: 0.25rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.15s;
}

.toggle-password:hover {
  color: var(--text-primary);
  background-color: var(--hover-bg, rgba(128, 128, 128, 0.1));
}
</style>