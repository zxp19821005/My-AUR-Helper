<!--
  MemoryCacheSettingsForm.vue - 内存缓存配置表单子组件

  功能：
  - 渲染内存缓存的各项配置项（启用开关、条目上限、有效期、写盘周期、写入目录）
  - 使用草稿模型，编辑仅修改本地 draft

  依赖：
  - SettingsCard / SettingRow
-->
<script setup lang="ts">
import { computed } from "vue";
import { useSettingsDraft } from "../../composables/useSettingsDraft";
import * as settingsApi from "@/api/settings";
import SettingsCard from "./SettingsCard.vue";
import SettingRow from "./SettingRow.vue";

interface MemoryCacheSettings {
  memory_cache_enabled: string;
  memory_cache_size: string;
  memory_cache_ttl: string;
  memory_cache_write_interval: string;
  memory_cache_dir: string;
}

const { draft, commit } = useSettingsDraft<MemoryCacheSettings>({
  memory_cache_enabled: "true",
  memory_cache_size: "100",
  memory_cache_ttl: "300",
  memory_cache_write_interval: "60",
  memory_cache_dir: "",
});

const defaultCacheDir = "~/.config/com.zxp19821005.aur-helper/cache";
const displayCacheDir = computed(() => draft.value.memory_cache_dir || defaultCacheDir);

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
  }
}

defineExpose({ load });
</script>

<template>
  <SettingsCard v-if="true" title="内存缓存管理设置" description="将常用数据（系统设置、License、编程语言等）缓存到内存，减少数据库查询。修改后点击右下角「保存设置」才生效。">
    <SettingRow label="启用内存缓存" description="关闭后所有读取直接访问数据库，行为与未启用缓存时一致">
      <label class="toggle-switch">
        <input
          type="checkbox"
          :checked="draft.memory_cache_enabled === 'true'"
          @change="draft.memory_cache_enabled = ($event.target as HTMLInputElement).checked ? 'true' : 'false'"
        />
        <span>{{ draft.memory_cache_enabled === "true" ? "已启用" : "已禁用" }}</span>
      </label>
    </SettingRow>

    <SettingRow label="缓存条目上限" description="内存缓存条目数上限，超出后按最近最少使用（LRU）淘汰">
      <select v-model="draft.memory_cache_size" class="select-input">
        <option value="10">10 条</option>
        <option value="50">50 条</option>
        <option value="100">100 条（默认）</option>
        <option value="500">500 条</option>
        <option value="1000">1000 条</option>
      </select>
    </SettingRow>

    <SettingRow label="缓存有效期" description="超过有效期后自动回源数据库刷新缓存">
      <select v-model="draft.memory_cache_ttl" class="select-input">
        <option value="0">永不过期</option>
        <option value="60">1 分钟</option>
        <option value="300">5 分钟（默认）</option>
        <option value="900">15 分钟</option>
        <option value="1800">30 分钟</option>
        <option value="3600">1 小时</option>
      </select>
    </SettingRow>

    <SettingRow label="缓存写入周期" description="定时将内存缓存写入磁盘的间隔，关闭后仅在退出应用时写盘">
      <select v-model="draft.memory_cache_write_interval" class="select-input">
        <option value="0">关闭定时写盘</option>
        <option value="30">30 秒</option>
        <option value="60">1 分钟（默认）</option>
        <option value="300">5 分钟</option>
        <option value="900">15 分钟</option>
      </select>
    </SettingRow>

    <SettingRow label="缓存写入目录" :description="`留空则使用默认目录: ${displayCacheDir}`">
      <input
        v-model="draft.memory_cache_dir"
        class="text-input"
        placeholder="留空使用默认目录"
      />
    </SettingRow>
  </SettingsCard>
</template>

<style scoped>
.toggle-switch {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  color: var(--text-primary);
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

.text-input:focus {
  border-color: var(--accent);
  outline: none;
}
</style>
