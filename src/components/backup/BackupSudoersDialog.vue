<!--
  BackupSudoersDialog.vue - sudoers 配置提示弹窗

  功能：
  - 提示用户配置 sudoers 免密以获取 root 权限
  - 显示需要执行的命令
  - 提供「安装」（继续安装）和「取消」操作

  实现约定：
  - 统一使用 StandardizedModal 作为弹窗容器（与全局对话框风格一致）
  - 底部操作区统一使用 StandardizedButton，带图标、彩色、右对齐
-->
<script setup lang="ts">
import { Icon } from "../../icons";
import StandardizedModal from "../common/StandardizedModal.vue";
import StandardizedButton from "../base/StandardizedButton.vue";

defineProps<{
  show: boolean;
  sudoersCommand: string;
  pendingInstallPath: string;
  pendingInstallPkgname: string;
  /** 是否正在安装中（用于禁用按钮并显示加载态） */
  installing: boolean;
}>();

const emit = defineEmits<{
  close: [];
  install: [path: string, pkgname: string];
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
      <p class="hint">配置完成后，点击"安装"按钮继续安装。</p>
    </div>
    <template #footer>
      <StandardizedButton variant="outline" size="sm" :disabled="installing" @click="emit('close')">
        <template #icon>
          <component :is="Icon.actionClear" :size="16" />
        </template>
        取消
      </StandardizedButton>
      <StandardizedButton
        variant="primary"
        tone="success"
        size="sm"
        :loading="installing"
        @click="emit('install', pendingInstallPath, pendingInstallPkgname)"
      >
        <template #icon>
          <component :is="Icon.install" :size="16" />
        </template>
        安装
      </StandardizedButton>
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
