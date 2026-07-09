<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { License } from "../types";
import LicenseFormModal from "../components/LicenseFormModal.vue";

const licenses = ref<License[]>([]);
const syncing = ref(false);
const message = ref("");
const searchQuery = ref("");

const showModal = ref(false);
const modalMode = ref<"add" | "edit">("add");
const modalForm = ref({
  id: null as number | null,
  spdx_id: "",
  full_name: "",
});

onMounted(() => loadLicenses());

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

const filtered = computed(() => {
  if (!searchQuery.value) return licenses.value;
  const q = searchQuery.value.toLowerCase();
  return licenses.value.filter(
    (l) =>
      l.spdx_id.toLowerCase().includes(q) ||
      l.full_name.toLowerCase().includes(q)
  );
});
</script>

<template>
  <div>
    <div v-if="message" class="card" style="margin-bottom: 1rem; border-color: var(--accent)">
      {{ message }}
    </div>

    <div style="display: flex; gap: 0.5rem; margin-bottom: 1rem; align-items: center; flex-wrap: wrap">
      <span style="color: var(--text-secondary); font-size: 0.875rem">总计: {{ licenses.length }}</span>
      <button class="btn btn-primary" @click="openAdd">新增 License</button>
      <button class="btn btn-outline" @click="syncFromSPDX" :disabled="syncing">
        {{ syncing ? "同步中..." : "从 SPDX 同步" }}
      </button>
    </div>

    <div class="card">
      <div style="margin-bottom: 1rem">
        <input
          type="text"
          v-model="searchQuery"
          placeholder="搜索 License (SPDX ID / 名称)..."
          class="search-input"
        />
      </div>

      <div v-if="licenses.length === 0" style="color: var(--text-secondary)">
        暂无 License 数据，请从 SPDX 同步或手动添加。
      </div>

      <div v-else class="license-grid">
        <div v-for="(lic, idx) in filtered" :key="lic.id ?? idx" class="license-card">
          <div class="license-id">{{ lic.spdx_id }}</div>
          <div class="license-name">{{ lic.full_name }}</div>
          <div class="license-actions">
            <button class="btn-sm" @click="openEdit(lic)">编辑</button>
            <button class="btn-sm btn-sm-danger" @click="handleDelete(lic)">删除</button>
          </div>
        </div>
      </div>
    </div>

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
.search-input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 8px;
  border: 1px solid var(--border);
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.875rem;
}
.license-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 0.75rem;
}
.license-card {
  padding: 0.75rem;
  border: 1px solid var(--border);
  border-radius: 8px;
}
.license-id {
  font-weight: 600;
  font-size: 0.875rem;
  margin-bottom: 0.25rem;
}
.license-name {
  font-size: 0.8125rem;
  color: var(--text-secondary);
  margin-bottom: 0.5rem;
}
.license-actions { display: flex; gap: 0.375rem; }
.btn-sm {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  border: 1px solid var(--border);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 0.75rem;
  cursor: pointer;
}
.btn-sm-danger { color: var(--error); border-color: var(--error); }
</style>
