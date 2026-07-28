<!--
  SettingsCacheSection.vue - 缓存目录设置组件

  功能：
  - 通过 settings 表配置缓存目录路径
  - 支持启用/禁用切换
  - 支持编辑、删除、添加缓存目录
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface CacheDir {
  name: string;
  path: string;
  is_enabled: boolean;
  is_default: boolean;
}

const message = ref("");
const loading = ref(false);
const cacheDirs = ref<CacheDir[]>([]);
const editingIndex = ref<number | null>(null);
const tempValues = ref({ name: "", path: "", is_enabled: true });
const showAddForm = ref(false);
const newCacheDir = ref({ name: "", path: "", is_enabled: true });

onMounted(async () => {
  await loadCacheDirs();
});

async function loadCacheDirs() {
  try {
    const dirs: CacheDir[] = [];
    
    // 系统缓存（默认）
    const systemPath = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_system" });
    const systemEnabled = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_system_enabled" });
    dirs.push({
      name: "系统缓存",
      path: systemPath?.value || "/var/cache/pacman/pkg",
      is_enabled: systemEnabled?.value !== "false",
      is_default: true,
    });
    
    // paru 缓存（默认）
    const paruPath = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_paru" });
    const paruEnabled = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_paru_enabled" });
    if (paruPath?.value) {
      dirs.push({
        name: "paru 缓存",
        path: paruPath.value,
        is_enabled: paruEnabled?.value !== "false",
        is_default: true,
      });
    }
    
    // yay 缓存（默认）
    const yayPath = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_yay" });
    const yayEnabled = await invoke<{ value: string } | null>("get_setting", { key: "cache_dir_yay_enabled" });
    if (yayPath?.value) {
      dirs.push({
        name: "yay 缓存",
        path: yayPath.value,
        is_enabled: yayEnabled?.value !== "false",
        is_default: true,
      });
    }
    
    // 自定义缓存目录（从 cache_dirs_custom 读取）
    const customDirs = await invoke<{ value: string } | null>("get_setting", { key: "cache_dirs_custom" });
    if (customDirs?.value) {
      const customList: { name: string; path: string; is_enabled: boolean }[] = JSON.parse(customDirs.value);
      for (const dir of customList) {
        dirs.push({
          name: dir.name,
          path: dir.path,
          is_enabled: dir.is_enabled,
          is_default: false,
        });
      }
    }
    
    cacheDirs.value = dirs;
  } catch (e) {
    message.value = "加载缓存目录失败: " + String(e);
  }
}

async function saveCustomDirs() {
  const customDirs = cacheDirs.value
    .filter(d => !d.is_default)
    .map(d => ({ name: d.name, path: d.path, is_enabled: d.is_enabled }));
  
  try {
    await invoke("set_setting", {
      key: "cache_dirs_custom",
      value: JSON.stringify(customDirs),
    });
  } catch (e) {
    throw e;
  }
}

function startEdit(index: number) {
  editingIndex.value = index;
  const dir = cacheDirs.value[index];
  tempValues.value = {
    name: dir.name,
    path: dir.path,
    is_enabled: dir.is_enabled,
  };
}

function cancelEdit() {
  editingIndex.value = null;
  tempValues.value = { name: "", path: "", is_enabled: true };
}

async function saveEdit(index: number) {
  loading.value = true;
  try {
    const dir = cacheDirs.value[index];
    const key = getCacheKey(index, dir.is_default);
    
    if (dir.is_default) {
      await invoke("set_setting", {
        key,
        value: tempValues.value.path,
      });
      
      const enabledKey = `${key}_enabled`;
      await invoke("set_setting", {
        key: enabledKey,
        value: String(tempValues.value.is_enabled),
      });
    } else {
      dir.name = tempValues.value.name;
      dir.path = tempValues.value.path;
      dir.is_enabled = tempValues.value.is_enabled;
      await saveCustomDirs();
    }
    
    await loadCacheDirs();
    editingIndex.value = null;
    message.value = "保存成功";
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "保存失败: " + String(e);
  } finally {
    loading.value = false;
  }
}

