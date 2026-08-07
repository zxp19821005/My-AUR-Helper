<!--
  LanguageManager.vue - 编程语言管理页面

  功能：
  - 显示编程语言列表（名称、简称）
  - 支持搜索、分页
  - 支持添加、编辑、删除语言

  使用组件：
  - StandardizedTable: 表格组件
  - PageToolbar: 页面工具栏
  - LanguageFormModal: 弹窗组件
  - PaginationControls: 分页控件
-->
<script setup lang="ts">
import { ref, computed, reactive, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { EnumProgrammingLanguage as ProgrammingLanguage } from "../types";
import type { FooterState } from "../composables/footer";
import { defaultFooterState } from "../composables/footer";
import LanguageFormModal from "../components/enum/LanguageFormModal.vue";
import { useSettingsStore } from "../stores/settings";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import PaginationControls from "../components/common/PaginationControls.vue";
import { Plus } from "@lucide/vue";

const settingsStore = useSettingsStore();
const languages = ref<ProgrammingLanguage[]>([]);
const message = ref("");
const searchQuery = ref("");
const currentPage = ref(1);

const pageSize = ref(50);

const showModal = ref(false);
const modalMode = ref<"add" | "edit">("add");
const modalForm = ref({
  id: null as number | null,
  name: "",
  short_name: "",
});

const footer = reactive<FooterState>(defaultFooterState());

const filteredEntries = computed(() => {
  let result = languages.value;
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    result = result.filter(
      (l) =>
        l.name.toLowerCase().includes(q) ||
        (l.short_name && l.short_name.toLowerCase().includes(q))
    );
  }
  return result;
});

function goToPage(page: number) {
  const totalPages = Math.ceil(filteredEntries.value.length / pageSize.value) || 1;
  if (page < 1 || page > totalPages) return;
  currentPage.value = page;
}

function syncToolbar() {
  const total = filteredEntries.value.length;
  footer.infoText = `共 ${total} 条`;
  footer.showPagination = total > pageSize.value;
  footer.totalRecords = total;
  footer.currentPage = currentPage.value;
  footer.pageSize = pageSize.value;
  footer.onPageChange = goToPage;
}

watch(filteredEntries, syncToolbar, { immediate: true });
watch(searchQuery, () => { currentPage.value = 1; });
watch(currentPage, syncToolbar);

onMounted(async () => {
  pageSize.value = await settingsStore.getSettingNumber("list_page_size_language", 50);
  await loadLanguages();
});

async function loadLanguages() {
  try {
    languages.value = await invoke<ProgrammingLanguage[]>("get_languages");
  } catch (e) {
    message.value = "加载失败: " + String(e);
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
    await invoke("upsert_language", {
      language: {
        id: data.id,
        name: data.name.trim(),
        short_name: data.short_name.trim(),
      },
    });
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
    await invoke("delete_language", { name: lang.name });
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
  <div class="language-manager">
    <PageToolbar v-model="searchQuery" @refresh="loadLanguages">
      <button
        class="btn-icon btn-icon-success"
        @click="openAdd"
        title="新增编程语言"
      >
        <Plus :size="16" />
      </button>
    </PageToolbar>

    <!-- 编程语言表格 -->
    <StandardizedTable
      :columns="columns"
      :data="filteredEntries"
      :pageSize="pageSize"
      rowKey="id"
      showIndex
      striped
      hoverable
      clickable
      :showPagination="false"
      emptyText="暂无编程语言数据，请手动添加。"
      @row-click="handleRowClick"
    >
      <!-- 操作列 -->
      <template #actions="{ row }">
        <button
          class="btn-icon btn-icon-default"
          @click.stop="openEdit(row)"
          title="编辑"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/></svg>
        </button>
        <button
          class="btn-icon btn-icon-danger"
          @click.stop="handleDelete(row)"
          title="删除"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
        </button>
      </template>
    </StandardizedTable>

    <!-- 底部分页 -->
    <div class="language-footer">
      <PaginationControls :footer="footer" />
    </div>

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
.language-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.language-footer {
  display: flex;
  justify-content: center;
  padding: 0.75rem 0;
  border-top: 1px solid var(--border);
}
</style>
