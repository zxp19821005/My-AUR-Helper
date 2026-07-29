<!--
  Settings.vue - 设置页面主视图

  功能：
  - 根据路由参数显示不同分类的设置
  - 支持通用设置、列表设置、AUR设置、检查器设置、备份设置、缓存设置、代理设置、日志设置
  - 提供保存设置和重置功能

  依赖组件：
  - SettingsCard: 通用设置卡片组件
  - SettingRow: 通用设置行组件
  - SettingsLogSection: 日志设置组件
  - SettingsCacheSection: 缓存设置组件
  - SettingsProxySection: 代理设置组件
-->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import type { Setting } from "../types";
import SettingsCard from "../components/SettingsCard.vue";
import SettingRow from "../components/SettingRow.vue";
import SettingsLogSection from "../components/SettingsLogSection.vue";
import SettingsCacheSection from "../components/SettingsCacheSection.vue";
import SettingsProxySection from "../components/SettingsProxySection.vue";
import { useSettingsStore } from "../stores/settings";

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

function saveTheme(value: string) {
  theme.value = value;
  localStorage.setItem("app-theme", value);
  applySettings();
}

function saveFontSize(value: string) {
  fontSize.value = value;
  localStorage.setItem("app-font-size", value);
  applySettings();
}

function applySettings() {
  document.documentElement.setAttribute("data-theme", theme.value);
  document.documentElement.style.fontSize = fontSize.value + "px";
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
  <div>
    <h2 style="margin-bottom: 1.5rem; font-size: 1.25rem">{{ categoryLabels[category] || category }}</h2>

    <div v-if="message" class="message message-success">
      {{ message }}
    </div>

    <!-- 加载中 -->
    <SettingsCard v-if="loading" title="加载中">
      <p style="color: var(--text-secondary)">正在加载设置...</p>
    </SettingsCard>

    <!-- 通用设置 -->
    <SettingsCard
      v-else-if="category === 'general'"
      title="外观设置"
      description="选择应用主题和字体大小"
    >
      <SettingRow label="主题" description="选择应用主题">
        <select :value="theme" @change="saveTheme(($event.target as HTMLSelectElement).value)" class="select-input">
          <option value="dark">深色</option>
          <option value="light">浅色</option>
        </select>
      </SettingRow>

      <SettingRow label="字体大小" description="调整界面文字大小">
        <select :value="fontSize" @change="saveFontSize(($event.target as HTMLSelectElement).value)" class="select-input">
          <option value="12">小 (12px)</option>
          <option value="14">默认 (14px)</option>
          <option value="16">大 (16px)</option>
          <option value="18">特大 (18px)</option>
        </select>
      </SettingRow>
    </SettingsCard>

    <!-- 日志设置 -->
    <SettingsLogSection v-else-if="category === 'log'" />

    <!-- 缓存设置 -->
    <SettingsCacheSection v-else-if="category === 'cache'" />

    <!-- 代理设置 -->
    <SettingsProxySection v-else-if="category === 'proxy'" />

    <!-- 暂无设置 -->
    <SettingsCard
      v-else-if="filteredSettings.length === 0 && category !== 'general'"
      title="暂无设置"
    >
      <p style="color: var(--text-secondary)">当前分类下暂无设置项</p>
    </SettingsCard>

    <!-- 动态设置 -->
    <SettingsCard
      v-else-if="filteredSettings.length > 0"
      :title="categoryLabels[category] || category"
    >
      <SettingRow
        v-for="s in filteredSettings"
        :key="s.key"
        :label="s.description || s.key"
        :description="s.key"
      >
        <!-- Token 类型输入框 -->
        <template v-if="isTokenKey(s.key)">
          <div class="password-wrapper">
            <input
              :type="inputType(s)"
              :value="s.value"
              @change="(e) => saveSetting(s.key, (e.target as HTMLInputElement).value)"
              class="text-input"
            />
            <button class="toggle-password" @click="togglePassword(s.key)" type="button" :title="passwordVisible[s.key] ? '隐藏' : '显示'">
              <svg v-if="passwordVisible[s.key]" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                <circle cx="12" cy="12" r="3"/>
              </svg>
              <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
                <line x1="1" y1="1" x2="23" y2="23"/>
              </svg>
            </button>
          </div>
        </template>
        <!-- 普通输入框 -->
        <template v-else>
          <input
            type="text"
            :value="s.value"
            @change="(e) => saveSetting(s.key, (e.target as HTMLInputElement).value)"
            class="text-input"
          />
        </template>
      </SettingRow>
    </SettingsCard>
  </div>
</template>

<style scoped>
.message {
  padding: 0.5rem 1rem;
  margin-bottom: 1rem;
  border-radius: 6px;
  font-size: 0.875rem;
}

.message-success {
  background-color: rgba(76, 175, 125, 0.1);
  color: var(--success);
}

.message-error {
  background-color: rgba(231, 76, 60, 0.1);
  color: #e74c3c;
}

.message-warning {
  background-color: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.password-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.password-wrapper .text-input {
  padding-right: 2.5rem;
}

.toggle-password {
  position: absolute;
  right: 0.25rem;
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
}

.toggle-password:hover {
  color: var(--text-primary);
  background-color: var(--hover-bg, rgba(128,128,128,0.1));
}

.text-input, .select-input {
  padding: 0.375rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  min-width: 240px;
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%239a9cb8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 0.5rem center;
  padding-right: 1.75rem;
}

.text-input:focus, .select-input:focus {
  border-color: var(--accent);
  outline: none;
}

.select-input option {
  background-color: var(--bg-primary);
  color: var(--text-primary);
}
</style>
