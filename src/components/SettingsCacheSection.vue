<!--
  SettingsCacheSection.vue - 缓存目录设置组件

  功能：
  - 通过 settings 表配置缓存目录路径
  - 支持启用/禁用切换
  - 支持编辑、删除、添加缓存目录

  依赖组件：
  - SettingsCard: 通用设置卡片组件

  注意：使用 useCacheDirs composable 管理目录状态，避免重复代码
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useCacheDirs, getDefaultCacheKey } from "../composables/useCacheDirs";
import SettingsCard from "./SettingsCard.vue";

const { cacheDirs, loading, message, load, saveCustom, showMessage } = useCacheDirs();

const editingIndex = ref<number | null>(null);
const tempValues = ref({ name: "", path: "", is_enabled: true });
const showAddForm = ref(false);
const newCacheDir = ref({ name: "", path: "", is_enabled: true });

onMounted(async () => {
  await load();
});

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
    const key = getDefaultCacheKey(index, cacheDirs.value);

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
      await saveCustom();
    }

    await load();
    editingIndex.value = null;
    showMessage("保存成功");
  } catch (e) {
    message.value = "保存失败: " + String(e);
  } finally {
    loading.value = false;
  }
}

async function toggleEnabled(index: number) {
  loading.value = true;
  try {
    const dir = cacheDirs.value[index];

    if (dir.is_default) {
      const key = getDefaultCacheKey(index, cacheDirs.value);
      const enabledKey = `${key}_enabled`;

      dir.is_enabled = !dir.is_enabled;
      await invoke("set_setting", {
        key: enabledKey,
        value: String(dir.is_enabled),
      });
    } else {
      dir.is_enabled = !dir.is_enabled;
      await saveCustom();
    }

    showMessage(dir.is_enabled ? "已启用" : "已禁用");
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
      const key = getDefaultCacheKey(index, cacheDirs.value);
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
      await saveCustom();
    }

    await load();
    showMessage("删除成功");
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

    await saveCustom();

    newCacheDir.value = { name: "", path: "", is_enabled: true };
    showAddForm.value = false;
    showMessage("添加成功");
  } catch (e) {
    message.value = "添加失败: " + String(e);
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <SettingsCard title="缓存目录配置" description="配置 AUR 助手的缓存目录路径。启用的目录将被扫描以查找缓存的软件包。">
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
  </SettingsCard>
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

.text-input:focus {
  border-color: var(--accent);
  outline: none;
}

.btn {
  padding: 0.5rem 1rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  cursor: pointer;
  font-size: 0.8125rem;
  transition: all 0.15s;
  white-space: nowrap;
}

.btn-sm {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
}

.btn-primary {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--bg-card);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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

.btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0.25rem;
  border-radius: 4px;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-icon-info { color: var(--text-secondary); }
.btn-icon-info:hover { color: var(--accent); background-color: var(--hover-bg, rgba(128,128,128,0.1)); }
.btn-icon-danger { color: var(--text-secondary); }
.btn-icon-danger:hover { color: #e74c3c; background-color: rgba(231,76,60,0.1); }
</style>
