<!--
  LanguageManager.vue - 编程语言管理页面

  功能：
  - 显示编程语言列表（名称、描述）
  - 支持搜索、分页
  - 支持添加、编辑、删除语言
  - 支持从 GitHub 同步

  使用组件：
  - StandardizedTable: 表格组件
  - StandardizedButton: 操作按钮
  - StandardizedMessage: 消息提示
  - StandardizedInput: 搜索输入框
  - StandardizedModal: 弹窗组件
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EnumProgrammingLanguage as ProgrammingLanguage } from "../types";
import LanguageFormModal from "../components/enum/LanguageFormModal.vue";
import { useSettingsStore } from "../stores/settings";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedButton from "../components/base/StandardizedButton.vue";
import StandardizedMessage from "../components/base/StandardizedMessage.vue";
import StandardizedInput from "../components/base/StandardizedInput.vue";

const settingsStore = useSettingsStore();
const languages = ref<ProgrammingLanguage[]>([]);
const syncing = ref(false);
const message = ref("");
const searchQuery = ref("");

const pageSize = ref(50);

const showModal = ref(false);
const modalMode = ref<"add" | "edit">("add");
const modalForm = ref({
  id: null as number | null,
  name: "",
  short_name: "",
});

onMounted(async () => {
  pageSize.value = await settingsStore.getSettingNumber("list_page_size_language", 50);
  await loadLanguages();
});

async function loadLanguages() {
  try {
    languages.value = await invoke<ProgrammingLanguage[]>("get_programming_languages");
  } catch (e) {
    message.value = "加载失败: " + String(e);
  }
}

async function syncFromGitHub() {
  syncing.value = true;
  message.value = "";
  try {
    const count = await invoke<number>("sync_programming_languages_from_github");
    message.value = `已同步 ${count} 个编程语言`;
    await loadLanguages();
  } catch (e) {
    message.value = "同步失败: " + String(e);
  } finally {
    syncing.value = false;
  }
}

function openAdd() {
  modalMode.value = "add";
  modalForm.value = { id: null, name: "", short_name: "" };
  showModal.value = true;
}

function openEdit(lang: ProgrammingLanguage) {
  modalMode.value = "edit";
  modalForm.value = { id: lang.id, name: lang.name, short_name: lang.short_name || "" };
  showModal.value = true;
}

async function handleSave(data: { id: number | null; name: string; short_name: string }) {
  try {
    if (modalMode.value === "add") {
      await invoke("add_programming_language", {
        name: data.name.trim(),
        shortName: data.short_name.trim(),
      });
    } else {
      await invoke("update_programming_language", {
        id: data.id,
        name: data.name.trim(),
        shortName: data.short_name.trim(),
      });
    }
    showModal.value = false;
    message.value = modalMode.value === "add" ? "已添加编程语言" : "已更新编程语言";
    await loadLanguages();
  } catch (e) {
    message.value = "保存失败: " + String(e);
  }
}

async function handleDelete(lang: ProgrammingLanguage) {
  if (!confirm(`确定要删除编程语言 "${lang.name}" 吗？`)) return;
  try {
    await invoke("delete_programming_language", { id: lang.id });
    message.value = "已删除编程语言";
    await loadLanguages();
  } catch (e) {
    message.value = "删除失败: " + String(e);
  }
}

/** 表格列配置 */
const columns = [
  { key: "name", title: "名称" },
  { key: "short_name", title: "简称" },
];

/** 处理行点击编辑 */
function handleRowClick(row: ProgrammingLanguage) {
  openEdit(row);
}
</script>

<template>
  <div>
    <!-- 消息提示 -->
    <StandardizedMessage
      v-if="message"
      type="success"
      :message="message"
      :duration="3000"
      @close="message = ''"
    />

    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <span class="total-count">总计: {{ languages.length }}</span>
      </div>
      <div class="toolbar-right">
        <StandardizedInput
          v-model="searchQuery"
          placeholder="搜索编程语言 (名称 / 简称)..."
          size="md"
          clearable
        />

        <StandardizedButton
          variant="outline"
          size="md"
          :loading="syncing"
          @click="syncFromGitHub"
        >
          {{ syncing ? "同步中..." : "从 GitHub 同步" }}
        </StandardizedButton>

        <StandardizedButton
          variant="primary"
          size="md"
          @click="openAdd"
        >
          新增编程语言
        </StandardizedButton>
      </div>
    </div>

    <!-- 编程语言表格 -->
    <StandardizedTable
      :columns="columns"
      :data="languages"
      :pageSize="pageSize"
      :searchQuery="searchQuery"
      :searchFields="['name', 'short_name']"
      rowKey="id"
      showIndex
      striped
      hoverable
      clickable
      emptyText="暂无编程语言数据，请从 GitHub 同步或手动添加。"
      @row-click="handleRowClick"
    >
      <!-- 操作列 -->
      <template #actions="{ row }">
        <StandardizedButton
          variant="outline"
          size="sm"
          @click.stop="openEdit(row)"
        >
          编辑
        </StandardizedButton>

        <StandardizedButton
          variant="danger"
          size="sm"
          @click.stop="handleDelete(row)"
        >
          删除
        </StandardizedButton>
      </template>
    </StandardizedTable>

    <!-- 编辑弹窗 -->
    <LanguageFormModal
      :show="showModal"
      :mode="modalMode"
      :language="modalForm"
      @save="handleSave"
      @close="showModal = false"
    />
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}

.toolbar-left {
  flex: 1;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.total-count {
  color: var(--text-secondary);
  font-size: 0.875rem;
}
</style>