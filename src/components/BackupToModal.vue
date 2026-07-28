<!--
  BackupToModal.vue - 备份到子目录弹窗组件

  功能：
  - 选择备份子目录
  - 执行缓存包备份操作
  - 显示备份进度和结果
-->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  show: boolean;
  selectedFilenames: string[];
  backupPath: string;
  subdirectories: string[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "success", result: [number, string[]]): void;
}>();

const backupToSubdirectory = ref("");
const backingUp = ref(false);

function handleClose() {
  if (!backingUp.value) emit("close");
}

async function handleBackupTo() {
  if (!backupToSubdirectory.value && props.subdirectories.length > 0) {
    alert("请选择一个备份子目录");
    return;
  }
  backingUp.value = true;
  try {
    const result = await invoke<[number, string[]]>("backup_cache_to_subdirectory", {
      filenames: props.selectedFilenames,
      backupPath: props.backupPath,
      subdirectory: backupToSubdirectory.value,
    });
    emit("success", result);
    emit("close");
  } catch (e) {
    alert(`备份失败: ${e}`);
  } finally {
    backingUp.value = false;
  }
}

function open() {
  backupToSubdirectory.value = "";
}

defineExpose({ open });
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click.self="handleClose">
      <div class="modal-content">
        <div class="modal-header">
          <h3>备份到子目录</h3>
          <button class="btn-icon" @click="handleClose" :disabled="backingUp">
            <span>&times;</span>
          </button>
        </div>
        <div class="modal-body">
          <p>选中 {{ selectedFilenames.length }} 个缓存包</p>
          <div class="form-group">
            <label>选择备份子目录：</label>
            <select v-model="backupToSubdirectory" class="backup-dir-select">
              <option value="">根目录</option>
              <option v-for="dir in subdirectories" :key="dir" :value="dir">{{ dir }}</option>
            </select>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary" @click="handleClose" :disabled="backingUp">取消</button>
          <button class="btn-primary" @click="handleBackupTo" :disabled="backingUp">
            {{ backingUp ? "备份中..." : "确认备份" }}
          </button>
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
.modal-content {
  background: var(--bg-card);
  border-radius: 12px;
  width: 420px;
  max-width: 90vw;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}
.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
}
.modal-header h3 {
  margin: 0;
  font-size: 1rem;
}
.modal-body {
  padding: 1.25rem;
}
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 1rem 1.25rem;
  border-top: 1px solid var(--border);
}
.form-group {
  margin-top: 1rem;
}
.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
  color: var(--text-secondary);
}
.backup-dir-select {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background-color: var(--bg-card);
  color: var(--text-primary);
  font-size: 0.875rem;
  outline: none;
}
.backup-dir-select:focus {
  border-color: var(--accent);
}
.btn-secondary {
  padding: 0.5rem 1rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 0.875rem;
}
.btn-secondary:hover:not(:disabled) {
  background: var(--bg-secondary);
}
.btn-primary {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  background: var(--accent);
  color: white;
  cursor: pointer;
  font-size: 0.875rem;
}
.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}
.btn-primary:disabled,
.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
