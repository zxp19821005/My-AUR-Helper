<!--
  SoftwareDetailModal.vue - 软件包详情弹窗（只读）

  Props:
  - show: boolean
  - pkgname: string

  Events:
  - close: 关闭弹窗
-->
<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { SoftwareInfo } from "../types";
import { pkgTypes, checkerTypes } from "../utils/enums";
import Modal from "./common/Modal.vue";

const props = defineProps<{
  show: boolean;
  pkgname: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

const pkg = ref<SoftwareInfo | null>(null);
const loading = ref(false);
const error = ref("");

async function loadSoftware() {
  if (!props.pkgname) return;
  loading.value = true;
  error.value = "";
  try {
    pkg.value = await invoke<SoftwareInfo | null>("get_software", { pkgname: props.pkgname });
    if (!pkg.value) error.value = "未找到软件包";
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.show,
  (val) => {
    if (val) loadSoftware();
  }
);
</script>

<template>
  <Modal :show="show" title="软件详情" @close="emit('close')">
    <template #error v-if="error">{{ error }}</template>
    <div v-if="loading" class="loading-text">加载中...</div>
    <table v-else-if="pkg" class="info-table">
      <tbody>
        <tr>
          <td class="label">包名</td>
          <td class="value">{{ pkg.pkgname }}</td>
        </tr>
        <tr>
          <td class="label">软件类型</td>
          <td class="value">{{ pkgTypes[pkg.package_type_id] || pkg.package_type_id }}</td>
        </tr>
        <tr>
          <td class="label">上游地址</td>
          <td class="value url-cell">{{ pkg.upstream_url || "-" }}</td>
        </tr>
        <tr>
          <td class="label">检查器类型</td>
          <td class="value">{{ checkerTypes[pkg.checker_type_id] || pkg.checker_type_id }}</td>
        </tr>
        <tr>
          <td class="label">状态</td>
          <td>
            <span class="status-badge" :class="pkg.is_outdated ? 'status-update_available' : 'status-up_to_date'">
              {{ pkg.is_outdated ? "需更新" : "已最新" }}
            </span>
          </td>
        </tr>
        <tr>
          <td class="label">自动检查更新</td>
          <td class="value">{{ pkg.auto_check_enabled ? "已启用" : "已禁用" }}</td>
        </tr>
        <tr>
          <td class="label">检查测试版本</td>
          <td class="value">{{ pkg.check_test_versions ? "已启用" : "已禁用" }}</td>
        </tr>
        <tr>
          <td class="label">检查二进制文件</td>
          <td class="value">{{ pkg.check_binary_files ? "已启用" : "已禁用" }}</td>
        </tr>
      </tbody>
    </table>
    <template #footer>
      <button class="btn btn-outline" @click="emit('close')">关闭</button>
    </template>
  </Modal>
</template>

<style scoped>
.loading-text {
  color: var(--text-secondary);
  text-align: center;
  padding: 1.5rem 0;
}

.url-cell {
  word-break: break-all;
  font-size: 0.8125rem;
}
</style>
