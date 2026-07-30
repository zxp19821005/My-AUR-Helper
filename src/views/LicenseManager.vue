<!--
  LicenseManager.vue - License 管理页面

  功能：
  - 显示 License 列表（SPDX ID、全名）
  - 支持搜索、分页
  - 支持添加、编辑、删除 License
  - 支持从 SPDX 同步

  使用组件：
  - StandardizedTable: 表格组件
  - PageToolbar: 页面工具栏
  - LicenseFormModal: 弹窗组件
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { License } from "../types";
import LicenseFormModal from "../components/enum/LicenseFormModal.vue";
import { useSettingsStore } from "../stores/settings";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import { RefreshCw, Plus } from "@lucide/vue";

const settingsStore = useSettingsStore();
const licenses = ref<License[]>([]);
const syncing = ref(false);
const message = ref("");
const searchQuery = ref("");

const pageSize = ref(50);

const showModal = ref(false);
const modalMode = ref<"add" | "edit">("add");
const modalForm = ref({
  id: null as number | null,
  spdx_id: "",
  full_name: "",
});

onMounted(async () => {
  pageSize.value = await settingsStore.getSettingNumber("list_page_size_license", 50);
  await loadLicenses();
});

async function loadLicenses() {
  try {
    licenses.value = await invoke<License[]>("get_licenses");
  } catch (e) {
    message.value = "加载失败: " + String(e);
  }
}

async function syncFromSPDX() {
  syncing.value = true;
  message.value = "";
  try {
    const count = await invoke<number>("sync_licenses_from_spdx");
    message.value = `已同步 ${count} 个 SPDX License`;
    await loadLicenses();
  } catch (e) {
    message.value = "同步失败: " + String(e);
  } finally {
    syncing.value = false;
  }
}

function openAdd() {
  modalMode.value = "add";
  modalForm.value = { id: null, spdx_id: "", full_name: "" };
  showModal.value = true;
}

function openEdit(lic: License) {
  modalMode.value = "edit";
  modalForm.value = { id: lic.id, spdx_id: lic.spdx_id, full_name: lic.full_name };
  showModal.value = true;
}

async function handleSave(data: { id: number | null; spdx_id: string; full_name: string }) {
  try {
    if (modalMode.value === "add") {
      await invoke("add_license", {
        spdxId: data.spdx_id.trim(),
        fullName: data.full_name.trim(),
      });
    } else {
      await invoke("update_license", {
        id: data.id,
        spdxId: data.spdx_id.trim(),
        fullName: data.full_name.trim(),
      });
    }
    showModal.value = false;
    message.value = modalMode.value === "add" ? "已添加 License" : "已更新 License";
    await loadLicenses();
  } catch (e) {
    message.value = "保存失败: " + String(e);
  }
}

async function handleDelete(lic: License) {
  if (!confirm(`确定要删除 License "${lic.spdx_id}" 吗？`)) return;
  try {
    await invoke("delete_license", { id: lic.id });
    message.value = "已删除 License";
    await loadLicenses();
  } catch (e) {
    message.value = "删除失败: " + String(e);
  }
}

/** 表格列配置 */
const columns = [
  { key: "spdx_id", title: "SPDX ID" },
  { key: "full_name", title: "全名" },
];

/** 处理行点击编辑 */
function handleRowClick(row: License) {
  openEdit(row);
}
</script>

<template>
  <div>
    <PageToolbar v-model="searchQuery" @refresh="loadLicenses">
      <button
        class="btn-icon btn-icon-accent"
        :disabled="syncing"
        @click="syncFromSPDX"
        title="从 SPDX 同步"
      >
        <RefreshCw :size="16" :class="{ 'spinning': syncing }" />
      </button>
      <button
        class="btn-icon btn-icon-success"
        @click="openAdd"
        title="新增 License"
      >
        <Plus :size="16" />
      </button>
    </PageToolbar>

    <!-- License 表格 -->
    <StandardizedTable
      :columns="columns"
      :data="licenses"
      :pageSize="pageSize"
      :searchQuery="searchQuery"
      :searchFields="['spdx_id', 'full_name']"
      rowKey="id"
      showIndex
      striped
      hoverable
      clickable
      emptyText="暂无 License 数据，请从 SPDX 同步或手动添加。"
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

    <!-- 编辑弹窗 -->
    <LicenseFormModal
      :show="showModal"
      :mode="modalMode"
      :license="modalForm"
      @save="handleSave"
      @close="showModal = false"
    />
  </div>
</template>

<style scoped>
.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>