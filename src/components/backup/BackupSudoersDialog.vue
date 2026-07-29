<!--
  BackupSudoersDialog.vue - sudoers 配置提示弹窗

  功能：
  - 提示用户配置 sudoers 免密以获取 root 权限
  - 显示需要执行的命令
  - 提供重试和取消操作
-->
<script setup lang="ts">
import StandardizedModal from "../components/common/StandardizedModal.vue";

defineProps<{
  show: boolean;
  sudoersCommand: string;
  pendingInstallPath: string;
  pendingInstallPkgname: string;
}>();

const emit = defineEmits<{
  close: [];
  retry: [path: string, pkgname: string];
}>();
</script>

<template>
  <StandardizedModal
    :show="show"
    title="需要配置 sudoers 免密"
    @close="emit('close')"
    width="600px"
  >
    <div class="sudoers-content">
      <p>安装备份包需要 root 权限。请在终端中执行以下命令配置 sudoers 免密：</p>
      <pre class="sudoers-command">{{ sudoersCommand }}</pre>
      <p class="hint">配置完成后，点击"重试"按钮继续安装。</p>
    </div>
    <template #footer>
      <button class="modal-btn" @click="emit('close')">取消</button>
      <button class="modal-btn primary" @click="emit('retry', pendingInstallPath, pendingInstallPkgname)">重试</button>
    </template>
  </StandardizedModal>
</template>

<style scoped>
.sudoers-content p {
  margin: 0.5rem 0;
  color: var(--text-primary);
  font-size: 0.875rem;
}

.sudoers-command {
  font-family: monospace;
  font-size: 0.8125rem;
  background: var(--bg-secondary);
  padding: 1rem;
  border-radius: 8px;
  margin: 0.75rem 0;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--accent);
}

.hint {
  color: var(--text-secondary);
  font-size: 0.8125rem;
  margin-top: 0.5rem;
}
</style>