<!--
  BackupManager.vue - 备份管理页面

  功能：
  - 执行备份操作
  - 显示备份结果

  数据来源：
  - run_backup: 执行备份
  - get_setting: 获取备份目录设置
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";                    // Vue 核心 API
import { invoke } from "@tauri-apps/api/core";            // Tauri IPC 调用
import type { BackupResult, Setting } from "../types";   // 类型定义
import { useToolbarStore } from "../stores/toolbar";
import PageToolbar from "../components/PageToolbar.vue";

const toolbar = useToolbarStore();

/** 备份结果 */
const result = ref<BackupResult | null>(null);

/** 运行中状态 */
const loading = ref(false);

/** 备份目录 - 从设置中读取 */
const backupDir = ref("/run/media/zxp/Backup/Linux/ZST");

/** 组件挂载时加载备份目录设置 */
onMounted(async () => {
  try {
    const setting = await invoke<Setting | null>("get_setting", { key: "backup_dir" });
    if (setting) backupDir.value = setting.value;
  } catch { /* 忽略获取设置错误 */ }
  toolbar.setInfo(`备份目录: ${backupDir.value}`);
});

/** 执行备份 */
async function runBackup() {
  loading.value = true;
  toolbar.setProgress(0, 1);
  result.value = null;
  try {
    result.value = await invoke<BackupResult>("run_backup", { backupPath: backupDir.value });
    if (result.value) {
      toolbar.setProgress(result.value.copied + result.value.removed, result.value.copied + result.value.removed);
      setTimeout(() => toolbar.clearProgress(), 2000);
    }
  } catch (e) {
    result.value = { copied: 0, removed: 0, errors: [String(e)] };
  } finally {
    loading.value = false;
    toolbar.clearProgress();
  }
}
</script>

<template>
  <div>
    <PageToolbar>
      <button class="btn btn-primary" @click="runBackup" :disabled="loading">
        {{ loading ? "运行中..." : "执行备份" }}
      </button>
    </PageToolbar>

    <div class="card" style="margin-bottom: 1rem">
      <p style="color: var(--text-secondary); font-size: 0.875rem">
        备份目录: <strong>{{ backupDir }}</strong>
      </p>
    </div>

    <div v-if="result" class="card">
      <h3>备份结果</h3>
      <div style="margin-top: 0.5rem; font-size: 0.875rem">
        <span style="color: var(--success)">已复制: {{ result.copied }}</span>
        &nbsp;
        <span style="color: var(--warning)">已清理: {{ result.removed }}</span>
      </div>
      <div v-if="result.errors.length" style="color: var(--error); margin-top: 0.5rem">
        <div v-for="(err, i) in result.errors" :key="i">{{ err }}</div>
      </div>
    </div>
  </div>
</template>
