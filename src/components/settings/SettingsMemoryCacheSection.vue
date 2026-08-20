<!--
  SettingsMemoryCacheSection.vue - 内存缓存管理设置组件

  功能：
  - 配置内存缓存：启用开关、条目上限、有效期、写盘周期、写入目录（5 项设置）
  - 展示缓存运行状态（各域加载情况 / 条目数 / 过期时间 / 磁盘文件大小）
  - 提供「立即写盘」「清空缓存」操作按钮
  - 采用草稿模型：编辑仅修改本地 draft，点击「保存设置」才写入数据库

  依赖组件：
  - SettingsCard / SettingRow / SettingsActionBar
-->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useSettingsStore } from "../../stores/settings";
import { useSettingsDraft } from "../../composables/useSettingsDraft";
import * as settingsApi from "@/api/settings";
import * as cacheApi from "@/api/cache";
import type { MemoryCacheStats } from "@/types";
import SettingsCard from "./SettingsCard.vue";
import SettingRow from "./SettingRow.vue";
import SettingsActionBar from "./SettingsActionBar.vue";
import StandardizedButton from "../base/StandardizedButton.vue";

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
/** 缓存运行状态（null 表示未加载成功） */
const stats = ref<MemoryCacheStats | null>(null);
const statsLoading = ref(false);
const defaultCacheDir = "~/.config/com.zxp19821005.aur-helper/cache";

const displayCacheDir = computed(() => draft.value.memory_cache_dir || defaultCacheDir);

/** 格式化 Unix 秒时间戳为本地时间 */
function formatTs(ts: number | null): string {
  if (ts === null) return "-";
  if (ts === 0) return "永不过期";
  return new Date(ts * 1000).toLocaleString();
}

/** 格式化字节数为可读大小 */
function formatSize(bytes: number): string {
  if (bytes <= 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

async function loadStats() {
  statsLoading.value = true;
  try {
    stats.value = await cacheApi.getMemoryCacheStats();
  } catch (e) {
    message.value = "获取缓存状态失败: " + String(e);
  } finally {
    statsLoading.value = false;
  }
}

onMounted(async () => {
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
  await loadStats();
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
    await loadStats();
  } catch {
    showMessage("保存失败");
  } finally {
    saving.value = false;
  }
}

async function handleFlush() {
  try {
    const written = await cacheApi.flushMemoryCache();
    showMessage(`已写盘 ${written} 个缓存域`);
    await loadStats();
  } catch (e) {
    showMessage("写盘失败: " + String(e));
  }
}

async function handleClear() {
  try {
    await cacheApi.clearMemoryCache();
    showMessage("内存缓存与磁盘缓存文件已清空");
    await loadStats();
  } catch (e) {
    showMessage("清空失败: " + String(e));
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

    <SettingsCard v-if="!loading" title="内存缓存管理设置" description="将常用数据（系统设置、License、编程语言等）缓存到内存，减少数据库查询。修改后点击右下角「保存设置」才生效。">
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

    <SettingsCard title="缓存运行状态" description="当前内存缓存中各域的加载情况与磁盘持久化状态。">
      <div v-if="statsLoading" class="loading-text">正在读取缓存状态...</div>
      <div v-else-if="stats" class="stats-list">
        <div class="stats-summary">
          <span class="stat-chip" :class="stats.enabled ? 'chip-on' : 'chip-off'">
            {{ stats.enabled ? "缓存已启用" : "缓存已禁用" }}
          </span>
          <span class="stat-chip">条目总数 {{ stats.total_entries }}</span>
          <span class="stat-chip">上限 {{ stats.max_entries }}</span>
          <span class="stat-chip">{{ stats.cache_dir }}</span>
        </div>

        <table class="stats-table">
          <thead>
            <tr>
              <th>缓存域</th>
              <th>状态</th>
              <th>条目数</th>
              <th>创建时间</th>
              <th>过期时间</th>
              <th>持久化</th>
              <th>磁盘文件</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="d in stats.domains" :key="d.domain">
              <td>{{ d.label }}</td>
              <td>
                <span class="state-dot" :class="d.loaded ? 'dot-on' : 'dot-off'"></span>
                {{ d.loaded ? "已加载" : "未加载" }}
              </td>
              <td>{{ d.size }}</td>
              <td>{{ formatTs(d.created_at) }}</td>
              <td>{{ formatTs(d.expires_at) }}</td>
              <td>{{ d.persistent ? "是" : "否（仅内存）" }}</td>
              <td>{{ formatSize(d.file_size) }}</td>
            </tr>
          </tbody>
        </table>

        <div class="stats-actions">
          <StandardizedButton variant="primary" size="sm" @click="handleFlush">
            立即写盘
          </StandardizedButton>
          <StandardizedButton variant="danger" size="sm" @click="handleClear">
            清空缓存
          </StandardizedButton>
        </div>
      </div>
      <div v-else class="loading-text">缓存状态不可用</div>
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

.toggle-switch {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.875rem;
  color: var(--text-primary);
}

.loading-text {
  color: var(--text-secondary);
  font-size: 0.875rem;
}

.stats-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.stats-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.stat-chip {
  padding: 0.25rem 0.625rem;
  border-radius: 999px;
  font-size: 0.75rem;
  background-color: var(--bg-secondary, rgba(128, 128, 128, 0.12));
  color: var(--text-secondary);
  word-break: break-all;
}

.chip-on {
  background-color: rgba(76, 175, 125, 0.15);
  color: var(--success);
}

.chip-off {
  background-color: rgba(255, 152, 0, 0.15);
  color: var(--warning, #ff9800);
}

.stats-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8125rem;
}

.stats-table th,
.stats-table td {
  text-align: left;
  padding: 0.5rem 0.625rem;
  border-bottom: 1px solid var(--border);
  color: var(--text-primary);
  white-space: nowrap;
}

.stats-table th {
  color: var(--text-secondary);
  font-weight: 500;
}

.state-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 0.375rem;
}

.dot-on {
  background-color: var(--success, #4caf7d);
}

.dot-off {
  background-color: var(--text-muted, #9a9cb8);
}

.stats-actions {
  display: flex;
  gap: 0.5rem;
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
