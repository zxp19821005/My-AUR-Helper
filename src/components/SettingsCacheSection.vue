<!--
  SettingsCacheSection.vue - 缓存目录设置组件

  功能：
  - 显示所有缓存目录配置
  - 支持启用/禁用切换
  - 支持编辑、删除、添加缓存目录
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CacheDir } from "../types";

const message = ref("");
const cacheDirs = ref<CacheDir[]>([]);
const editingCacheDir = ref<CacheDir | null>(null);
const showAddCacheDir = ref(false);
const newCacheDir = ref({ name: "", path: "", is_enabled: true });

onMounted(async () => {
  await loadCacheDirs();
});

async function loadCacheDirs() {
  try {
    cacheDirs.value = await invoke<CacheDir[]>("list_cache_dirs");
  } catch (e) {
    message.value = "加载缓存目录失败: " + String(e);
  }
}

async function addCacheDir() {
  if (!newCacheDir.value.name || !newCacheDir.value.path) {
    message.value = "请填写名称和路径";
    return;
  }
  try {
    await invoke("add_cache_dir", {
      name: newCacheDir.value.name,
      path: newCacheDir.value.path,
      isEnabled: newCacheDir.value.is_enabled,
    });
    await loadCacheDirs();
    newCacheDir.value = { name: "", path: "", is_enabled: true };
    showAddCacheDir.value = false;
    message.value = "添加成功";
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "添加失败: " + String(e);
  }
}

async function updateCacheDir(dir: CacheDir) {
  try {
    await invoke("update_cache_dir", {
      id: dir.id,
      name: dir.name,
      path: dir.path,
      isEnabled: dir.is_enabled,
    });
    await loadCacheDirs();
    editingCacheDir.value = null;
    message.value = "更新成功";
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "更新失败: " + String(e);
  }
}

async function deleteCacheDir(id: number) {
  if (!confirm("确定要删除此缓存目录配置吗？")) return;
  try {
    await invoke("delete_cache_dir", { id });
    await loadCacheDirs();
    message.value = "删除成功";
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "删除失败: " + String(e);
  }
}
</script>

<template>
  <div class="card">
    <h3 style="margin-bottom: 1rem">缓存目录配置</h3>
    <p style="color: var(--text-secondary); font-size: 0.8125rem; margin-bottom: 1rem">
      配置 AUR 助手的缓存目录路径。启用的目录将被扫描以查找缓存的软件包。
    </p>

    <div v-if="message" class="message">{{ message }}</div>

    <div v-for="dir in cacheDirs" :key="dir.id" class="cache-dir-row">
      <div class="cache-dir-info">
        <label class="cache-dir-toggle">
          <input type="checkbox" :checked="dir.is_enabled"
            @change="(e) => { dir.is_enabled = (e.target as HTMLInputElement).checked; updateCacheDir(dir); }" />
          <span>{{ dir.name }}</span>
        </label>
        <template v-if="editingCacheDir?.id === dir.id">
          <input v-model="editingCacheDir.name" class="text-input" style="width: 120px" placeholder="名称" />
          <input v-model="editingCacheDir.path" class="text-input" style="flex: 1" placeholder="路径" />
          <button class="btn btn-primary btn-sm" @click="updateCacheDir(editingCacheDir!)">保存</button>
          <button class="btn btn-secondary btn-sm" @click="editingCacheDir = null">取消</button>
        </template>
        <template v-else>
          <span class="cache-dir-path">{{ dir.path }}</span>
          <button class="btn-icon btn-icon-info" @click="editingCacheDir = { ...dir }" title="编辑">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
          </button>
          <button class="btn-icon btn-icon-danger" @click="deleteCacheDir(dir.id!)" title="删除">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
          </button>
        </template>
      </div>
    </div>

    <div v-if="showAddCacheDir" class="cache-dir-row" style="margin-top: 0.5rem">
      <div class="cache-dir-info">
        <label class="cache-dir-toggle">
          <input type="checkbox" v-model="newCacheDir.is_enabled" />
          <span>新增</span>
        </label>
        <input v-model="newCacheDir.name" class="text-input" style="width: 120px" placeholder="名称" />
        <input v-model="newCacheDir.path" class="text-input" style="flex: 1" placeholder="路径" />
        <button class="btn btn-primary btn-sm" @click="addCacheDir">添加</button>
        <button class="btn btn-secondary btn-sm" @click="showAddCacheDir = false">取消</button>
      </div>
    </div>

    <button v-if="!showAddCacheDir" class="btn btn-outline" style="margin-top: 1rem" @click="showAddCacheDir = true">
      + 添加缓存目录
    </button>
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
}

.cache-dir-row:last-child {
  border-bottom: none;
}

.cache-dir-info {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.cache-dir-toggle {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  min-width: 100px;
  cursor: pointer;
}

.cache-dir-toggle input[type="checkbox"] {
  cursor: pointer;
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

.btn-outline {
  background: none;
  border: 1px dashed var(--border);
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-size: 0.8125rem;
  transition: all 0.15s;
}

.btn-outline:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.btn-icon-info { color: var(--text-secondary); }
.btn-icon-info:hover { color: var(--accent); }
.btn-icon-danger { color: var(--text-secondary); }
.btn-icon-danger:hover { color: #e74c3c; }
</style>
