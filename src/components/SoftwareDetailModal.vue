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
import { X } from "@lucide/vue";

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

const pkgTypes: Record<number, string> = {
  1: "编译安装",
  2: "二进制包",
  3: "Git 仓库",
  4: "AppImage",
};

const checkerTypes: Record<number, string> = {
  1: "GitHub Release",
  2: "GitHub Tag",
  3: "Gitee",
  4: "GitLab",
  5: "重定向",
  6: "HTTP 页面解析",
  7: "手动",
};

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
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="emit('close')">
      <div class="modal">
        <div class="modal-header">
          <h3>软件详情</h3>
          <button class="modal-close" @click="emit('close')">
            <X :size="18" />
          </button>
        </div>

        <div v-if="error" class="modal-error">{{ error }}</div>

        <div class="modal-body">
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
        </div>

        <div class="modal-footer">
          <button class="btn btn-outline" @click="emit('close')">关闭</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.modal {
  background: var(--bg-card);
  border-radius: 12px;
  min-width: 420px;
  max-width: 500px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
}
.modal-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}
.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.25rem;
  border-radius: 4px;
}
.modal-close:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.05);
}
.modal-error {
  padding: 0.5rem 1.25rem;
  color: var(--error);
  font-size: 0.8125rem;
  background: rgba(239, 83, 80, 0.08);
}
.modal-body {
  padding: 1rem 1.25rem;
  overflow-y: auto;
  flex: 1;
}
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0.75rem 1.25rem;
  border-top: 1px solid var(--border);
}

.loading-text {
  color: var(--text-secondary);
  text-align: center;
  padding: 1.5rem 0;
}

.info-table {
  width: 100%;
  border-collapse: collapse;
}
.info-table td {
  padding: 0.5rem 0;
  font-size: 0.875rem;
  vertical-align: middle;
  border-bottom: 1px solid var(--border);
}
.info-table td:last-child {
  border-bottom: none;
}
.info-table .label {
  color: var(--text-secondary);
  font-weight: 600;
  width: 120px;
  white-space: nowrap;
}
.info-table .value {
  color: var(--text-primary);
}
.url-cell {
  word-break: break-all;
  font-size: 0.8125rem;
}
</style>
