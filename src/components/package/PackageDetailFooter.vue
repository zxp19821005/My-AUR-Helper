<!--
  PackageDetailFooter.vue - 软件包详情底部操作栏

  功能：
  - 提供编辑、更新AUR、同步PKGBUILD、检查上游、删除操作按钮
  - 显示加载状态
-->
<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import type { SoftwareDetail } from "../../types";
import { Edit, Trash2, RefreshCw, FileCode, GitBranch } from "@lucide/vue";

const props = defineProps<{
  detail: SoftwareDetail | null;
}>();

const emit = defineEmits<{
  edit: [];
  saved: [];
  deleted: [];
  "update-aur": [];
  "update-pkgbuild": [];
  "check-update": [];
}>();

const deleting = defineModel<boolean>("deleting", { required: true });
const updatingAur = defineModel<boolean>("updatingAur", { required: true });
const updatingPkgbuild = defineModel<boolean>("updatingPkgbuild", { required: true });
const checking = defineModel<boolean>("checking", { required: true });
const successMsg = defineModel<string>("successMsg", { required: true });
const error = defineModel<string>("error", { required: true });

async function handleDelete() {
  if (!props.detail?.software_id) return;
  if (!confirm(`确定要删除软件包 "${props.detail.pkgname}" 吗？`)) return;

  deleting.value = true;
  error.value = "";
  try {
    await invoke("delete_software", { softwareId: props.detail.software_id });
    emit("deleted");
  } catch (e) {
    error.value = String(e);
  } finally {
    deleting.value = false;
  }
}

async function updateAurInfo() {
  if (!props.detail) return;
  updatingAur.value = true;
  error.value = "";
  try {
    await invoke<number>("update_aur_info", { pkgnameList: [props.detail.pkgname] });
    successMsg.value = "AUR 信息更新完成";
    emit("update-aur");
  } catch (e) {
    error.value = String(e);
  } finally {
    updatingAur.value = false;
    setTimeout(() => { successMsg.value = ""; }, 2000);
  }
}

async function updatePkgbuild() {
  if (!props.detail) return;
  updatingPkgbuild.value = true;
  error.value = "";
  try {
    await invoke<number>("sync_from_pkgbuild", { pkgname: props.detail.pkgname });
    successMsg.value = "PKGBUILD 信息更新完成";
    emit("update-pkgbuild");
  } catch (e) {
    error.value = String(e);
  } finally {
    updatingPkgbuild.value = false;
    setTimeout(() => { successMsg.value = ""; }, 2000);
  }
}

async function checkUpdate() {
  if (!props.detail) return;
  checking.value = true;
  error.value = "";
  try {
    await invoke<string>("check_upstream_version", { pkgname: props.detail.pkgname });
    successMsg.value = "上游版本检查完成";
    emit("check-update");
  } catch (e) {
    error.value = String(e);
  } finally {
    checking.value = false;
    setTimeout(() => { successMsg.value = ""; }, 2000);
  }
}
</script>

<template>
  <div class="detail-footer">
    <div class="footer-actions">
      <button
        class="toolbar-btn btn-blue"
        @click="emit('edit')"
        title="编辑"
      >
        <Edit :size="16" />
        <span>编辑</span>
      </button>

      <button
        class="toolbar-btn btn-purple"
        :disabled="updatingAur"
        @click="updateAurInfo"
        title="更新 AUR 信息"
      >
        <GitBranch :size="16" :class="{ spinning: updatingAur }" />
        <span>{{ updatingAur ? '更新中...' : 'AUR 信息' }}</span>
      </button>

      <button
        class="toolbar-btn btn-teal"
        :disabled="updatingPkgbuild"
        @click="updatePkgbuild"
        title="同步 PKGBUILD"
      >
        <FileCode :size="16" :class="{ spinning: updatingPkgbuild }" />
        <span>{{ updatingPkgbuild ? '更新中...' : 'PKGBUILD' }}</span>
      </button>

      <button
        class="toolbar-btn btn-green"
        :disabled="checking"
        @click="checkUpdate"
        title="检查上游更新"
      >
        <RefreshCw :size="16" :class="{ spinning: checking }" />
        <span>{{ checking ? '检查中...' : '检查上游' }}</span>
      </button>

      <button
        class="toolbar-btn btn-red"
        :disabled="deleting"
        @click="handleDelete"
        title="删除"
      >
        <Trash2 :size="16" :class="{ spinning: deleting }" />
        <span>{{ deleting ? '删除中...' : '删除' }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.detail-footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.75rem 1.25rem;
  background-color: var(--bg-primary);
  border-top: 1px solid var(--border);
}

.footer-actions {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  max-width: 900px;
  margin: 0 auto;
}

@media (max-width: 768px) {
  .footer-actions {
    flex-direction: column;
  }

  .footer-actions > * {
    width: 100%;
  }
}
</style>