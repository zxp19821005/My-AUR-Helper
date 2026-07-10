<!--
  LanguageManager.vue - 编程语言管理页面

  功能：
  - 显示编程语言列表（使用通用 DataTable 组件）
  - 支持添加编程语言
  - 支持删除编程语言

  数据来源：
  - get_languages: 获取编程语言列表
  - upsert_language: 添加/更新编程语言
  - delete_language: 删除编程语言
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Language } from "../types";
import DataTable from "../components/DataTable.vue";
import type { Column } from "../components/DataTable.vue";
import { useSettingsStore } from "../stores/settings";

const settingsStore = useSettingsStore();
const languages = ref<Language[]>([]);
const message = ref("");
const editing = ref(false);
const searchQuery = ref("");
const pageSize = ref(50);

const editName = ref("");
const editShortName = ref("");

/** 表格列配置 */
const columns: Column[] = [
  { key: "name", title: "名称", width: "150px" },
  { key: "short_name", title: "简称", width: "100px" },
  { key: "name", title: "操作", width: "100px", align: "center" },
];

onMounted(async () => {
  pageSize.value = await settingsStore.getSettingNumber("list_page_size_language", 50);
  await loadLanguages();
});

/** 加载编程语言列表 - 调用后端获取所有编程语言 */
async function loadLanguages() {
  try {
    languages.value = await invoke<Language[]>("get_languages");
  } catch (e) {
    message.value = "加载失败: " + String(e);
  }
}

/**
 * 保存编程语言
 * 验证名称不为空后调用后端 upsert_language 命令添加新语言
 * 保存成功后重置表单并刷新列表
 */
async function saveLanguage() {
  if (!editName.value.trim()) return;
  try {
    await invoke("upsert_language", {
      language: {
        id: null,
        name: editName.value.trim(),
        short_name: editShortName.value.trim() || null,
      },
    });
    message.value = "已保存";
    editing.value = false;
    resetForm();
    await loadLanguages();
    setTimeout(() => (message.value = ""), 2000);
  } catch (e) {
    message.value = "保存失败: " + String(e);
  }
}

/**
 * 删除编程语言
 * 二次确认后调用后端 delete_language 命令删除指定语言
 * @param name - 要删除的语言名称
 */
async function deleteLanguage(name: string) {
  if (!confirm(`确定删除 "${name}"？`)) return;
  try {
    await invoke("delete_language", { name });
    await loadLanguages();
  } catch (e) {
    message.value = "删除失败: " + String(e);
  }
}

/** 开始添加模式 - 重置表单并显示添加编辑器 */
function startAdd() {
  resetForm();
  editing.value = true;
}

/** 重置表单 - 清空所有编辑字段 */
function resetForm() {
  editName.value = "";
  editShortName.value = "";
}
</script>

<template>
  <div>
    <!-- 消息提示区域 -->
    <div v-if="message" class="card" style="margin-bottom: 1rem; border-color: var(--accent)">
      {{ message }}
    </div>

    <!-- 操作按钮和统计区域 -->
    <div style="display: flex; gap: 0.5rem; margin-bottom: 1rem; align-items: center; flex-wrap: wrap">
      <span style="color: var(--text-secondary); font-size: 0.875rem">
        总计: {{ languages.length }}
      </span>
      <button class="btn btn-primary" @click="startAdd">添加语言</button>
      <input
        type="text"
        v-model="searchQuery"
        placeholder="搜索语言..."
        class="search-input"
      />
    </div>

    <!-- 添加/编辑表单 - 显示在表格上方 -->
    <div v-if="editing" class="card" style="margin-bottom: 1rem">
      <h3>添加编程语言</h3>
      <div class="form-grid">
        <!-- 语言名称输入 -->
        <label>
          名称
          <input type="text" v-model="editName" class="text-input" placeholder="如: Rust" />
        </label>
        <!-- 简称输入 -->
        <label>
          简称
          <input type="text" v-model="editShortName" class="text-input" placeholder="如: rs" />
        </label>
      </div>
      <!-- 表单操作按钮 -->
      <div style="display: flex; gap: 0.5rem; margin-top: 1rem">
        <button class="btn btn-primary" @click="saveLanguage">保存</button>
        <button class="btn btn-outline" @click="editing = false">取消</button>
      </div>
    </div>

    <!-- 编程语言表格 - 使用 DataTable 组件 -->
    <DataTable
      :columns="columns"
      :data="languages"
      :pageSize="pageSize"
      :searchQuery="searchQuery"
      :searchFields="['name', 'short_name']"
      :showIndex="true"
      emptyText="暂无编程语言数据"
    >
      <template #cell-name="{ row }">
        <button class="btn btn-danger btn-sm" @click="deleteLanguage(row.name)">删除</button>
      </template>
    </DataTable>
  </div>
</template>

<style scoped>
/* 表单网格 - 两列布局 */
.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  margin-top: 0.75rem;
}

.form-grid label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.8125rem;
  color: var(--text-secondary);
}
</style>