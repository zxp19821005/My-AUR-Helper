<!--
  Settings.vue - 设置页面

  功能：
  - 显示和编辑应用设置
  - 支持主题和字体大小设置（客户端设置，存储在 localStorage）
  - 根据路由路径显示对应分类的设置项

  路由映射：
  - /settings -> general（通用设置）
  - /settings/aur -> aur（AUR 设置）
  - /settings/checker -> checker（上游检查器设置）
  - /settings/backup -> backup（备份管理设置）
  - /settings/cache -> cache（缓存软件设置）
  - /settings/proxy -> proxy（代理管理设置）

  数据来源：
  - get_settings: 获取所有设置
  - set_setting: 保存设置
  - localStorage: 主题和字体大小设置
-->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";         // Vue 核心 API
import { useRoute } from "vue-router";                   // 路由 API：获取当前设置分类
import { invoke } from "@tauri-apps/api/core";            // Tauri IPC 调用
import type { Setting } from "../types";                  // 设置类型定义

const route = useRoute();

/** 服务端设置列表 - 从后端数据库获取的所有设置项 */
const settings = ref<Setting[]>([]);

/** 加载中状态 - 标识是否正在加载设置数据 */
const loading = ref(false);

/** 消息提示 - 操作结果反馈信息 */
const message = ref("");

/** 主题设置 - 客户端设置，存储在 localStorage 中，默认深色 */
const theme = ref(localStorage.getItem("app-theme") || "dark");

/** 字体大小设置 - 客户端设置，存储在 localStorage 中，默认 14px */
const fontSize = ref(localStorage.getItem("app-font-size") || "14");

/** 路由路径到设置分类的映射 - 根据当前路由确定显示哪个分类的设置 */
const categoryMap: Record<string, string> = {
  "/settings": "general",
  "/settings/aur": "aur",
  "/settings/checker": "checker",
  "/settings/backup": "backup",
  "/settings/cache": "cache",
  "/settings/proxy": "proxy",
};

/** 当前设置分类 - 根据路由路径计算得出的设置分组 */
const category = computed(() => categoryMap[route.path] || "general");

/** 分类显示名称映射 - 将分类 ID 转换为中文显示名称 */
const categoryLabels: Record<string, string> = {
  general: "通用设置",
  aur: "AUR 设置",
  checker: "上游检查器设置",
  backup: "备份管理设置",
  cache: "缓存软件设置",
  proxy: "代理管理设置",
};

/** 过滤后的设置项 - 仅显示当前分类对应的设置项 */
const filteredSettings = computed(() =>
  settings.value.filter((s) => s.category === category.value)
);

/** 组件挂载时加载设置并应用客户端设置 */
onMounted(async () => {
  await loadAll();
  applySettings();
});

/** 加载所有设置 - 从后端获取全部设置项 */
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

/**
 * 保存服务端设置
 * 调用后端 set_setting 命令持久化设置值
 * @param key - 设置键
 * @param value - 设置值
 */
async function saveSetting(key: string, value: string) {
  try {
    await invoke("set_setting", { key, value });
    message.value = "已保存";
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "保存失败: " + String(e);
  }
}

/**
 * 保存主题设置
 * 将主题值写入 localStorage 并立即应用到页面
 * @param value - 主题值（"dark" 或 "light"）
 */
function saveTheme(value: string) {
  theme.value = value;
  localStorage.setItem("app-theme", value);
  applySettings();
}

/**
 * 保存字体大小设置
 * 将字体大小写入 localStorage 并立即应用到页面
 * @param value - 字体大小字符串（如 "12"、"14"）
 */
function saveFontSize(value: string) {
  fontSize.value = value;
  localStorage.setItem("app-font-size", value);
  applySettings();
}

/**
 * 应用客户端设置到文档
 * 通过修改 document.documentElement 的属性和样式来实时更新主题和字体
 */
function applySettings() {
  document.documentElement.setAttribute("data-theme", theme.value);
  document.documentElement.style.fontSize = fontSize.value + "px";
}
</script>

<template>
  <div>
    <!-- 页面标题 - 显示当前设置分类的中文名称 -->
    <h2 style="margin-bottom: 1.5rem; font-size: 1.25rem">{{ categoryLabels[category] || category }}</h2>

    <!-- 消息提示 - 保存成功或失败时的反馈信息 -->
    <div v-if="message" style="padding: 0.5rem 1rem; margin-bottom: 1rem; border-radius: 6px; background-color: rgba(76, 175, 125, 0.1); color: var(--success); font-size: 0.875rem">
      {{ message }}
    </div>

    <!-- 通用设置 - 外观配置区域（主题和字体大小），仅在通用分类下显示 -->
    <div v-if="category === 'general'" class="card" style="margin-bottom: 1rem">
      <h3 style="margin-bottom: 1rem">外观设置</h3>
      <div class="setting-row">
        <div class="setting-label">
          <strong>主题</strong>
          <span class="setting-desc">选择应用主题</span>
        </div>
        <div class="setting-input">
          <!-- 主题选择下拉框 -->
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
          <!-- 字体大小选择下拉框 -->
          <select :value="fontSize" @change="saveFontSize(($event.target as HTMLSelectElement).value)" class="select-input">
            <option value="12">小 (12px)</option>
            <option value="14">默认 (14px)</option>
            <option value="16">大 (16px)</option>
            <option value="18">特大 (18px)</option>
          </select>
        </div>
      </div>
    </div>

    <!-- 加载中提示 -->
    <div v-if="loading" class="card">加载中...</div>

    <!-- 无数据提示 - 当分类没有设置项且不是通用分类时显示 -->
    <div v-else-if="filteredSettings.length === 0 && category !== 'general'" class="card">
      <p style="color: var(--text-secondary)">暂无设置项</p>
    </div>

    <!-- 服务端设置列表 - 显示当前分类的所有可编辑设置项 -->
    <div v-else-if="filteredSettings.length > 0" class="card">
      <div v-for="s in filteredSettings" :key="s.key" class="setting-row">
        <div class="setting-label">
          <strong>{{ s.key }}</strong>
          <span v-if="s.description" class="setting-desc">{{ s.description }}</span>
        </div>
        <div class="setting-input">
          <!-- 文本输入框 - 修改后自动保存 -->
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
/* 设置行 - 左右布局，标签在左，输入框在右 */
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

/* 设置标签 - 垂直排列的标题和描述 */
.setting-label {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
}

/* 设置描述文字 - 小号灰色 */
.setting-desc {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

/* 设置输入框容器 - 禁止收缩 */
.setting-input {
  flex-shrink: 0;
}

/* 文本输入框和下拉选择框 - 统一样式 */
.text-input, .select-input {
  padding: 0.375rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  min-width: 240px;
  appearance: none;                /* 移除下拉框默认箭头 */
  -webkit-appearance: none;
  /* 自定义下拉箭头 SVG */
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%239a9cb8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 0.5rem center;
  padding-right: 1.75rem;
}

/* 下拉选项样式 */
.select-input option {
  background-color: var(--bg-primary);
  color: var(--text-primary);
}
</style>
