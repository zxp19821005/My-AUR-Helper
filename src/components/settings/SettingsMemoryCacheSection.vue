<!--
  SettingsMemoryCacheSection.vue - 内存缓存管理设置组件（重构版）

  功能：
  - 组合 MemoryCacheSettingsForm 和 MemoryCacheStatusPanel
  - 处理保存、清空等操作的消息反馈
  - 采用草稿模型：编辑仅修改本地 draft，点击「保存设置」才写入数据库

  依赖组件：
  - MemoryCacheSettingsForm: 配置表单
  - MemoryCacheStatusPanel: 状态展示面板
  - SettingsActionBar: 操作按钮栏
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useSettingsStore } from "../../stores/settings";
import { useSettingsDraft } from "../../composables/useSettingsDraft";
import * as settingsApi from "@/api/settings";
import MemoryCacheSettingsForm from "./MemoryCacheSettingsForm.vue";
import MemoryCacheStatusPanel from "./MemoryCacheStatusPanel.vue";
import SettingsActionBar from "./SettingsActionBar.vue";

interface MemoryCacheSettings {
  memory_cache_enabled: string;
  memory_cache_size: string;
  memory_cache_ttl: string;
  memory_cache_write_interval: string;
  memory_cache_dir: string;
}

const { draft, dirty, saving, reset, commit } = useSettingsDraft<MemoryCacheSettings>({
  memory_cache_enabled: "true",
  memory_cache_size: "100",
  memory_cache_ttl: "300",
  memory_cache_write_interval: "60",
  memory_cache_dir: "",
});

const loading = ref(true);
const message = ref("");
const statusRef = ref<InstanceType<typeof MemoryCacheStatusPanel> | null>(null);

async function load() {
  try {
    const settings = await settingsApi.getSettings();
    const get = (k: string) => settings.find((s) => s.key === k)?.value ?? "";
    draft.value = {
      memory_cache_enabled: get("memory_cache_enabled") || "true",
      memory_cache_size: get("memory_cache_size") || "100",
      memory_cache_ttl: get("memory_cache_ttl") || "300",
      memory_cache_write_interval: get("memory_cache_write_interval") || "60",
      memory_cache_dir: get("memory_cache_dir"),
    };
    commit();
  } catch {
    /* ignore */
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  load();
});

async function handleSave() {
  saving.value = true;
  try {
    const settingsStore = useSettingsStore();
    await Promise.all([
      settingsStore.setSetting("memory_cache_enabled", draft.value.memory_cache_enabled),
      settingsStore.setSetting("memory_cache_size", draft.value.memory_cache_size),
      settingsStore.setSetting("memory_cache_ttl", draft.value.memory_cache_ttl),
      settingsStore.setSetting("memory_cache_write_interval", draft.value.memory_cache_write_interval),
      settingsStore.setSetting("memory_cache_dir", draft.value.memory_cache_dir),
    ]);
    commit();
    showMessage("已保存（配置将在下次访问缓存时生效）");
    await statusRef.value?.loadStats();
  } catch {
    showMessage("保存失败");
  } finally {
    saving.value = false;
  }
}

function showMessage(text: string) {
  message.value = text;
  setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 3000);
}
</script>

<template>
  <div class="memory-cache-section-root">
    <div v-if="message" class="message">{{ message }}</div>

    <MemoryCacheSettingsForm ref="formRef" v-if="!loading" />

    <MemoryCacheStatusPanel ref="statusRef" />

    <SettingsActionBar
      v-if="!loading"
      :dirty="dirty"
      :saving="saving"
      @save="handleSave"
      @reset="reset"
    />
  </div>
</template>

<style scoped>
.memory-cache-section-root {
  width: 100%;
}

.message {
  padding: 0.5rem 1rem;
  margin-bottom: 1rem;
  border-radius: 6px;
  font-size: 0.875rem;
  background-color: rgba(76, 175, 125, 0.1);
  color: var(--success);
}
</style>
