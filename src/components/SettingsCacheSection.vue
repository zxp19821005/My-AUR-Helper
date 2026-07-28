<!--
  SettingsCacheSection.vue - 缓存目录设置组件

  功能：
  - 通过 settings 表配置缓存目录路径
  - 支持编辑三个缓存目录：系统缓存、paru 缓存、yay 缓存
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const message = ref("");
const loading = ref(false);
const cacheDirs = ref({
  system: "",
  paru: "",
  yay: "",
});
const editing = ref({
  system: false,
  paru: false,
  yay: false,
});
const tempValues = ref({
  system: "",
  paru: "",
  yay: "",
});

onMounted(async () => {
  await loadCacheDirs();
});

async function loadCacheDirs() {
  try {
    const systemDir = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_system" });
    cacheDirs.value.system = systemDir?.value || "/var/cache/pacman/pkg";
    
    const paruDir = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_paru" });
    cacheDirs.value.paru = paruDir?.value || "";
    
    const yayDir = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_yay" });
    cacheDirs.value.yay = yayDir?.value || "";
  } catch (e) {
    message.value = "加载缓存目录失败: " + String(e);
  }
}

function startEdit(type: "system" | "paru" | "yay") {
  editing.value[type] = true;
  tempValues.value[type] = cacheDirs.value[type];
}

function cancelEdit(type: "system" | "paru" | "yay") {
  editing.value[type] = false;
  tempValues.value[type] = "";
}

async function saveEdit(type: "system" | "paru" | "yay") {
  const key = type === "system" ? "cache_dir_system" : 
              type === "paru" ? "cache_dir_paru" : 
              "cache_dir_yay";
  loading.value = true;
  try {
    await invoke("set_setting", {
      key,
      value: tempValues.value[type],
    });
    cacheDirs.value[type] = tempValues.value[type];
    editing.value[type] = false;
    message.value = "保存成功";
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "保存失败: " + String(e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="card">
    <h3 style="margin-bottom: 1rem">缓存目录配置</h3>
    <p style="color: var(--text-secondary); font-size: 0.8125rem; margin-bottom: 1rem">
      配置 AUR 助手的缓存目录路径。留空则不使用该缓存目录。
    </p>

    <div v-if="message" class="message">{{ message }}</div>

    <div class="cache-dir-row">
      <label class="cache-dir-label">系统缓存</label>
      <template v-if="editing.system">
        <input v-model="tempValues.system" class="text-input" style="flex: 1" placeholder="/var/cache/pacman/pkg" />
        <button class="btn btn-primary btn-sm" @click="saveEdit('system')" :disabled="loading">保存</button>
        <button class="btn btn-secondary btn-sm" @click="cancelEdit('system')">取消</button>
      </template>
      <template v-else>
        <span class="cache-dir-path">{{ cacheDirs.system }}</span>
        <button class="btn-icon btn-icon-info" @click="startEdit('system')" title="编辑">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
        </button>
      </template>
    </div>

    <div class="cache-dir-row">
      <label class="cache-dir-label">paru 缓存</label>
      <template v-if="editing.paru">
        <input v-model="tempValues.paru" class="text-input" style="flex: 1" placeholder="~/.cache/paru/clone" />
        <button class="btn btn-primary btn-sm" @click="saveEdit('paru')" :disabled="loading">保存</button>
        <button class="btn btn-secondary btn-sm" @click="cancelEdit('paru')">取消</button>
      </template>
      <template v-else>
        <span class="cache-dir-path">{{ cacheDirs.paru || "未配置" }}</span>
        <button class="btn-icon btn-icon-info" @click="startEdit('paru')" title="编辑">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
        </button>
      </template>
    </div>

    <div class="cache-dir-row">
      <label class="cache-dir-label">yay 缓存</label>
      <template v-if="editing.yay">
        <input v-model="tempValues.yay" class="text-input" style="flex: 1" placeholder="~/.config/yay" />
        <button class="btn btn-primary btn-sm" @click="saveEdit('yay')" :disabled="loading">保存</button>
        <button class="btn btn-secondary btn-sm" @click="cancelEdit('yay')">取消</button>
      </template>
      <template v-else>
        <span class="cache-dir-path">{{ cacheDirs.yay || "未配置" }}</span>
        <button class="btn-icon btn-icon-info" @click="startEdit('yay')" title="编辑">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
        </button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.message {
  padding: 0.5rem 1rem;
  margin-bottom: 1rem;
  border-radius: 6px;
  background-color: rgba(76, 175, 125, 0.1);
  color: var(--success);
  font-size: 0.875rem;
}

.cache-dir-row {
  padding: 0.75rem 0;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.cache-dir-row:last-child {
  border-bottom: none;
}

.cache-dir-label {
  min-width: 100px;
  font-weight: 500;
  color: var(--text-primary);
}

.cache-dir-path {
  color: var(--text-secondary);
  font-size: 0.8125rem;
  font-family: monospace;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.text-input {
  padding: 0.375rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
}

.btn-sm {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
}

.btn-icon-info { color: var(--text-secondary); }
.btn-icon-info:hover { color: var(--accent); }
</style>