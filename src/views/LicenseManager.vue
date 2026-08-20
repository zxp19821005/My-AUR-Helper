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
  - BaseFormModal: 新增/编辑弹窗（通用表单弹窗）
  - PaginationControls: 分页控件
-->
<script setup lang="ts">
import { ref, computed, reactive, watch, onMounted } from "vue";
import type { License } from "../types";
import type { FooterState } from "../composables/footer";
import { defaultFooterState } from "../composables/footer";
import * as licenseApi from "@/api/license";
import BaseFormModal, { type FormField } from "../components/common/BaseFormModal.vue";
import { useSettingsStore } from "../stores/settings";
import { openConfirm as confirm } from "../composables/useConfirm";
import PageToolbar from "../components/common/PageToolbar.vue";
import StandardizedTable from "../components/common/StandardizedTable.vue";
import PaginationControls from "../components/common/PaginationControls.vue";
import { Icon } from "../icons";

const settingsStore = useSettingsStore();
const licenses = ref<License[]>([]);
const syncing = ref(false);
const message = ref("");
const searchQuery = ref("");
const currentPage = ref(1);

const pageSize = ref(50);

const showModal = ref(false);
const modalMode = ref<"add" | "edit">("add");
const modalId = ref<number | null>(null);
const modalValues = ref<Record<string, string>>({ spdx_id: "", full_name: "" });

const licenseFields: FormField[] = [
  { key: "spdx_id", label: "SPDX ID", placeholder: "如 MIT, GPL-3.0-only", required: true },
  { key: "full_name", label: "完整名称", placeholder: "如 MIT License", required: true },
];

const footer = reactive<FooterState>(defaultFooterState());

const filteredEntries = computed(() => {
  let result = licenses.value;
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    result = result.filter(
      (l) =>
        l.spdx_id.toLowerCase().includes(q) ||
        l.full_name.toLowerCase().includes(q)
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
  pageSize.value = await settingsStore.getSettingNumber("list_page_size_license", 50);
  await loadLicenses();
});

async function loadLicenses() {
  try {
    licenses.value = await licenseApi.getLicenses();
  } catch (e) {
    message.value = "加载失败: " + String(e);
  }
}

async function syncFromSPDX() {
  syncing.value = true;
  message.value = "";
  try {
    const count = await licenseApi.syncLicensesFromSpdx();
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
  modalId.value = null;
  modalValues.value = { spdx_id: "", full_name: "" };
  showModal.value = true;
}

function openEdit(lic: License) {
  modalMode.value = "edit";
  modalId.value = lic.id;
  modalValues.value = { spdx_id: lic.spdx_id, full_name: lic.full_name };
  showModal.value = true;
}

async function handleSave(data: Record<string, string>) {
  try {
    if (modalMode.value === "add") {
      await licenseApi.addLicense(data.spdx_id.trim(), data.full_name.trim());
    } else {
      await licenseApi.updateLicense(modalId.value!, data.spdx_id.trim(), data.full_name.trim());
    }
    showModal.value = false;
    message.value = modalMode.value === "add" ? "已添加 License" : "已更新 License";
    await loadLicenses();
  } catch (e) {
    message.value = "保存失败: " + String(e);
  }
}

async function handleDelete(lic: License) {
  if (!(await confirm({ message: `确定要删除 License "${lic.spdx_id}" 吗？`, variant: "danger" }))) return;
  try {
    await licenseApi.deleteLicense(lic.id!);
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
  <div class="license-manager">
    <PageToolbar v-model="searchQuery" @refresh="loadLicenses">
      <button
        class="btn-icon btn-icon-accent"
        :disabled="syncing"
        @click="syncFromSPDX"
        title="从 SPDX 同步"
      >
        <component :is="Icon.syncAur" :size="16" :class="{ 'spinning': syncing }" />
      </button>
      <button
        class="btn-icon btn-icon-success"
        @click="openAdd"
        title="新增 License"
      >
        <component :is="Icon.actionAdd" :size="16" />
      </button>
    </PageToolbar>

    <!-- License 表格 -->
    <StandardizedTable
      :columns="columns"
      :data="filteredEntries"
      :pageSize="pageSize"
      :current-page="currentPage"
      rowKey="id"
      showIndex
      striped
      hoverable
      clickable
      :showPagination="false"
      emptyText="暂无 License 数据，请从 SPDX 同步或手动添加。"
      @row-click="handleRowClick"
    >
      <!-- 操作列 -->
      <template #actions="{ row }">
        <button
          class="btn-icon btn-icon-warning"
          @click.stop="openEdit(row)"
          title="编辑"
        >
          <component :is="Icon.actionEdit" :size="14" />
        </button>
        <button
          class="btn-icon btn-icon-danger"
          @click.stop="handleDelete(row)"
          title="删除"
        >
          <component :is="Icon.actionDelete" :size="14" />
        </button>
      </template>
    </StandardizedTable>

    <!-- 底部分页 -->
    <div class="license-footer">
      <PaginationControls :footer="footer" />
    </div>

    <!-- 编辑弹窗 -->
    <BaseFormModal
      :show="showModal"
      :mode="modalMode"
      entity-name="License"
      :fields="licenseFields"
      :model-value="modalValues"
      @save="handleSave"
      @close="showModal = false"
    />
  </div>
</template>

<style scoped>
.license-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.license-footer {
  display: flex;
  justify-content: center;
  padding: 0.75rem 0;
  border-top: 1px solid var(--border);
}
</style>
