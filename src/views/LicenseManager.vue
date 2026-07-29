<!--
  LicenseManager.vue - License 管理页面

  功能：
  - 显示 License 列表（SPDX ID、全名）
  - 支持搜索、分页
  - 支持添加、编辑、删除 License
  - 支持从 SPDX 同步

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
import type { License } from "../types";
import LicenseFormModal from "../components/enum/LicenseFormModal.vue";
import { useSettingsStore } from "../stores/settings";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import StandardizedButton from "../components/base/StandardizedButton.vue";
import StandardizedMessage from "../components/base/StandardizedMessage.vue";
import StandardizedInput from "../components/base/StandardizedInput.vue";

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
        <span class="total-count">总计: {{ licenses.length }}</span>
      </div>
      <div class="toolbar-right">
        <StandardizedInput
          v-model="searchQuery"
          placeholder="搜索 License (SPDX ID / 名称)..."
          size="md"
          clearable
        />

        <StandardizedButton
          variant="outline"
          size="md"
          :loading="syncing"
          @click="syncFromSPDX"
        >
          {{ syncing ? "同步中..." : "从 SPDX 同步" }}
        </StandardizedButton>

        <StandardizedButton
          variant="primary"
          size="md"
          @click="openAdd"
        >
          新增 License
        </StandardizedButton>
      </div>
    </div>

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