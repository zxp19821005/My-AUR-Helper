<!--
  AppearanceSettings.vue - 外观设置组件

  功能：
  - 主题选择（深色/浅色）
  - 字体大小选择
  - 采用草稿模型：修改仅改本地 draft，点击「保存设置」才写入 localStorage 并应用
  - 「重置设置」撤销未保存修改，恢复到上次保存值
-->
<script setup lang="ts">
import { onMounted } from "vue";
import { useSettingsDraft } from "../../composables/useSettingsDraft";
import SettingsCard from "./SettingsCard.vue";
import SettingsActionBar from "./SettingsActionBar.vue";
import StandardizedSelect from "../base/StandardizedSelect.vue";

interface Appearance {
  theme: string;
  fontSize: string;
}

const { draft, dirty, saving, reset, commit } = useSettingsDraft<Appearance>({
  theme: "dark",
  fontSize: "14",
});

onMounted(() => {
  draft.value = {
    theme: localStorage.getItem("app-theme") || "dark",
    fontSize: localStorage.getItem("app-font-size") || "14",
  };
  commit();
});

function apply() {
  document.documentElement.setAttribute("data-theme", draft.value.theme);
  document.documentElement.style.fontSize = draft.value.fontSize + "px";
}

async function handleSave() {
  saving.value = true;
  try {
    localStorage.setItem("app-theme", draft.value.theme);
    localStorage.setItem("app-font-size", draft.value.fontSize);
    apply();
    commit();
  } finally {
    saving.value = false;
  }
}

function handleReset() {
  reset();
  apply();
}
</script>

<template>
  <div class="appearance-section-root">
    <SettingsCard
      title="外观设置"
      description="选择应用主题和字体大小。修改后点击右下角「保存设置」才会应用。"
    >
      <div class="setting-row">
        <div class="setting-info">
          <h4>主题</h4>
          <p>选择应用主题</p>
        </div>
        <StandardizedSelect v-model="draft.theme" size="md">
          <option value="dark">深色</option>
          <option value="light">浅色</option>
        </StandardizedSelect>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <h4>字体大小</h4>
          <p>调整界面文字大小</p>
        </div>
        <StandardizedSelect v-model="draft.fontSize" size="md">
          <option value="12">小 (12px)</option>
          <option value="14">默认 (14px)</option>
          <option value="16">大 (16px)</option>
          <option value="18">特大 (18px)</option>
        </StandardizedSelect>
      </div>
    </SettingsCard>

    <SettingsActionBar
      :dirty="dirty"
      :saving="saving"
      @save="handleSave"
      @reset="handleReset"
    />
  </div>
</template>

<style scoped>
.appearance-section-root {
  width: 100%;
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
  flex: 0 0 200px;
  min-width: 120px;
  max-width: 300px;
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
</style>
