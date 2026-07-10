<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import type { Setting } from "../types";
import SettingsLogSection from "../components/SettingsLogSection.vue";
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
    // 更新设置缓存，使其他组件能立即获取最新值
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

    <div v-if="message" style="padding: 0.5rem 1rem; margin-bottom: 1rem; border-radius: 6px; background-color: rgba(76, 175, 125, 0.1); color: var(--success); font-size: 0.875rem">
      {{ message }}
    </div>

    <div v-if="category === 'general'" class="card" style="margin-bottom: 1rem">
      <h3 style="margin-bottom: 1rem">外观设置</h3>
      <div class="setting-row">
        <div class="setting-label">
          <strong>主题</strong>
          <span class="setting-desc">选择应用主题</span>
        </div>
        <div class="setting-input">
          <select :value="theme" @change="saveTheme(($event.target as HTMLSelectElement).value)" class="select-input">
            <option value="dark">深色</option>
            <option value="light">浅色</option>
          </select>
        </div>
      </div>
      <div class="setting-row">
        <div class="setting-label">
          <strong>字体大小</strong>
          <span class="setting-desc">调整界面文字大小</span>
        </div>
        <div class="setting-input">
          <select :value="fontSize" @change="saveFontSize(($event.target as HTMLSelectElement).value)" class="select-input">
            <option value="12">小 (12px)</option>
            <option value="14">默认 (14px)</option>
            <option value="16">大 (16px)</option>
            <option value="18">特大 (18px)</option>
          </select>
        </div>
      </div>
    </div>

    <div v-if="loading" class="card">加载中...</div>

    <div v-else-if="filteredSettings.length === 0 && category !== 'general'" class="card">
      <p style="color: var(--text-secondary)">暂无设置项</p>
    </div>

    <div v-else-if="category === 'log'">
      <SettingsLogSection />
    </div>

    <div v-else-if="filteredSettings.length > 0" class="card">
      <div v-for="s in filteredSettings" :key="s.key" class="setting-row">
        <div class="setting-label">
          <strong>{{ s.description || s.key }}</strong>
          <span class="setting-desc">{{ s.key }}</span>
        </div>
        <div class="setting-input password-wrapper" v-if="isTokenKey(s.key)">
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
        <div class="setting-input" v-else>
          <input
            type="text"
            :value="s.value"
            @change="(e) => saveSetting(s.key, (e.target as HTMLInputElement).value)"
            class="text-input"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--border);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-label {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

.setting-desc {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.setting-input {
  flex-shrink: 0;
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

.select-input option {
  background-color: var(--bg-primary);
  color: var(--text-primary);
}
</style>