function getCacheKey(index: number, isDefault: boolean): string {
  if (!isDefault) return "";
  
  const customDirsCount = cacheDirs.value.slice(0, index).filter(d => !d.is_default).length;
  const defaultIndex = index - customDirsCount;
  
  switch (defaultIndex) {
    case 0: return "cache_dir_system";
    case 1: return "cache_dir_paru";
    case 2: return "cache_dir_yay";
    default: return "";
  }
}

async function toggleEnabled(index: number) {
  loading.value = true;
  try {
    const dir = cacheDirs.value[index];
    
    if (dir.is_default) {
      const key = getCacheKey(index, true);
      const enabledKey = `${key}_enabled`;
      
      dir.is_enabled = !dir.is_enabled;
      await invoke("set_setting", {
        key: enabledKey,
        value: String(dir.is_enabled),
      });
    } else {
      dir.is_enabled = !dir.is_enabled;
      await saveCustomDirs();
    }
    
    message.value = dir.is_enabled ? "已启用" : "已禁用";
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "操作失败: " + String(e);
  } finally {
    loading.value = false;
  }
}

async function deleteCacheDir(index: number) {
  if (!confirm("确定要删除此缓存目录配置吗？")) return;
  
  loading.value = true;
  try {
    const dir = cacheDirs.value[index];
    
    if (dir.is_default) {
      const key = getCacheKey(index, true);
      await invoke("set_setting", {
        key,
        value: "",
      });
      
      const enabledKey = `${key}_enabled`;
      await invoke("set_setting", {
        key: enabledKey,
        value: "true",
      });
    } else {
      cacheDirs.value.splice(index, 1);
      await saveCustomDirs();
    }
    
    await loadCacheDirs();
    message.value = "删除成功";
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "删除失败: " + String(e);
  } finally {
    loading.value = false;
  }
}

async function addCacheDir() {
  if (!newCacheDir.value.name || !newCacheDir.value.path) {
    message.value = "请填写名称和路径";
    return;
  }
  
  loading.value = true;
  try {
    cacheDirs.value.push({
      name: newCacheDir.value.name,
      path: newCacheDir.value.path,
      is_enabled: newCacheDir.value.is_enabled,
      is_default: false,
    });
    
    await saveCustomDirs();
    
    newCacheDir.value = { name: "", path: "", is_enabled: true };
    showAddForm.value = false;
    message.value = "添加成功";
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "添加失败: " + String(e);
  } finally {
    loading.value = false;
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

    <div v-for="(dir, index) in cacheDirs" :key="index" class="cache-dir-row">
      <label class="cache-dir-toggle">
        <input type="checkbox" :checked="dir.is_enabled"
          @change="toggleEnabled(index)" />
        <span>{{ dir.name }}</span>
      </label>
      <template v-if="editingIndex === index">
        <input v-model="tempValues.name" class="text-input" style="width: 120px" placeholder="名称" :disabled="dir.is_default" />
        <input v-model="tempValues.path" class="text-input" style="flex: 1" placeholder="路径" />
        <button class="btn btn-primary btn-sm" @click="saveEdit(index)" :disabled="loading">保存</button>
        <button class="btn btn-secondary btn-sm" @click="cancelEdit">取消</button>
      </template>
      <template v-else>
        <span class="cache-dir-path">{{ dir.path }}</span>
        <button class="btn-icon btn-icon-info" @click="startEdit(index)" title="编辑">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
        </button>
        <button class="btn-icon btn-icon-danger" @click="deleteCacheDir(index)" title="删除">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
        </button>
      </template>
    </div>

    <div v-if="showAddForm" class="cache-dir-row" style="margin-top: 0.5rem">
      <label class="cache-dir-toggle">
        <input type="checkbox" v-model="newCacheDir.is_enabled" />
        <span>新增</span>
      </label>
      <input v-model="newCacheDir.name" class="text-input" style="width: 120px" placeholder="名称" />
      <input v-model="newCacheDir.path" class="text-input" style="flex: 1" placeholder="路径" />
      <button class="btn btn-primary btn-sm" @click="addCacheDir" :disabled="loading">添加</button>
      <button class="btn btn-secondary btn-sm" @click="showAddForm = false">取消</button>
    </div>

    <button v-if="!showAddForm" class="btn btn-outline" style="margin-top: 1rem" @click="showAddForm = true">
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
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.cache-dir-row:last-child {
  border-bottom: none;
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

.text-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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