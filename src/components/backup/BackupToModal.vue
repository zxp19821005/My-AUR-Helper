<!--
  BackupToModal.vue - 备份到子目录弹窗组件

  功能：
  - 选择备份子目录
  - 执行缓存包备份操作
  - 显示备份进度和结果

  说明：基于 StandardizedModal 统一模态框风格（取代原先自写的 modal-overlay 结构）
-->
<script setup lang="ts">
import { ref, watch, inject } from "vue";
import { FOOTER_KEY, addMessage } from "../../composables/footer";
import * as cacheApi from "@/api/cache";
import StandardizedModal from "../common/StandardizedModal.vue";
import StandardizedButton from "../base/StandardizedButton.vue";

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

const footer = inject(FOOTER_KEY)!;
const backupToSubdirectory = ref("");
const backingUp = ref(false);

// 打开时重置选择（原 open() 行为）
watch(() => props.show, (val) => {
  if (val) backupToSubdirectory.value = "";
});

async function handleBackupTo() {
  if (!backupToSubdirectory.value && props.subdirectories.length > 0) {
    addMessage(footer, "warning", "请选择一个备份子目录");
    return;
  }
  backingUp.value = true;
  try {
    const result = await cacheApi.backupCacheToSubdirectory(
      props.selectedFilenames,
      props.backupPath,
      backupToSubdirectory.value,
    );
    emit("success", result);
    emit("close");
  } catch (e) {
    addMessage(footer, "error", `备份失败: ${e}`);
  } finally {
    backingUp.value = false;
  }
}
</script>

<template>
  <StandardizedModal
    :show="show"
    title="备份到子目录"
    width="sm"
    :closable="!backingUp"
    @close="emit('close')"
  >
    <div class="backup-body">
      <p>选中 {{ selectedFilenames.length }} 个缓存包</p>
      <div class="form-group">
        <label>选择备份子目录：</label>
        <select v-model="backupToSubdirectory" class="backup-dir-select">
          <option value="">根目录</option>
          <option v-for="dir in subdirectories" :key="dir" :value="dir">{{ dir }}</option>
        </select>
      </div>
    </div>

    <template #footer>
      <StandardizedButton variant="outline" size="sm" :disabled="backingUp" @click="emit('close')">
        取消
      </StandardizedButton>
      <StandardizedButton variant="primary" size="sm" :loading="backingUp" @click="handleBackupTo">
        {{ backingUp ? "备份中..." : "确认备份" }}
      </StandardizedButton>
    </template>
  </StandardizedModal>
</template>

<style scoped>
.backup-body p {
  margin: 0 0 0.75rem 0;
  color: var(--text-secondary);
  font-size: 0.875rem;
}
.form-group {
  margin-top: 0.5rem;
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
</style>
