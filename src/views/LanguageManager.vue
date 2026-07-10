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
import { ref, onMounted, inject } from "vue";            // Vue 核心 API
import { invoke } from "@tauri-apps/api/core";        // Tauri IPC 调用
import type { Language } from "../types";             // 编程语言类型定义
import DataTable from "../components/DataTable.vue";   // 通用数据表格组件
import type { Column } from "../components/DataTable.vue";

/** 编程语言列表 - 存储所有已配置的编程语言 */
const languages = ref<Language[]>([]);

/** 消息提示 - 操作结果反馈信息 */
const message = ref("");

/** 编辑模式状态 - 标识是否正在显示添加表单 */
const editing = ref(false);

/** 搜索关键词 */
const searchQuery = ref("");

/** 从设置中获取每页行数 */
const getListPageSize = inject<(key?: string) => Promise<number>>("getListPageSize", () => Promise.resolve(50));
const pageSize = ref(50);

/** 表单数据 - 添加编程语言时使用的各字段输入值 */
const editName = ref("");          // 语言名称
const editExts = ref("");          // 文件扩展名（逗号分隔）
const editBuildSys = ref("");      // 构建系统
const editBuildCmd = ref("");      // 构建命令

/** 表格列配置 */
const columns: Column[] = [
  { key: "name", title: "名称", width: "150px" },
  { key: "file_extensions", title: "文件扩展名", width: "180px" },
  { key: "build_system", title: "构建系统", width: "120px" },
  { key: "build_command", title: "构建命令" },
  { key: "name", title: "操作", width: "100px", align: "center" },
];

/** 组件挂载时加载编程语言列表 */
onMounted(async () => {
  pageSize.value = await getListPageSize("list_page_size_language");
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
        file_extensions: editExts.value.trim() || null,
        build_system: editBuildSys.value.trim() || null,
        build_command: editBuildCmd.value.trim() || null,
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
  editExts.value = "";
  editBuildSys.value = "";
  editBuildCmd.value = "";
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
        <!-- 文件扩展名输入 -->
        <label>
          文件扩展名
          <input type="text" v-model="editExts" class="text-input" placeholder="如: .rs,.toml" />
        </label>
        <!-- 构建系统输入 -->
        <label>
          构建系统
          <input type="text" v-model="editBuildSys" class="text-input" placeholder="如: cargo" />
        </label>
        <!-- 构建命令输入 -->
        <label>
          构建命令
          <input type="text" v-model="editBuildCmd" class="text-input" placeholder="如: cargo build" />
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
      :searchFields="['name', 'file_extensions', 'build_system', 'build_command']"
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

/* 文本输入框样式 */
.text-input {
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
}

/* 搜索输入框 */
.search-input {
  width: 200px;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
}

/* 小号按钮 */
.btn-sm {
  padding: 0.25rem 0.75rem;
  font-size: 0.75rem;
}
</style>
