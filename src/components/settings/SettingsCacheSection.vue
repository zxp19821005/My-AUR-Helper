<!--
  SettingsCacheSection.vue - 缓存目录设置组件

  功能：
  - 通过 settings 表配置缓存目录路径与启用状态
  - 采用草稿模型：增/删/改/启用切换仅修改本地 draft
  - 点击「保存设置」才整表写入数据库；「重置设置」撤销未保存修改

  注意：复用 useCacheDirs 的底层函数（loadCacheDirs/saveCustomCacheDirs/getDefaultCacheKey），
  不改动 composable 本身（CacheManager 仍在复用）。
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useSettingsStore } from "../../stores/settings";
import {
  loadCacheDirs,
  saveCustomCacheDirs,
  getDefaultCacheKey,
  type CacheDir,
} from "../../composables/useCacheDirs";
import { useSettingsDraft } from "../../composables/useSettingsDraft";
import { openConfirm as confirm } from "../../composables/useConfirm";
import SettingsCard from "./SettingsCard.vue";
import SettingsActionBar from "./SettingsActionBar.vue";
import { Icon } from "../../icons";
import StandardizedButton from "../base/StandardizedButton.vue";

const { draft, dirty, saving, reset, commit } = useSettingsDraft<CacheDir[]>([]);

const loading = ref(false);
const message = ref("");
const editingIndex = ref<number | null>(null);
/** 进入编辑时的行快照，用于「取消」单行撤销 */
const editBackup = ref<CacheDir | null>(null);
const showAddForm = ref(false);
const newCacheDir = ref({ name: "", path: "", is_enabled: true });

onMounted(load);

async function load() {
  loading.value = true;
  try {
    draft.value = await loadCacheDirs();
    commit();
  } catch (e) {
    message.value = "加载缓存目录失败: " + String(e);
  } finally {
    loading.value = false;
  }
}

async function handleSave() {
  saving.value = true;
  try {
    const settingsStore = useSettingsStore();
    // 自定义目录整表保存（内部走 store.setSetting）
    await saveCustomCacheDirs(draft.value);
    // 默认目录逐个保存 path + enabled（并行写入，避免串行 IPC）
    const writes: Promise<void>[] = [];
    for (let i = 0; i < draft.value.length; i++) {
      const d = draft.value[i];
      if (d.is_default) {
        const key = getDefaultCacheKey(i, draft.value);
        if (key) {
          writes.push(settingsStore.setSetting(key, d.path));
          writes.push(settingsStore.setSetting(`${key}_enabled`, String(d.is_enabled)));
        }
      }
    }
    await Promise.all(writes);
    commit();
    editingIndex.value = null;
    showMessage("已保存");
  } catch (e) {
    message.value = "保存失败: " + String(e);
  } finally {
    saving.value = false;
  }
}

function startEdit(index: number) {
  editingIndex.value = index;
  editBackup.value = JSON.parse(JSON.stringify(draft.value[index]));
}

/** 完成编辑：改动已写入 draft，关闭编辑态（不立即持久化） */
function finishEdit() {
  editingIndex.value = null;
}

/** 取消编辑：用快照恢复该行原始值 */
function cancelEdit() {
  if (editingIndex.value !== null && editBackup.value) {
    draft.value[editingIndex.value] = JSON.parse(
      JSON.stringify(editBackup.value),
    );
  }
  editingIndex.value = null;
}

function addCacheDir() {
  if (!newCacheDir.value.name || !newCacheDir.value.path) {
    message.value = "请填写名称和路径";
    return;
  }
  draft.value.push({
    name: newCacheDir.value.name,
    path: newCacheDir.value.path,
    is_enabled: newCacheDir.value.is_enabled,
    is_default: false,
  });
  newCacheDir.value = { name: "", path: "", is_enabled: true };
  showAddForm.value = false;
  showMessage("已添加（点击保存设置后写入）");
}

async function deleteCacheDir(index: number) {
  if (
    !(await confirm({ message: "确定要删除此缓存目录配置吗？（未保存前可用「重置设置」撤销）", variant: "danger" }))
  )
    return;
  draft.value.splice(index, 1);
  editingIndex.value = null;
}

function showMessage(msg: string) {
  message.value = msg;
  setTimeout(() => {
    if (message.value === msg) message.value = "";
  }, 2000);
}
</script>

<template>
  <div class="cache-section-root">
    <div v-if="message" class="message">{{ message }}</div>

    <SettingsCard title="缓存目录配置" description="配置 AUR 助手的缓存目录路径。启用的目录将被扫描以查找缓存的软件包。修改后点击右下角「保存设置」才写入。">
      <div v-for="(dir, index) in draft" :key="index" class="cache-dir-row">
        <label class="cache-dir-toggle">
          <input
            type="checkbox"
            :checked="dir.is_enabled"
            @change="dir.is_enabled = ($event.target as HTMLInputElement).checked"
          />
          <span>{{ dir.name }}</span>
        </label>
        <template v-if="editingIndex === index">
          <input v-model="dir.name" class="text-input" style="width: 120px" placeholder="名称" :disabled="dir.is_default" />
          <input v-model="dir.path" class="text-input" style="flex: 1" placeholder="路径" />
          <StandardizedButton variant="primary" size="sm" @click="finishEdit" :disabled="saving">完成</StandardizedButton>
          <StandardizedButton variant="secondary" size="sm" @click="cancelEdit" :disabled="saving">取消</StandardizedButton>
        </template>
        <template v-else>
          <span class="cache-dir-path">{{ dir.path }}</span>
          <button class="btn-icon btn-icon-warning" @click="startEdit(index)" title="编辑">
            <component :is="Icon.actionEdit" :size="14" />
          </button>
          <button class="btn-icon btn-icon-danger" @click="deleteCacheDir(index)" title="删除">
            <component :is="Icon.actionDelete" :size="14" />
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
        <StandardizedButton variant="primary" size="sm" @click="addCacheDir" :disabled="saving">添加</StandardizedButton>
        <StandardizedButton variant="secondary" size="sm" @click="showAddForm = false">取消</StandardizedButton>
      </div>

      <StandardizedButton v-if="!showAddForm" variant="outline" tone="neutral" style="margin-top: 1rem" @click="showAddForm = true">
        + 添加缓存目录
      </StandardizedButton>
    </SettingsCard>

    <SettingsActionBar
      :dirty="dirty"
      :saving="saving"
      @save="handleSave"
      @reset="reset"
    />
  </div>
</template>

<style scoped>
.cache-section-root {
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
