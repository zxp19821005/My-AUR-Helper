<!--
  SettingsLogSection.vue - 日志管理设置组件

  功能：
  - 配置单个日志文件大小上限
  - 配置保留的日志文件数量
  - 实时应用日志设置

  依赖组件：
  - SettingsCard: 通用设置卡片组件
  - SettingRow: 通用设置行组件
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import SettingsCard from "./SettingsCard.vue";
import SettingRow from "./SettingRow.vue";

const logMaxSize = ref("10485760");
const logMaxFiles = ref("7");
const loading = ref(true);

onMounted(async () => {
  try {
    const settings = await invoke<{ key: string; value: string }[]>("get_settings");
    const size = settings.find((s: any) => s.key === "log_max_size");
    const files = settings.find((s: any) => s.key === "log_max_files");
    if (size) logMaxSize.value = size.value;
    if (files) logMaxFiles.value = files.value;
  } catch { /* ignore */ }
  loading.value = false;
});

async function saveLogSize(value: string) {
  logMaxSize.value = value;
  try {
    await invoke("set_setting", { key: "log_max_size", value });
    await invoke("apply_log_settings", { maxSize: parseInt(value), maxFiles: parseInt(logMaxFiles.value) });
  } catch { /* ignore */ }
}

async function saveLogCount(value: string) {
  logMaxFiles.value = value;
  try {
    await invoke("set_setting", { key: "log_max_files", value });
    await invoke("apply_log_settings", { maxSize: parseInt(logMaxSize.value), maxFiles: parseInt(value) });
  } catch { /* ignore */ }
}
</script>

<template>
  <SettingsCard v-if="!loading" title="日志管理设置" description="配置日志文件的大小上限和保留数量。">
    <SettingRow label="单个日志文件大小上限" description="当日志文件超过此大小时自动轮转">
      <select
        :value="logMaxSize"
        @change="saveLogSize(($event.target as HTMLSelectElement).value)"
        class="select-input"
      >
        <option value="1048576">1 MB</option>
        <option value="5242880">5 MB</option>
        <option value="10485760">10 MB（默认）</option>
        <option value="20971520">20 MB</option>
        <option value="52428800">50 MB</option>
        <option value="104857600">100 MB</option>
      </select>
    </SettingRow>

    <SettingRow label="保留的日志文件数量" description="超过此数量时自动删除最旧的日志文件">
      <select
        :value="logMaxFiles"
        @change="saveLogCount(($event.target as HTMLSelectElement).value)"
        class="select-input"
      >
        <option value="3">3 个</option>
        <option value="5">5 个</option>
        <option value="7">7 个（默认）</option>
        <option value="14">14 个</option>
        <option value="30">30 个</option>
        <option value="60">60 个</option>
      </select>
    </SettingRow>
  </SettingsCard>
</template>

<style scoped>
.select-input {
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

.select-input:focus {
  border-color: var(--accent);
  outline: none;
}

.select-input option {
  background-color: var(--bg-primary);
  color: var(--text-primary);
}
</style>
