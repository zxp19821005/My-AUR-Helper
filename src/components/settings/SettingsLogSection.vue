<!--
  SettingsLogSection.vue - 日志管理设置组件

  功能：
  - 配置单个日志文件大小上限、保留数量、日志目录、文件名前缀
  - 采用草稿模型：编辑仅修改本地 draft，点击「保存设置」才写入并应用
  - 「重置设置」撤销未保存修改，恢复到上次保存值

  依赖组件：
  - SettingsCard / SettingRow / SettingsActionBar
-->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../../stores/settings";
import { useSettingsDraft } from "../../composables/useSettingsDraft";
import SettingsCard from "./SettingsCard.vue";
import SettingRow from "./SettingRow.vue";
import SettingsActionBar from "./SettingsActionBar.vue";

interface LogSettings {
  log_max_size: string;
  log_max_files: string;
  log_dir: string;
  log_prefix: string;
}

const defaultLogDir = "~/.config/com.zxp19821005.aur-helper/logs";

const { draft, dirty, saving, reset, commit } = useSettingsDraft<LogSettings>({
  log_max_size: "",
  log_max_files: "",
  log_dir: "",
  log_prefix: "",
});

const loading = ref(true);
const message = ref("");

const displayLogDir = computed(() => draft.value.log_dir || defaultLogDir);

onMounted(async () => {
  try {
    const settings = await invoke<{ key: string; value: string }[]>("get_settings");
    const get = (k: string) => settings.find((s) => s.key === k)?.value ?? "";
    draft.value = {
      log_max_size: get("log_max_size"),
      log_max_files: get("log_max_files"),
      log_dir: get("log_dir"),
      log_prefix: get("log_prefix"),
    };
    commit();
  } catch {
    /* ignore */
  } finally {
    loading.value = false;
  }
});

async function handleSave() {
  saving.value = true;
  try {
    // 通过 store 集中写入并更新缓存；4 个设置项并行保存
    const settingsStore = useSettingsStore();
    await Promise.all([
      settingsStore.setSetting("log_max_size", draft.value.log_max_size),
      settingsStore.setSetting("log_max_files", draft.value.log_max_files),
      settingsStore.setSetting("log_dir", draft.value.log_dir),
      settingsStore.setSetting("log_prefix", draft.value.log_prefix),
    ]);
    await invoke("apply_log_settings");
    commit();
    showMessage("已保存");
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
  <div class="log-section-root">
    <div v-if="message" class="message">{{ message }}</div>

    <SettingsCard v-if="!loading" title="日志管理设置" description="配置日志文件的大小上限、保留数量、存储目录和文件名前缀。修改后点击右下角「保存设置」才会写入。">
      <SettingRow label="单个日志文件大小上限" description="当日志文件超过此大小时自动轮转">
        <select v-model="draft.log_max_size" class="select-input">
          <option value="1048576">1 MB</option>
          <option value="5242880">5 MB</option>
          <option value="10485760">10 MB（默认）</option>
          <option value="20971520">20 MB</option>
          <option value="52428800">50 MB</option>
          <option value="104857600">100 MB</option>
        </select>
      </SettingRow>

      <SettingRow label="保留的日志文件数量" description="超过此数量时自动删除最旧的日志文件">
        <select v-model="draft.log_max_files" class="select-input">
          <option value="3">3 个</option>
          <option value="5">5 个</option>
          <option value="7">7 个（默认）</option>
          <option value="14">14 个</option>
          <option value="30">30 个</option>
          <option value="60">60 个</option>
        </select>
      </SettingRow>

      <SettingRow label="日志目录" :description="`留空则使用默认目录: ${displayLogDir}`">
        <input
          v-model="draft.log_dir"
          class="text-input"
          placeholder="留空使用默认目录"
        />
      </SettingRow>

      <SettingRow label="日志文件名前缀" description="完整文件名为 前缀-YYYY-MM-DD.log">
        <input
          v-model="draft.log_prefix"
          class="text-input"
          placeholder="applog"
        />
      </SettingRow>
    </SettingsCard>

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
.log-section-root {
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

.select-input {
  padding: 0.375rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  min-width: 240px;
  max-width: 100%;
  flex: 1 1 0;
  width: 0;
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

.text-input {
  padding: 0.375rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
  min-width: 240px;
  max-width: 100%;
  flex: 1 1 0;
  width: 0;
}

.text-input:focus {
  border-color: var(--accent);
  outline: none;
}
</style>
